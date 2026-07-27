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

//! Celestial body spawning systems.
//!
//! Per ADR-0047 (centralized spawning), this module contains the spawn systems
//! for celestial bodies (suns, planets, asteroids). Each system is named
//! `spawn_<entity_type>` and listens for `SpawnEntity` events.
//!
//! See ADR-0038 (entity template system) and ADR-0009 (Newtonian physics).

use bevy::gltf::Gltf;
use bevy::light::{NotShadowCaster, PointLight};
use bevy::prelude::*;
use delta_v_core::{
    DebugAxesEligible, EntityType, RenderLayer, SpawnEntity, Targetable, WorldEntityId,
};
use delta_v_types::{BoundingBox, CollisionShapeData, CollisionShapeType};

use crate::{
    CollisionLayersComponent, CollisionShape, MassSource, Navigable, OrbitalParentId,
    PendingCelestialMesh, Planet, RigidBody, Sun,
};

/// Scales a `CollisionShapeData` by the given scale factor.
///
/// Returns a new `CollisionShapeData` with scaled dimensions.
#[must_use]
fn scale_collision_shape(shape: &CollisionShapeData, scale: f32) -> CollisionShapeData {
    match shape.shape_type {
        CollisionShapeType::Sphere { radius } => {
            CollisionShapeData::sphere(radius * scale, shape.offset * scale)
        }
        CollisionShapeType::Box { half_extents } => CollisionShapeData::box_shape(
            Vec3::new(
                half_extents.x * scale,
                half_extents.y * scale,
                half_extents.z * scale,
            ),
            shape.offset * scale,
        ),
        CollisionShapeType::ConvexHull => {
            // ConvexHull is not yet implemented for scaling
            // Per ADR-0052, this is a known limitation
            *shape
        }
    }
}

/// Computes debug axis length from a `BoundingBox` (120% of longest side).
#[must_use]
fn compute_debug_axis_length(bbox: &BoundingBox) -> f32 {
    let size = bbox.size();
    let max_dim = size.x.max(size.y).max(size.z);
    max_dim * 1.2
}

/// Scales a `BoundingBox` by the given scale factor.
///
/// Both min and max corners are multiplied by the scale.
#[must_use]
fn scale_bounding_box(bbox: &BoundingBox, scale: f32) -> BoundingBox {
    BoundingBox {
        min: bbox.min * scale,
        max: bbox.max * scale,
    }
}

/// Resolves the final mass value, using override if present.
///
/// Mass is NOT scaled - it is used as-is or overridden.
#[must_use]
fn resolve_mass(template_mass: f32, mass_override: Option<f32>) -> f32 {
    mass_override.unwrap_or(template_mass)
}

// ---------------------------------------------------------------------------
// Spawn systems
// ---------------------------------------------------------------------------

