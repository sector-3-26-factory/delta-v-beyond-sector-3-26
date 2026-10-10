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

//! Asteroid belt and field spawning systems.
//!
//! Per ADR-0047 (centralized spawning), this module contains the spawn systems
//! for asteroid belts and fields. Each system is named `spawn_<entity_type>` and
//! listens for `SpawnEntity` events.
//!
//! See ADR-0038 (entity template system), ADR-0055 (Keplerian orbits + SOI gravity),
//! and the asteroid belt plan (plans/asteroid-belt-plan.md Phase 7).

use bevy::prelude::*;
use delta_v_core::{EntityType, SpawnEntity, WorldEntityId};
use delta_v_types::{AsteroidBeltTemplate, AsteroidFieldTemplate};

use crate::OrbitalParentId;
use crate::celestial::OrbitalBody;

// ---------------------------------------------------------------------------
// Components for belt and field entities
// ---------------------------------------------------------------------------

/// Component for asteroid belt entities.
///
/// An asteroid belt is a procedural region that generates asteroids along an orbital ring.
/// The belt itself is a single entity; the asteroids it contains are generated on demand.
/// Orbital parameters are stored in the `OrbitalBody` component (from world definition).
/// Belt-specific parameters are stored in this component (from world definition).
#[derive(Component, Clone, Debug)]
pub struct AsteroidBelt {
    /// The belt template with generation parameters (meshes, density).
    pub template: AsteroidBeltTemplate,
    /// Unique identifier for this belt (from world definition).
    pub id: String,
    /// Radial extent of the belt in metres.
    pub radial_extent: f32,
    /// Vertical thickness of the belt in metres.
    pub thickness: f32,
    /// Mean spacing between asteroids in metres.
    pub mean_spacing: f32,
    /// Excluded radii (Kirkwood gaps) in the belt.
    pub excluded_radii: Vec<crate::belt_field_generation::ExcludedRadius>,
    /// Random seed for deterministic generation.
    pub seed: u64,
    /// Angular size of each sector in radians.
    pub sector_size: f32,
    /// Size distribution of asteroids in the belt.
    pub size_distribution: Vec<crate::belt_field_generation::SizeDistributionEntry>,
    /// Material density of asteroids in kg/m³.
    pub density: f32,
}

/// Component for asteroid field entities.
///
/// An asteroid field is a procedural region that generates asteroids in a defined volume.
/// The field itself is a single entity; the asteroids it contains are generated on demand.
/// Orbital parameters are stored in the `OrbitalBody` component (from world definition).
/// Field-specific parameters are stored in this component (from world definition).
#[derive(Component, Clone, Debug)]
pub struct AsteroidField {
    /// The field template with generation parameters (meshes, density).
    pub template: AsteroidFieldTemplate,
    /// Unique identifier for this field (from world definition).
    pub id: String,
    /// Spatial extent of the field in metres (cube side length for box shape).
    pub extent: f32,
    /// Shape of the field (e.g., "box", "sphere").
    pub shape: String,
    /// Mean spacing between asteroids in metres.
    pub mean_spacing: f32,
    /// Random seed for deterministic generation.
    pub seed: u64,
    /// Optional sector size for streaming (None = no streaming).
    pub sector_size: Option<f32>,
    /// Size distribution of asteroids in the field.
    pub size_distribution: Vec<crate::belt_field_generation::SizeDistributionEntry>,
    /// Material density of asteroids in kg/m³.
    pub density: f32,
}

// ---------------------------------------------------------------------------
// Spawn systems
// ---------------------------------------------------------------------------

