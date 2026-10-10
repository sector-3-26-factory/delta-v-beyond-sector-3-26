// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Streaming system for asteroid belt sectors and fields.
//!
//! This module manages the dynamic loading and unloading of asteroid belt sectors
//! and fields based on the player's position. Per the asteroid belt plan
//! (plans/asteroid-belt-plan.md Phase 7):
//! - Belt sectors are spawned when the player is within streaming distance
//! - Fields are spawned using the same rule, measured from the field's current position
//! - Sector identity comes from orbital position, not a running counter
//! - Analytic propagation at sector load, not rigid rotation
//! - Moons are NOT streamed (they exist from world load)

use bevy::prelude::*;
use delta_v_core::{AppState, PlayerShipEntity, Targetable, WorldEntityId};
use delta_v_types::PlayerSettings;
use delta_v_types::collision::layers::ASTEROID;
use std::hash::{Hash, Hasher};

use crate::belt_field_generation::{
    AsteroidDeltaStore, BeltSectorParams, FieldParams, GeneratedAsteroid, SizeDistributionEntry,
    StableBodyId, generate_belt_sector, generate_field, propagate_displaced_asteroid,
    separate_overlaps,
};
use crate::belt_field_spawn::{AsteroidBelt, AsteroidField};
use crate::celestial::{LazyLoadMesh, OrbitalBody};
use crate::constants::{COLLISION_RELEVANCE_PX, STREAMING_DISTANCE_M};
use crate::{CollisionLayersComponent, CollisionShape, Navigable, RigidBody};

/// Component marking a belt sector that has been generated.
#[derive(Component, Clone, Debug)]
pub struct BeltSector {
    /// Unique identifier for the belt.
    pub belt_id: String,
    /// Sector index along the belt's orbital ring.
    pub sector_index: i64,
    /// Generated asteroids in this sector.
    pub generated_asteroids: Vec<GeneratedAsteroid>,
}

/// Component marking a field that has been generated.
#[derive(Component, Clone, Debug)]
pub struct GeneratedField {
    /// Unique identifier for the field.
    pub field_id: String,
    /// Generated asteroids in this field.
    pub generated_asteroids: Vec<GeneratedAsteroid>,
}

/// Resource tracking which sectors/fields are currently loaded.
// allow-default: Bevy Resource trait bound requires Default for init_resource
#[derive(Debug, Default, Resource)]
pub struct LoadedSectors {
    /// Currently loaded belt sectors as (`belt_id`, `sector_index`) pairs.
    pub belt_sectors: Vec<(String, i64)>,
    /// Currently loaded field IDs.
    pub fields: Vec<String>,
}