/// Spawns a sun entity from a `SpawnEntity` event.
///
/// Per ADR-0017, this system runs in `FixedUpdate` at 60 Hz.
///
/// # Panics
///
/// Panics if the template is missing required fields. This is intentional per
/// ADR-0013 (no silent fallbacks).
#[allow(
    clippy::needless_pass_by_value,
    clippy::cast_possible_truncation,
    clippy::expect_used
)]
pub fn spawn_sun(
    mut events: MessageReader<'_, '_, SpawnEntity>,
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
) {
    for spawn in events.read() {
        // Use the template from the event (already loaded by WorldPlugin)
        let delta_v_types::EntityTemplate::Sun(sun_template) = &spawn.template else {
            continue;
        };

        // Resolve mass: use override if present, otherwise use template mass.
        // Mass is NOT scaled - it is used as-is or overridden.
        let mass = resolve_mass(sun_template.mass, spawn.mass);

        // Extract collision shape data
        let collision_shape_data = sun_template.collision_shape;

        // Extract bounding box for debug axes computation
        let bbox = &sun_template.bounding_box;
        // Use the maximum scale component for uniform scaling
        let scale_factor = spawn.scale.x.max(spawn.scale.y).max(spawn.scale.z);
        // Scale the bounding box for debug axis computation
        let scaled_bbox = scale_bounding_box(bbox, scale_factor);
        let axis_length = compute_debug_axis_length(&scaled_bbox);

        let entity_id = spawn.id.clone();

        // Queue glTF mesh load
        let gltf_handle = asset_server.load::<Gltf>(&spawn.mesh_path());

        // Spawn the sun entity
        // Per ADR-0053, NotShadowCaster is a VFX exemption - prevents the sun mesh
        // from blocking its own light.
        let sun_entity = commands
            .spawn((
                Name::new(format!("Sun: {entity_id}")),
                Transform::from_translation(spawn.position)
                    .with_rotation(spawn.rotation)
                    .with_scale(spawn.scale),
                RigidBody::new(mass, 1.0),
                MassSource,
                Sun {
                    rotation_period: sun_template.rotation_period,
                },
                Navigable,
                Targetable,
                EntityType("sun".to_string()),
                WorldEntityId(entity_id.clone()),
                PendingCelestialMesh { gltf_handle },
                DebugAxesEligible::new(entity_id.clone(), axis_length),
                NotShadowCaster,
            ))
            .id();

        // Add collision shape with scaling
        if let CollisionShapeType::Sphere { radius } = collision_shape_data.shape_type {
            let scaled_radius = radius * scale_factor;
            commands
                .entity(sun_entity)
                .insert(CollisionShape::sphere(scaled_radius, Vec3::ZERO));
        }

        // Spawn a child entity with PointLight
        // The light follows the sun's transform automatically via ChildOf
        commands.spawn((
            Name::new(format!("Sun Light: {entity_id}")),
            PointLight {
                color: Color::srgb(
                    sun_template.light_color.r,
                    sun_template.light_color.g,
                    sun_template.light_color.b,
                ),
                intensity: sun_template.light_intensity,
                range: sun_template.light_range,
                ..default()
            },
            RenderLayer::Gameplay.render_layers(),
            ChildOf(sun_entity),
        ));

        tracing::info!(
            "spawned sun: {entity_id} (mass={mass:.3e} kg, light={} lux, range={:.3e} m)",
            sun_template.light_intensity,
            sun_template.light_range
        );
    }
}

/// Spawns a planet entity from a `SpawnEntity` event.
///
/// Per ADR-0017, this system runs in `FixedUpdate` at 60 Hz.
///
/// # Panics
///
/// Panics if the template is missing required fields. This is intentional per
/// ADR-0013 (no silent fallbacks).
#[allow(
    clippy::needless_pass_by_value,
    clippy::cast_possible_truncation,
    clippy::expect_used
)]
pub fn spawn_planet(
    mut events: MessageReader<'_, '_, SpawnEntity>,
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
) {
    for spawn in events.read() {
        // Use the template from the event (already loaded by WorldPlugin)
        let delta_v_types::EntityTemplate::Planet(planet_template) = &spawn.template else {
            continue;
        };

        // Resolve mass: use override if present, otherwise use template mass.
        // Mass is NOT scaled - it is used as-is or overridden.
        let mass = resolve_mass(planet_template.mass, spawn.mass);

        // Extract collision shape data
        let collision_shape_data = planet_template.collision_shape;

        // Extract bounding box for debug axes computation
        let bbox = &planet_template.bounding_box;
        // Use the maximum scale component for uniform scaling
        let scale_factor = spawn.scale.x.max(spawn.scale.y).max(spawn.scale.z);
        // Scale the bounding box for debug axis computation
        let scaled_bbox = scale_bounding_box(bbox, scale_factor);
        let axis_length = compute_debug_axis_length(&scaled_bbox);

        let entity_id = spawn.id.clone();

        // Queue glTF mesh load
        let gltf_handle = asset_server.load::<Gltf>(&spawn.mesh_path());

        // We need to resolve the orbital_parent_id to an Entity.
        // This is done in a second pass after all entities are spawned.
        // For now, we store the parent ID as a placeholder.
        let mut entity_commands = commands.spawn((
            Name::new(format!("Planet: {entity_id}")),
            Transform::from_translation(spawn.position)
                .with_rotation(spawn.rotation)
                .with_scale(spawn.scale),
            RigidBody::new(mass, 1.0),
            MassSource,
            Planet {
                orbital_parent: Entity::PLACEHOLDER,
                orbital_distance: planet_template.orbital_distance,
                orbital_period: planet_template.orbital_period,
                orbital_eccentricity: planet_template.orbital_eccentricity,
                orbital_inclination: planet_template.orbital_inclination,
                initial_orbital_angle: planet_template.initial_orbital_angle,
            },
            OrbitalParentId(planet_template.orbital_parent.clone()),
            Navigable,
            Targetable,
            EntityType("planet".to_string()),
            WorldEntityId(entity_id.clone()),
            PendingCelestialMesh { gltf_handle },
            DebugAxesEligible::new(entity_id.clone(), axis_length),
        ));

        // Add collision shape with scaling
        if let CollisionShapeType::Sphere { radius } = collision_shape_data.shape_type {
            let scaled_radius = radius * scale_factor;
            entity_commands.insert(CollisionShape::sphere(scaled_radius, Vec3::ZERO));
        }

        tracing::info!(
            "spawned planet: {entity_id} (mass={mass:.3e} kg, distance={orbital_distance:.3e} m, period={orbital_period:.3e} s)",
            orbital_distance = planet_template.orbital_distance,
            orbital_period = planet_template.orbital_period
        );
    }
}

