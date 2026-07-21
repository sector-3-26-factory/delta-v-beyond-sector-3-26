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

//! Celestial body components for suns and planets.
//!
//! These components are used by the orbital motion and sun rotation systems.
//! Per ADR-0009, celestial bodies are always gravity sources.

use bevy::gltf::Gltf;
use bevy::light::{NotShadowCaster, PointLight};
use bevy::prelude::*;
use delta_v_core::{
    DebugAxesEligible, EntityType, RenderLayer, SpawnEntity, Targetable, WorldEntityId,
};
use delta_v_types::{BoundingBoxJson, CollisionShapeJson, LightColorJson, Vec3Json};
use serde_json::Value;

use crate::{CollisionShape, MassSource, RigidBody};

/// Component for sun entities.
///
/// Suns are stars that provide light and gravity in the game world.
/// Per ADR-0009, suns are always gravity sources.
///
/// The `rotation_period` is in hours and controls the slow rotation animation.
#[derive(Component, Clone, Debug)]
pub struct Sun {
    /// Rotation period in hours. None means no rotation.
    pub rotation_period: Option<f32>,
}

/// Component for planet entities.
///
/// Planets orbit a parent body (sun or another planet) and are always
/// gravity sources per ADR-0009.
///
/// Orbital parameters are stored in SI units (metres, seconds, radians).
#[derive(Component, Clone, Debug)]
pub struct Planet {
    /// The parent entity this planet orbits.
    pub orbital_parent: Entity,
    /// Orbital distance in metres.
    pub orbital_distance: f32,
    /// Orbital period in seconds.
    pub orbital_period: f32,
    /// Orbital eccentricity (0 = circular, 0.1-0.9 = increasingly elliptical).
    pub orbital_eccentricity: f32,
    /// Orbital inclination in radians.
    pub orbital_inclination: f32,
    /// Initial orbital angle in radians.
    pub initial_orbital_angle: f32,
}

/// Marker component for entities that can be targeted or selected in navigation.
///
/// This includes ships, stations, planets, and other navigable objects.
/// Per ADR-0054, targeting is explicit player control only.
#[derive(Component, Clone, Copy, Debug)]
pub struct Navigable;

/// Pending mesh marker for celestial bodies.
///
/// Used to track when celestial body meshes are loaded and ready to attach.
/// Implements a local version of the `PendingMesh` pattern to avoid the
/// `delta-v-spawn` dependency cycle.
#[derive(Component, Clone, Debug)]
pub struct PendingCelestialMesh {
    /// Handle to the glTF asset being loaded.
    pub gltf_handle: Handle<Gltf>,
}

/// Component storing the parent entity ID string for resolution.
///
/// This is a temporary marker used during spawning to store the parent ID
/// before it can be resolved to an Entity reference.
/// Per ADR-0038, this is resolved in a second pass after all entities are spawned.
#[derive(Component, Clone, Debug)]
pub struct OrbitalParentId(pub String);

// ---------------------------------------------------------------------------
// Template extraction helpers (inlined from delta-v-spawn to avoid cycle)
// ---------------------------------------------------------------------------

/// Extracts a mass value from a validated template JSON value.
///
/// # Panics
///
/// Panics if `mass.value` is missing or not a valid number. This is safe because
/// the schema requires this field and it is validated by `delta-v-json` (ADR-0013).
#[allow(clippy::expect_used, clippy::cast_possible_truncation)]
#[must_use]
fn extract_mass(template: &Value) -> f32 {
    // INVARIANT: mass.value is required by schema and validated by delta-v-json (ADR-0013)
    template
        .get("mass")
        .and_then(Value::as_object)
        .and_then(|m| m.get("value"))
        .and_then(Value::as_f64)
        .expect("mass.value should be present and valid per schema") as f32
}

/// Extracts a `CollisionShapeJson` from a validated template JSON value.
///
/// # Panics
///
/// Panics if `collision_shape` is missing from the template. This is safe because
/// the schema requires this field and it is validated by `delta-v-json` (ADR-0013).
#[allow(clippy::expect_used)]
#[must_use]
fn extract_collision_shape(template: &Value) -> CollisionShapeJson {
    // INVARIANT: collision_shape is required by schema and validated by delta-v-json (ADR-0013)
    let shape = template
        .get("collision_shape")
        .expect("collision_shape should be present per schema");
    serde_json::from_value(shape.clone())
        .expect("collision_shape must be valid JSON (validated by delta-v-json)")
}