/// Spawns an asteroid belt entity from a `SpawnEntity` event.
///
/// Per ADR-0017, this system runs in `Update` during `AppState::SpawningEntities`.
///
/// # Panics
///
/// Panics if the template is missing required fields. This is intentional per
/// ADR-0013 (no silent fallbacks).
#[allow(clippy::needless_pass_by_value, clippy::expect_used)]
pub fn spawn_asteroid_belt(
    mut events: MessageReader<'_, '_, SpawnEntity>,
    mut commands: Commands<'_, '_>,
    _asset_server: Res<'_, AssetServer>,
) {
    for spawn in events.read() {
        // Use the template from the event (already loaded by WorldPlugin)
        let delta_v_types::EntityTemplate::AsteroidBelt(belt_template) = &spawn.template else {
            continue;
        };

        // Read orbital parameters from world definition
        let (
            orbital_parent,
            orbital_distance,
            orbital_period,
            orbital_eccentricity,
            orbital_inclination,
        ) = spawn.orbital_parameters.as_ref().map_or_else(
            || (String::new(), 0.0, 0.0, 0.0, 0.0),
            |op| {
                (
                    op.orbital_parent.clone(),
                    op.orbital_distance,
                    op.orbital_period,
                    op.orbital_eccentricity,
                    op.orbital_inclination,
                )
            },
        );

        // Read belt-specific parameters from world definition
        let belt_params = spawn
            .asteroid_belt_parameters
            .as_ref()
            .expect("asteroid belt entity must have asteroid_belt_parameters");

        let entity_id = spawn.id.clone();

        // Convert excluded radii
        let excluded_radii: Vec<crate::belt_field_generation::ExcludedRadius> = belt_params
            .excluded_radii
            .iter()
            .map(|er| crate::belt_field_generation::ExcludedRadius {
                orbital_distance: er.orbital_distance,
                gap_radius: er.gap_radius,
            })
            .collect();

        // Convert size distribution
        let size_distribution: Vec<crate::belt_field_generation::SizeDistributionEntry> =
            belt_params
                .size_distribution
                .iter()
                .map(|se| crate::belt_field_generation::SizeDistributionEntry {
                    radius: se.radius,
                    weight: se.weight,
                })
                .collect();

        // Spawn the belt entity (the belt itself is a definition, not a physical body)
        let belt_entity = commands
            .spawn((
                Name::new(format!("AsteroidBelt: {entity_id}")),
                Transform::from_translation(spawn.position)
                    .with_rotation(spawn.rotation)
                    .with_scale(spawn.scale),
                AsteroidBelt {
                    template: belt_template.clone(),
                    id: entity_id.clone(),
                    radial_extent: belt_params.radial_extent,
                    thickness: belt_params.thickness,
                    mean_spacing: belt_params.mean_spacing,
                    excluded_radii,
                    seed: belt_params.seed,
                    sector_size: belt_params.sector_size,
                    size_distribution,
                    density: belt_params.density,
                },
                EntityType("asteroid_belt".to_string()),
                WorldEntityId(entity_id.clone()),
            ))
            .id();

        // Add OrbitalBody component if orbital parameters are present
        if !orbital_parent.is_empty() {
            commands.entity(belt_entity).insert(OrbitalBody {
                orbital_parent: Entity::PLACEHOLDER,
                orbital_distance,
                orbital_period,
                orbital_eccentricity,
                orbital_inclination,
                initial_orbital_angle: 0.0, // Belts don't have a single initial angle
            });
            commands
                .entity(belt_entity)
                .insert(OrbitalParentId(orbital_parent));
        }

        tracing::info!(
            "spawned asteroid belt: {entity_id} (meshes={}, density={:.3e} kg/m³)",
            belt_template.meshes.len(),
            belt_template.density
        );
    }
}

/// Spawns an asteroid field entity from a `SpawnEntity` event.
///
/// Per ADR-0017, this system runs in `Update` during `AppState::SpawningEntities`.
///
/// # Panics
///
/// Panics if the template is missing required fields. This is intentional per
/// ADR-0013 (no silent fallbacks).
#[allow(clippy::needless_pass_by_value, clippy::expect_used)]
pub fn spawn_asteroid_field(
    mut events: MessageReader<'_, '_, SpawnEntity>,
    mut commands: Commands<'_, '_>,
    _asset_server: Res<'_, AssetServer>,
) {
    for spawn in events.read() {
        // Use the template from the event (already loaded by WorldPlugin)
        let delta_v_types::EntityTemplate::AsteroidField(field_template) = &spawn.template else {
            continue;
        };

        // Read orbital parameters from world definition
        let (orbital_parent, orbital_distance, orbital_period) =
            spawn.orbital_parameters.as_ref().map_or_else(
                || (None, None, None),
                |op| {
                    (
                        Some(op.orbital_parent.clone()),
                        Some(op.orbital_distance),
                        Some(op.orbital_period),
                    )
                },
            );

        // Read field-specific parameters from world definition
        let field_params = spawn
            .asteroid_field_parameters
            .as_ref()
            .expect("asteroid field entity must have asteroid_field_parameters");

        let entity_id = spawn.id.clone();

        // Convert size distribution
        let size_distribution: Vec<crate::belt_field_generation::SizeDistributionEntry> =
            field_params
                .size_distribution
                .iter()
                .map(|se| crate::belt_field_generation::SizeDistributionEntry {
                    radius: se.radius,
                    weight: se.weight,
                })
                .collect();

        // Spawn the field entity (the field itself is a definition, not a physical body)
        let field_entity = commands
            .spawn((
                Name::new(format!("AsteroidField: {entity_id}")),
                Transform::from_translation(spawn.position)
                    .with_rotation(spawn.rotation)
                    .with_scale(spawn.scale),
                AsteroidField {
                    template: field_template.clone(),
                    id: entity_id.clone(),
                    extent: field_params.extent,
                    shape: field_params.shape.clone(),
                    mean_spacing: field_params.mean_spacing,
                    seed: field_params.seed,
                    sector_size: field_params.sector_size,
                    size_distribution,
                    density: field_params.density,
                },
                EntityType("asteroid_field".to_string()),
                WorldEntityId(entity_id.clone()),
            ))
            .id();

        // Add OrbitalBody component if orbital parameters are present
        if let (Some(parent), Some(distance), Some(period)) =
            (orbital_parent, orbital_distance, orbital_period)
        {
            commands.entity(field_entity).insert(OrbitalBody {
                orbital_parent: Entity::PLACEHOLDER,
                orbital_distance: distance,
                orbital_period: period,
                orbital_eccentricity: 0.0,
                orbital_inclination: 0.0,
                initial_orbital_angle: 0.0,
            });
            commands
                .entity(field_entity)
                .insert(OrbitalParentId(parent));
        }

        tracing::info!(
            "spawned asteroid field: {entity_id} (meshes={}, density={:.3e} kg/m³)",
            field_template.meshes.len(),
            field_template.density
        );
    }
}

// Tests will be added in a separate file