/// System that determines which belt sectors should be loaded based on player position.
///
/// Runs in `FixedUpdate` during `AppState::InGame` per ADR-0017.
#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
pub fn belt_sector_streaming_system(
    mut commands: Commands<'_, '_>,
    mut loaded_sectors: ResMut<'_, LoadedSectors>,
    delta_store: ResMut<'_, AsteroidDeltaStore>,
    time: Res<'_, Time<Fixed>>,
    player_settings: Res<'_, PlayerSettings>,
    player_query: Query<'_, '_, &GlobalTransform, With<PlayerShipEntity>>,
    belt_query: Query<'_, '_, (Entity, &AsteroidBelt, &GlobalTransform, &OrbitalBody)>,
    existing_sectors: Query<'_, '_, (Entity, &BeltSector)>,
    asset_server: Res<'_, AssetServer>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation();
    let simulation_time = time.elapsed_secs();
    let detail_level = player_settings.detail_level;

    // Determine which sectors should be loaded
    let mut should_load: Vec<(String, i64)> = Vec::new();

    for (_belt_entity, belt, belt_transform, orbital_body) in &belt_query {
        // Get the belt's current position (center of the ring)
        let belt_pos = belt_transform.translation();

        // Calculate distance from player to belt center
        let distance = player_pos.distance(belt_pos);

        // Check if within streaming distance
        if distance <= STREAMING_DISTANCE_M {
            // Calculate which sector the player is near
            // Sector index is based on orbital angle around the parent
            // Use the belt's id (string identifier from world definition) for sector identity
            let sector_index = calculate_sector_index(
                &belt.id, // Use belt's string ID for sector identity
                orbital_body.orbital_distance,
                belt.sector_size,
                player_pos,
            );

            should_load.push((belt.id.clone(), sector_index));
        }
    }

    // Despawn sectors that are no longer needed
    let mut to_despawn: Vec<Entity> = Vec::new();
    for (entity, sector) in &existing_sectors {
        let still_needed = should_load
            .iter()
            .any(|(id, idx)| id == &sector.belt_id && *idx == sector.sector_index);
        if !still_needed {
            to_despawn.push(entity);
            // Remove from loaded sectors tracking
            loaded_sectors
                .belt_sectors
                .retain(|(id, idx)| id != &sector.belt_id || *idx != sector.sector_index);
        }
    }
    for entity in to_despawn {
        commands.entity(entity).despawn();
    }

    // Spawn new sectors that are needed
    for (_belt_entity, belt, belt_transform, orbital_body) in &belt_query {
        for (belt_id, sector_index) in &should_load {
            if belt_id != &belt.id {
                continue;
            }

            // Check if already loaded
            let already_loaded = existing_sectors
                .iter()
                .any(|(_, s)| s.belt_id == *belt_id && s.sector_index == *sector_index);
            if already_loaded {
                continue;
            }

            // Generate the sector
            let params = build_belt_sector_params(
                belt,
                belt_transform,
                orbital_body,
                *sector_index,
                simulation_time,
                detail_level,
            );
            let mut asteroids = generate_belt_sector(&params);

            // Apply delta store modifications
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            belt_id.hash(&mut hasher);
            let belt_hash = hasher.finish();
            apply_deltas_to_sector(
                &mut asteroids,
                &delta_store,
                StableBodyId {
                    kind: crate::belt_field_generation::BodyKind::Belt,
                    id: belt_hash,
                },
                simulation_time,
            );

            // Separate overlaps
            separate_overlaps(&mut asteroids);

            // Spawn the asteroids
            let sector_entity = commands
                .spawn((
                    Name::new(format!("BeltSector: {belt_id} sector {sector_index}")),
                    BeltSector {
                        belt_id: belt_id.clone(),
                        sector_index: *sector_index,
                        generated_asteroids: asteroids.clone(),
                    },
                ))
                .id();

            // Spawn each asteroid as a child entity
            for asteroid in asteroids {
                spawn_generated_asteroid(&mut commands, &asset_server, asteroid, sector_entity);
            }

            // Track as loaded
            loaded_sectors
                .belt_sectors
                .push((belt_id.clone(), *sector_index));
        }
    }
}