/// Extracts a `Vec3Json` from a JSON object with x, y, z fields.
///
/// # Panics
///
/// Panics if any of `x`, `y`, `z` fields are missing or not valid numbers.
/// This is safe because the schema requires these fields and they are validated
/// by `delta-v-json` (ADR-0013).
#[allow(clippy::expect_used, clippy::cast_possible_truncation)]
#[must_use]
fn extract_vec3(json: &serde_json::Map<String, Value>) -> Vec3Json {
    // INVARIANT: x, y, z are required by schema and validated by delta-v-json (ADR-0013)
    Vec3Json {
        x: json
            .get("x")
            .and_then(Value::as_f64)
            .expect("x should be present and valid") as f32,
        y: json
            .get("y")
            .and_then(Value::as_f64)
            .expect("y should be present and valid") as f32,
        z: json
            .get("z")
            .and_then(Value::as_f64)
            .expect("z should be present and valid") as f32,
    }
}

/// Extracts a `BoundingBoxJson` from a validated template JSON value.
///
/// # Panics
///
/// Panics if `bounding_box`, `min`, or `max` fields are missing or invalid.
/// This is safe because the schema requires these fields and they are validated
/// by `delta-v-json` (ADR-0013).
#[allow(clippy::expect_used)]
#[must_use]
fn extract_bounding_box(template: &Value) -> BoundingBoxJson {
    // INVARIANT: bounding_box.min and bounding_box.max are required by schema (ADR-0013)
    let bbox = template
        .get("bounding_box")
        .expect("bounding_box should be present per schema");

    let min = extract_vec3(
        bbox.get("min")
            .and_then(Value::as_object)
            .expect("min should be an object"),
    );
    let max = extract_vec3(
        bbox.get("max")
            .and_then(Value::as_object)
            .expect("max should be an object"),
    );

    BoundingBoxJson { min, max }
}

/// Computes debug axis length from a `BoundingBoxJson` (120% of longest side).
#[must_use]
fn compute_debug_axis_length(bbox: &BoundingBoxJson) -> f32 {
    let size = bbox.size();
    let max_dim = size.x.max(size.y).max(size.z);
    max_dim * 1.2
}

/// Scales a `BoundingBoxJson` by the given scale factor.
///
/// Both min and max corners are multiplied by the scale.
#[must_use]
fn scale_bounding_box(bbox: &BoundingBoxJson, scale: f32) -> BoundingBoxJson {
    BoundingBoxJson {
        min: Vec3Json {
            x: bbox.min.x * scale,
            y: bbox.min.y * scale,
            z: bbox.min.z * scale,
        },
        max: Vec3Json {
            x: bbox.max.x * scale,
            y: bbox.max.y * scale,
            z: bbox.max.z * scale,
        },
    }
}

/// Resolves the final mass value, using override if present.
///
/// Mass is NOT scaled - it is used as-is from the template, or overridden if
/// `mass_override` is specified.
#[must_use]
fn resolve_mass(template_mass: f32, mass_override: Option<f32>) -> f32 {
    mass_override.unwrap_or(template_mass)
}

/// Extracts a light intensity value from a validated template JSON value.
///
/// # Panics
///
/// Panics if `light_intensity` is missing or not a valid number.
/// This is safe because the schema provides a default and validates the field.
#[allow(clippy::expect_used, clippy::cast_possible_truncation)]
#[must_use]
fn extract_light_intensity(template: &Value) -> f32 {
    // INVARIANT: light_intensity has a default in schema and is validated by delta-v-json
    template
        .get("light_intensity")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .expect("light_intensity should be present (default from schema)")
}

/// Extracts a light color from a validated template JSON value.
///
/// # Panics
///
/// Panics if `light_color` is missing or missing required fields.
/// This is safe because the schema provides a default and validates the field.
#[allow(clippy::expect_used, clippy::cast_possible_truncation)]
#[must_use]
fn extract_light_color(template: &Value) -> LightColorJson {
    // INVARIANT: light_color has a default in schema and is validated by delta-v-json
    let color = template
        .get("light_color")
        .and_then(Value::as_object)
        .expect("light_color should be present (default from schema)");
    LightColorJson {
        r: color
            .get("r")
            .and_then(Value::as_f64)
            .map(|v| v as f32)
            .expect("light_color.r should be present (default from schema)"),
        g: color
            .get("g")
            .and_then(Value::as_f64)
            .map(|v| v as f32)
            .expect("light_color.g should be present (default from schema)"),
        b: color
            .get("b")
            .and_then(Value::as_f64)
            .map(|v| v as f32)
            .expect("light_color.b should be present (default from schema)"),
    }
}