/// Spawns an asteroid entity from a `SpawnEntity` event.
///
/// Per ADR-0017, this system runs in `FixedUpdate` at 60 Hz.
///
/// # Panics
///
/// Panics if the template is missing required fields. This is intentional per
/// ADR-0013 (no silent fallbacks).
#[allow(
    clippy::needless_pass_by_value,
    clippy::cast_possible_truncation,
    clippy::expect_used
)]
pub fn spawn_asteroid(
    mut events: MessageReader<'_, '_, SpawnEntity>,
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
) {
    for spawn in events.read() {
        // Use the template from the event (already loaded by WorldPlugin)
        let delta_v_types::EntityTemplate::Asteroid(asteroid_template) = &spawn.template else {
            continue;
        };

        // Resolve mass: use override if present, otherwise use template mass.
        // Mass is NOT scaled - it is used as-is or overridden.
        let mass = resolve_mass(asteroid_template.mass, spawn.mass);

        // Extract collision shape data
        let collision_shape_data = asteroid_template.collision_shape;

        // Extract bounding box for debug axes computation
        let bbox = &asteroid_template.bounding_box;
        // Use the maximum scale component for uniform scaling
        let scale_factor = spawn.scale.x.max(spawn.scale.y).max(spawn.scale.z);
        // Scale the bounding box for debug axis computation
        let scaled_bbox = scale_bounding_box(bbox, scale_factor);
        let axis_length = compute_debug_axis_length(&scaled_bbox);

        let entity_id = spawn.id.clone();

        // Queue glTF mesh load
        let gltf_handle = asset_server.load::<Gltf>(&spawn.mesh_path());

        // Spawn the asteroid entity
        // Asteroids are dynamic bodies that respond to collisions based on their mass.
        // They use the ASTEROID collision layer.
        commands.spawn((
            Name::new(format!("Asteroid: {entity_id}")),
            Transform::from_translation(spawn.position)
                .with_rotation(spawn.rotation)
                .with_scale(spawn.scale),
            RigidBody::new(mass, 1.0), // inertia_scale = 1.0 for sphere
            CollisionShape(scale_collision_shape(&collision_shape_data, scale_factor)),
            CollisionLayersComponent::new(delta_v_types::collision::layers::ASTEROID),
            Navigable,
            Targetable,
            EntityType("asteroid".to_string()),
            WorldEntityId(entity_id.clone()),
            PendingCelestialMesh { gltf_handle },
            DebugAxesEligible::new(entity_id.clone(), axis_length),
        ));

        tracing::info!("spawned asteroid: {entity_id} (mass={mass:.3e} kg)",);
    }
}