/// System that determines which fields should be loaded based on player position.
///
/// Runs in `FixedUpdate` during `AppState::InGame` per ADR-0017.
#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
pub fn field_streaming_system(
    mut commands: Commands<'_, '_>,
    mut loaded_sectors: ResMut<'_, LoadedSectors>,
    delta_store: ResMut<'_, AsteroidDeltaStore>,
    time: Res<'_, Time<Fixed>>,
    player_settings: Res<'_, PlayerSettings>,
    player_query: Query<'_, '_, &GlobalTransform, With<PlayerShipEntity>>,
    field_query: Query<
        '_,
        '_,
        (
            Entity,
            &AsteroidField,
            &GlobalTransform,
            Option<&OrbitalBody>,
        ),
    >,
    existing_fields: Query<'_, '_, (Entity, &GeneratedField)>,
    asset_server: Res<'_, AssetServer>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation();
    let simulation_time = time.elapsed_secs();
    let detail_level = player_settings.detail_level;

    // Determine which fields should be loaded
    let mut should_load: Vec<String> = Vec::new();

    for (_field_entity, field, field_transform, orbital_body) in &field_query {
        // Get the field's current position
        let field_pos = orbital_body.map_or_else(
            || field_transform.translation(),
            |orbital| compute_orbital_position(orbital, simulation_time),
        );

        // Check if within streaming distance
        let distance = player_pos.distance(field_pos);
        if distance <= STREAMING_DISTANCE_M {
            should_load.push(field.id.clone());
        }
    }

    // Despawn fields that are no longer needed
    let mut to_despawn: Vec<Entity> = Vec::new();
    for (entity, gen_field) in &existing_fields {
        if !should_load.contains(&gen_field.field_id) {
            to_despawn.push(entity);
            loaded_sectors.fields.retain(|id| id != &gen_field.field_id);
        }
    }
    for entity in to_despawn {
        commands.entity(entity).despawn();
    }

    // Spawn new fields that are needed
    for (_field_entity, field, field_transform, orbital_body) in &field_query {
        for field_id in &should_load {
            if field_id != &field.id {
                continue;
            }

            // Check if already loaded
            let already_loaded = existing_fields.iter().any(|(_, f)| f.field_id == *field_id);
            if already_loaded {
                continue;
            }

            // Generate the field
            let params = build_field_params(
                field,
                field_transform,
                orbital_body,
                simulation_time,
                detail_level,
            );
            let mut asteroids = generate_field(&params);

            // Apply delta store modifications
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            field_id.hash(&mut hasher);
            let field_hash = hasher.finish();
            apply_deltas_to_field(
                &mut asteroids,
                &delta_store,
                StableBodyId {
                    kind: crate::belt_field_generation::BodyKind::Field,
                    id: field_hash,
                },
                simulation_time,
            );

            // Separate overlaps
            separate_overlaps(&mut asteroids);

            // Spawn the asteroids
            let field_gen_entity = commands
                .spawn((
                    Name::new(format!("GeneratedField: {field_id}")),
                    GeneratedField {
                        field_id: field_id.clone(),
                        generated_asteroids: asteroids.clone(),
                    },
                ))
                .id();

            // Spawn each asteroid as a child entity
            for asteroid in asteroids {
                spawn_generated_asteroid(&mut commands, &asset_server, asteroid, field_gen_entity);
            }

            // Track as loaded
            loaded_sectors.fields.push(field_id.clone());
        }
    }
}

/// Calculates the sector index for a belt based on player position.
///
/// Sector identity comes from orbital position, not a running counter.
fn calculate_sector_index(
    _orbital_parent: &str,
    orbital_distance: f32,
    sector_size: f32,
    player_pos: Vec3,
) -> i64 {
    // Calculate the angle around the orbital parent
    // For simplicity, assume parent is at origin (sun)
    // Angle 0 is at positive X axis, increasing towards positive Y
    let angle = player_pos.y.atan2(player_pos.x);
    let circumference = 2.0 * std::f32::consts::PI * orbital_distance;
    #[allow(clippy::cast_possible_truncation)]
    let num_sectors = (circumference / sector_size).ceil() as i64;
    // Map angle from [-PI, PI] to [0, 2*PI), then to [0, num_sectors)
    let angle_normalized = if angle < 0.0 {
        2.0f32.mul_add(std::f32::consts::PI, angle)
    } else {
        angle
    };
    let sector_index = {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let idx =
            (angle_normalized / (2.0 * std::f32::consts::PI) * num_sectors as f32).floor() as i64;
        idx
    };
    sector_index.clamp(0, num_sectors - 1)
}