/// Extracts a light range value from a validated template JSON value.
///
/// # Panics
///
/// Panics if `light_range` is missing or missing the value field.
/// This is safe because the schema provides a default and validates the field.
#[allow(clippy::expect_used, clippy::cast_possible_truncation)]
#[must_use]
fn extract_light_range(template: &Value) -> f32 {
    // INVARIANT: light_range has a default in schema and is validated by delta-v-json
    template
        .get("light_range")
        .and_then(Value::as_object)
        .and_then(|r| r.get("value"))
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .expect("light_range should be present (default from schema)")
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
#[allow(clippy::needless_pass_by_value, clippy::cast_possible_truncation)]
pub fn spawn_sun(
    mut events: MessageReader<'_, '_, SpawnEntity>,
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
) {
    for spawn in events.read() {
        if spawn.entity_type != "sun" {
            continue;
        }

        let template = &spawn.template;

        // Extract mass (required)
        let template_mass = extract_mass(template);
        // Resolve mass: use override if present, otherwise use template mass.
        // Mass is NOT scaled - it is used as-is or overridden.
        let mass = resolve_mass(template_mass, spawn.mass);

        // Extract collision shape (required)
        let collision_shape = extract_collision_shape(template);

        // Extract bounding box (required) for debug axes computation
        let bbox = extract_bounding_box(template);
        // Use the maximum scale component for uniform scaling
        let scale_factor = spawn.scale.x.max(spawn.scale.y).max(spawn.scale.z);
        // Scale the bounding box for debug axis computation
        let scaled_bbox = scale_bounding_box(&bbox, scale_factor);
        let axis_length = compute_debug_axis_length(&scaled_bbox);

        // Extract rotation period (optional)
        let rotation_period = template
            .get("rotation_period")
            .and_then(|r| r.get("value"))
            .and_then(Value::as_f64)
            .map(|v| v as f32);

        // Extract light properties (all optional, with defaults)
        let light_intensity = extract_light_intensity(template);
        let light_color = extract_light_color(template);
        let light_range = extract_light_range(template);

        let entity_id = spawn.id.clone();

        // Queue glTF mesh load
        let gltf_handle = asset_server.load::<Gltf>(&spawn.mesh_template_path);

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
                Sun { rotation_period },
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
        if let Some(radius) = collision_shape.radius {
            let scaled_radius = radius.value * scale_factor;
            commands
                .entity(sun_entity)
                .insert(CollisionShape::sphere(scaled_radius, Vec3::ZERO));
        }

        // Spawn a child entity with PointLight
        // The light follows the sun's transform automatically via ChildOf
        commands.spawn((
            Name::new(format!("Sun Light: {entity_id}")),
            PointLight {
                color: Color::srgb(light_color.r, light_color.g, light_color.b),
                intensity: light_intensity,
                range: light_range,
                ..default()
            },
            RenderLayer::Gameplay.render_layers(),
            ChildOf(sun_entity),
        ));

        tracing::info!(
            "spawned sun: {entity_id} (mass={mass:.3e} kg, light={light_intensity} lux, range={light_range:.3e} m)"
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
    clippy::expect_used,
    clippy::cast_possible_truncation
)]
pub fn spawn_planet(
    mut events: MessageReader<'_, '_, SpawnEntity>,
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
) {
    for spawn in events.read() {
        if spawn.entity_type != "planet" {
            continue;
        }

        let template = &spawn.template;

        // Extract mass (required)
        let template_mass = extract_mass(template);
        // Resolve mass: use override if present, otherwise use template mass.
        // Mass is NOT scaled - it is used as-is or overridden.
        let mass = resolve_mass(template_mass, spawn.mass);

        // Extract collision shape (required)
        let collision_shape = extract_collision_shape(template);

        // Extract bounding box (required) for debug axes computation
        let bbox = extract_bounding_box(template);
        // Use the maximum scale component for uniform scaling
        let scale_factor = spawn.scale.x.max(spawn.scale.y).max(spawn.scale.z);
        // Scale the bounding box for debug axis computation
        let scaled_bbox = scale_bounding_box(&bbox, scale_factor);
        let axis_length = compute_debug_axis_length(&scaled_bbox);

        // Extract orbital parameters (required)
        let orbital_parent_id = template
            .get("orbital_parent")
            .and_then(Value::as_str)
            .expect("orbital_parent is required for planets")
            .to_string();

        let orbital_distance = template
            .get("orbital_distance")
            .and_then(|d| d.get("value"))
            .and_then(Value::as_f64)
            .expect("orbital_distance is required for planets")
            as f32;

        let orbital_period = template
            .get("orbital_period")
            .and_then(|p| p.get("value"))
            .and_then(Value::as_f64)
            .expect("orbital_period is required for planets") as f32;

        // Extract optional orbital parameters (defaults filled by delta-v-json)
        let orbital_eccentricity = template
            .get("orbital_eccentricity")
            .and_then(Value::as_f64)
            .map_or(0.0, |v| v as f32);

        let orbital_inclination = template
            .get("orbital_inclination")
            .and_then(|i| i.get("value"))
            .and_then(Value::as_f64)
            .map_or(0.0, |v| v as f32);

        let initial_orbital_angle = template
            .get("initial_orbital_angle")
            .and_then(|a| a.get("value"))
            .and_then(Value::as_f64)
            .map_or(0.0, |v| v as f32);

        let entity_id = spawn.id.clone();

        // Queue glTF mesh load
        let gltf_handle = asset_server.load::<Gltf>(&spawn.mesh_template_path);

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
                orbital_distance,
                orbital_period,
                orbital_eccentricity,
                orbital_inclination,
                initial_orbital_angle,
            },
            OrbitalParentId(orbital_parent_id),
            Navigable,
            Targetable,
            EntityType("planet".to_string()),
            WorldEntityId(entity_id.clone()),
            PendingCelestialMesh { gltf_handle },
            DebugAxesEligible::new(entity_id.clone(), axis_length),
        ));

        // Add collision shape with scaling
        if let Some(radius) = collision_shape.radius {
            let scaled_radius = radius.value * scale_factor;
            entity_commands.insert(CollisionShape::sphere(scaled_radius, Vec3::ZERO));
        }

        tracing::info!(
            "spawned planet: {entity_id} (mass={mass:.3e} kg, distance={orbital_distance:.3e} m, period={orbital_period:.3e} s)"
        );
    }
}