/// Builds parameters for generating a belt sector.
fn build_belt_sector_params(
    belt: &AsteroidBelt,
    _belt_transform: &GlobalTransform,
    orbital_body: &OrbitalBody,
    sector_index: i64,
    simulation_time: f32,
    detail_level: f32,
) -> BeltSectorParams {
    // Convert excluded radii from the belt component (already in SI units)
    let excluded_radii: Vec<crate::belt_field_generation::ExcludedRadius> = belt
        .excluded_radii
        .iter()
        .map(|e| crate::belt_field_generation::ExcludedRadius {
            orbital_distance: e.orbital_distance,
            gap_radius: e.gap_radius,
        })
        .collect();

    // Convert size distribution from the belt component
    let size_distribution: Vec<SizeDistributionEntry> = belt
        .size_distribution
        .iter()
        .map(|s| SizeDistributionEntry {
            radius: s.radius,
            weight: s.weight,
        })
        .collect();

    BeltSectorParams {
        belt_id: belt.id.clone(),
        sector_index,
        template: belt.template.clone(),
        orbital_parent: orbital_body.orbital_parent.to_string(), // This is an Entity, need to resolve
        orbital_distance: orbital_body.orbital_distance,
        orbital_period: orbital_body.orbital_period,
        orbital_eccentricity: orbital_body.orbital_eccentricity,
        orbital_inclination: orbital_body.orbital_inclination,
        sector_size: belt.sector_size,
        radial_extent: belt.radial_extent,
        thickness: belt.thickness,
        mean_spacing: belt.mean_spacing,
        excluded_radii,
        size_distribution,
        density: belt.density,
        seed: belt.seed,
        simulation_time,
        detail_level,
    }
}

/// Builds parameters for generating a field.
fn build_field_params(
    field: &AsteroidField,
    _field_transform: &GlobalTransform,
    orbital_body: Option<&OrbitalBody>,
    simulation_time: f32,
    detail_level: f32,
) -> FieldParams {
    // Convert size distribution from the field component
    let size_distribution: Vec<SizeDistributionEntry> = field
        .size_distribution
        .iter()
        .map(|s| SizeDistributionEntry {
            radius: s.radius,
            weight: s.weight,
        })
        .collect();

    FieldParams {
        field_id: field.id.clone(),
        template: field.template.clone(),
        orbital_parent: orbital_body.map(|ob| ob.orbital_parent.to_string()),
        orbital_distance: orbital_body.map(|ob| ob.orbital_distance),
        orbital_period: orbital_body.map(|ob| ob.orbital_period),
        extent: field.extent,
        mean_spacing: field.mean_spacing,
        size_distribution,
        density: field.density,
        seed: field.seed,
        simulation_time,
        detail_level,
    }
}

/// Applies delta store modifications to a belt sector's asteroids.
fn apply_deltas_to_sector(
    asteroids: &mut Vec<GeneratedAsteroid>,
    delta_store: &AsteroidDeltaStore,
    parent_id: StableBodyId,
    simulation_time: f32,
) {
    let deltas = delta_store.get_deltas_for(parent_id.kind, parent_id.id);
    for delta in deltas {
        match delta {
            crate::belt_field_generation::AsteroidDelta::Destroyed { stable_id } => {
                asteroids.retain(|a| a.stable_id != stable_id.id);
            }
            crate::belt_field_generation::AsteroidDelta::Displaced {
                stable_id,
                position,
                velocity,
                timestamp,
            } => {
                if let Some(asteroid) = asteroids.iter_mut().find(|a| a.stable_id == stable_id.id)
                    && let Some(orbital_params) = asteroid.orbital_params.as_ref()
                {
                    let (new_pos, _new_vel) = propagate_displaced_asteroid(
                        *position,
                        *velocity,
                        *timestamp,
                        simulation_time,
                        orbital_params,
                    );
                    asteroid.position = new_pos;
                    // Would need to store velocity on the asteroid
                }
            }
            crate::belt_field_generation::AsteroidDelta::Added {
                stable_id: _stable_id,
                position: _position,
                velocity: _velocity,
                radius: _radius,
                mesh_name: _mesh_name,
                timestamp: _timestamp,
            } => {
                // Add new asteroid from delta
                // This would create a new GeneratedAsteroid from the delta data
            }
        }
    }
}

/// Applies delta store modifications to a field's asteroids.
fn apply_deltas_to_field(
    asteroids: &mut Vec<GeneratedAsteroid>,
    delta_store: &AsteroidDeltaStore,
    parent_id: StableBodyId,
    simulation_time: f32,
) {
    let deltas = delta_store.get_deltas_for(parent_id.kind, parent_id.id);
    for delta in deltas {
        match delta {
            crate::belt_field_generation::AsteroidDelta::Destroyed { stable_id } => {
                asteroids.retain(|a| a.stable_id != stable_id.id);
            }
            crate::belt_field_generation::AsteroidDelta::Displaced {
                stable_id,
                position,
                velocity,
                timestamp,
            } => {
                if let Some(asteroid) = asteroids.iter_mut().find(|a| a.stable_id == stable_id.id)
                    && let Some(orbital_params) = asteroid.orbital_params.as_ref()
                {
                    let (new_pos, _new_vel) = propagate_displaced_asteroid(
                        *position,
                        *velocity,
                        *timestamp,
                        simulation_time,
                        orbital_params,
                    );
                    asteroid.position = new_pos;
                }
            }
            crate::belt_field_generation::AsteroidDelta::Added {
                stable_id: _stable_id,
                position: _position,
                velocity: _velocity,
                radius: _radius,
                mesh_name: _mesh_name,
                timestamp: _timestamp,
            } => {
                // Add new asteroid from delta
            }
        }
    }
}

/// Computes orbital position for a body at a given time.
fn compute_orbital_position(orbital: &OrbitalBody, time: f32) -> Vec3 {
    // Simplified - would use proper Keplerian propagation
    let angle =
        orbital.initial_orbital_angle + 2.0 * std::f32::consts::PI * time / orbital.orbital_period;
    let x = orbital.orbital_distance * angle.cos();
    let y = orbital.orbital_distance * angle.sin();
    let z = 0.0;
    Vec3::new(x, y, z)
}

/// Spawns a generated asteroid as an entity.
fn spawn_generated_asteroid(
    commands: &mut Commands<'_, '_>,
    _asset_server: &Res<'_, AssetServer>,
    asteroid: GeneratedAsteroid,
    parent: Entity,
) {
    let entity_id = format!("asteroid_{}", asteroid.stable_id);

    let mut entity_commands = commands.spawn((
        Name::new(format!("Asteroid: {entity_id}")),
        Transform::from_translation(asteroid.position)
            .with_rotation(asteroid.rotation)
            .with_scale(Vec3::splat(asteroid.radius / 4.29633)), // Scale from template radius
        RigidBody::new(asteroid.mass, 1.0),
        CollisionShape(asteroid.collision_shape),
        CollisionLayersComponent::new(ASTEROID),
        Navigable,
        Targetable,
        WorldEntityId(entity_id),
        LazyLoadMesh {
            min_screen_radius_px: crate::celestial::ALWAYS_VISIBLE_SCREEN_RADIUS_PX,
            loaded: false,
            mesh_path: format!("templates/{}/mesh.glb", asteroid.mesh_name),
            current_screen_radius_px: 0.0,
            collision_relevance_px: COLLISION_RELEVANCE_PX,
            mesh_child_entities: Vec::new(),
        },
    ));

    // Add orbital parameters if present
    if let Some(orbital) = asteroid.orbital_params {
        entity_commands.insert(crate::celestial::OrbitalBody {
            orbital_parent: Entity::PLACEHOLDER,
            orbital_distance: orbital.orbital_distance,
            orbital_period: orbital.orbital_period,
            orbital_eccentricity: orbital.orbital_eccentricity,
            orbital_inclination: orbital.orbital_inclination,
            initial_orbital_angle: orbital.initial_orbital_angle,
        });
        entity_commands.insert(crate::OrbitalParentId(orbital.orbital_parent));
    }

    entity_commands.set_parent_in_place(parent);
}

/// Plugin for belt and field streaming.
pub struct BeltFieldStreamingPlugin;

impl Plugin for BeltFieldStreamingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadedSectors>()
            .init_resource::<AsteroidDeltaStore>()
            .add_systems(
                FixedUpdate,
                (belt_sector_streaming_system, field_streaming_system)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

#[cfg(test)]
#[path = "belt_field_streaming_tests.rs"]
mod tests;