/// Resolves orbital parent IDs to Entity references.
///
/// This system runs after all entities are spawned to resolve the `OrbitalParentId`
/// component to an actual `Entity` reference.
///
/// Per ADR-0017, this system runs in `FixedUpdate` at 60 Hz.
#[allow(clippy::needless_pass_by_value, clippy::explicit_iter_loop)]
pub fn resolve_orbital_parents(
    mut commands: Commands<'_, '_>,
    mut planets: Query<'_, '_, (Entity, &OrbitalParentId, &mut Planet)>,
    all_entities: Query<'_, '_, (Entity, &Name)>,
) {
    for (entity, parent_id, mut planet) in &mut planets {
        // Find the parent entity by name
        for (potential_parent, name) in &all_entities {
            if name.as_str() == parent_id.0 {
                planet.orbital_parent = potential_parent;
                commands.entity(entity).remove::<OrbitalParentId>();
                tracing::debug!(
                    "Resolved orbital parent for {entity:?}: {} -> {potential_parent:?}",
                    parent_id.0
                );
                break;
            }
        }
    }
}

/// Attaches loaded glTF scenes to celestial body entities.
///
/// This system runs in `Update` during `AppState::InGame` to attach meshes
/// once the glTF assets are loaded.
#[allow(clippy::needless_pass_by_value)]
pub fn attach_celestial_meshes(
    mut commands: Commands<'_, '_>,
    gltf_assets: Res<'_, Assets<Gltf>>,
    query: Query<'_, '_, (Entity, &PendingCelestialMesh)>,
) {
    for (entity, pending) in &query {
        if let Some(gltf) = gltf_assets.get(&pending.gltf_handle) {
            if gltf.scenes.is_empty() {
                continue;
            }
            for scene_handle in &gltf.scenes {
                let child = commands.spawn(SceneRoot(scene_handle.clone())).id();
                commands.entity(entity).add_child(child);
            }
            commands.entity(entity).remove::<PendingCelestialMesh>();
        }
    }
}

/// Makes sun meshes emissive after they are loaded.
///
/// Per ADR-0053, this is a VFX exemption - the sun's mesh should glow
/// to appear as a light source. This system runs after `attach_celestial_meshes`
/// to modify the material of the sun's mesh to be emissive.
///
/// The glTF scene structure is: Sun -> SceneRoot -> Mesh (with material).
/// We need to recursively check all descendants to find meshes with materials.
#[allow(clippy::needless_pass_by_value)]
pub fn make_sun_emissive(
    suns: Query<'_, '_, (Entity, &Children, &Sun)>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
    mesh_query: Query<'_, '_, &MeshMaterial3d<StandardMaterial>>,
    children_query: Query<'_, '_, &Children>,
) {
    for (sun_entity, children, _sun) in &suns {
        // Recursively check all descendants for meshes with materials
        let mut stack: Vec<Entity> = Vec::new();
        for child in children.iter() {
            stack.push(child);
        }
        while let Some(entity) = stack.pop() {
            if let Ok(mat_handle) = mesh_query.get(entity) {
                if let Some(material) = materials.get_mut(&mat_handle.0) {
                    // Make the material emissive with a bright yellow-white color
                    // matching the light color (warm white: 1.0, 0.95, 0.8)
                    material.emissive = LinearRgba::new(1.0, 0.95, 0.8, 1.0);
                    tracing::debug!(
                        "Made sun descendant {entity:?} emissive (sun: {sun_entity:?})"
                    );
                }
            }
            // Add children to stack for depth-first traversal
            if let Ok(entity_children) = children_query.get(entity) {
                for child in entity_children.iter() {
                    stack.push(child);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "celestial_tests.rs"]
mod tests;
