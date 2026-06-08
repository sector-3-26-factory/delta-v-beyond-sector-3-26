// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Asteroid spawning system.
//!
//! Listens for [`SpawnEntity`] events with `entity_type: "asteroid"` and spawns
//! static rigid body asteroids with collision shapes.

use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_core::{DebugAxesEligible, SpawnEntity};
use delta_v_physics::{CollisionShape, RigidBody, StaticBody};
use serde_json::Value;

/// Marker component for a pending asteroid mesh waiting for its glTF to load.
#[derive(Component)]
pub(crate) struct PendingAsteroidMesh {
    /// Handle to the glTF asset being loaded.
    gltf_handle: Handle<Gltf>,
}

/// Spawns an asteroid entity from a template.
///
/// This system runs during [`AppState::SpawningEntities`] and listens for
/// `SpawnEntity` events with `entity_type: "asteroid"`.
///
/// The asteroid mesh is loaded from a glTF file (`.glb`). The entity is spawned
/// immediately with a [`PendingAsteroidMesh`] marker, and the mesh is attached
/// asynchronously once the glTF asset is loaded.
///
/// # Panics
///
/// Panics if the asteroid template is missing required fields (mass, `collision_shape`).
/// This is intentional per ADR-0013 (no silent fallbacks).
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_asteroid_system(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    mut reader: EventReader<'_, '_, SpawnEntity>,
) {
    for event in reader.read() {
        if event.entity_type != "asteroid" {
            continue;
        }

        // Parse the template JSON to extract asteroid-specific data
        let template = &event.template;

        // Extract mass (required)
        // INVARIANT: asteroid template must have mass.value (ADR-0013)
        #[allow(clippy::expect_used)]
        let mass = get_mass_from_template(template);

        // Extract collision shape (required)
        let collision_shape = get_collision_shape_from_template(template);

        // Extract bounding box (required) for debug axes computation.
        // Debug axes are 120% of the longest side of the bounding box.
        let axis_length = get_debug_axis_length_from_template(template);

        // Extract scale from event
        let scale = event.scale;

        // Derive mesh path from the mesh template path (always mesh.glb in the template directory).
        let mesh_path = event
            .mesh_template_path
            .replace("asteroid.json", "mesh.glb");

        // Queue glTF mesh load.
        let gltf_handle = asset_server.load::<Gltf>(&mesh_path);

        // Spawn the asteroid as a static body with a pending mesh marker.
        // The mesh will be attached asynchronously once the glTF is loaded.
        commands.spawn((
            StaticBody,
            RigidBody::new(mass, 1.0), // inertia_scale = 1.0 for sphere
            collision_shape,
            Transform::from_translation(event.position)
                .with_rotation(event.rotation)
                .with_scale(scale),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            Name::new(event.id.clone()),
            PendingAsteroidMesh { gltf_handle },
            DebugAxesEligible::new(event.id.clone(), axis_length),
        ));

        info!("Spawned asteroid '{}' with mass {} kg", event.id, mass);
    }
}

/// System that attaches loaded glTF scenes to asteroid entities.
///
/// Runs in `Update` during `InGame`. When the glTF asset for an asteroid
/// finishes loading, this system extracts the scene from the glTF and
/// attaches it to the entity, removing the [`PendingAsteroidMesh`] marker.
///
/// Note: We only insert the `Handle<Scene>` to avoid overwriting the entity's
/// existing `Transform` and `GlobalTransform` that were set during spawning.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn attach_asteroid_meshes(
    mut commands: Commands<'_, '_>,
    query: Query<'_, '_, (Entity, &PendingAsteroidMesh)>,
    gltf_assets: Res<'_, Assets<Gltf>>,
) {
    for (entity, pending) in &query {
        // Check if the glTF asset has finished loading.
        if let Some(gltf) = gltf_assets.get(&pending.gltf_handle) {
            // Extract the first scene from the glTF.
            if let Some(scene_handle) = gltf.scenes.first().cloned() {
                // Insert only the scene handle to preserve the entity's existing transform.
                // The entity already has Transform/GlobalTransform from spawning.
                commands.entity(entity).insert(scene_handle);

                // Remove the pending marker now that scene is attached.
                commands.entity(entity).remove::<PendingAsteroidMesh>();
            }
        }
    }
}

/// Extracts mass from the asteroid template.
///
/// # Panics
///
/// Panics if mass is missing or invalid. This is intentional per ADR-0013.
#[allow(clippy::expect_used, clippy::cast_possible_truncation)]
fn get_mass_from_template(template: &Value) -> f32 {
    template
        .get("mass")
        .and_then(|m| m.get("value"))
        .and_then(serde_json::Value::as_f64)
        .map(|v| v as f32)
        .expect("asteroid template must have mass.value")
}

/// Extracts collision shape from the asteroid template.
///
/// # Panics
///
/// Panics if `collision_shape` is missing or has unsupported type.
/// This is intentional per ADR-0013.
#[allow(
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::uninlined_format_args
)]
fn get_collision_shape_from_template(template: &Value) -> CollisionShape {
    let shape = template
        .get("collision_shape")
        .expect("asteroid template must have collision_shape");

    let shape_type = shape
        .get("type")
        .and_then(serde_json::Value::as_str)
        .expect("collision_shape must have type");

    match shape_type {
        "sphere" => {
            let radius = shape
                .get("radius")
                .and_then(|r| r.get("value"))
                .and_then(serde_json::Value::as_f64)
                .map(|v| v as f32)
                .expect("sphere collision_shape must have radius.value");
            CollisionShape::sphere(radius, Vec3::ZERO)
        }
        _ => panic!("unsupported collision shape type: {shape_type}"),
    }
}

/// Extracts debug axis length from the bounding box.
///
/// Debug axes are 120% of the longest side of the bounding box.
///
/// # Panics
///
/// Panics if `bounding_box` is missing or invalid.
/// This is intentional per ADR-0013.
#[allow(clippy::expect_used, clippy::cast_possible_truncation)]
fn get_debug_axis_length_from_template(template: &Value) -> f32 {
    let bbox = template
        .get("bounding_box")
        .expect("asteroid template must have bounding_box");

    let min = bbox.get("min").expect("bounding_box must have min");
    let max = bbox.get("max").expect("bounding_box must have max");

    let min_x = min
        .get("x")
        .and_then(serde_json::Value::as_f64)
        .expect("bounding_box.min must have x") as f32;
    let min_y = min
        .get("y")
        .and_then(serde_json::Value::as_f64)
        .expect("bounding_box.min must have y") as f32;
    let min_z = min
        .get("z")
        .and_then(serde_json::Value::as_f64)
        .expect("bounding_box.min must have z") as f32;

    let max_x = max
        .get("x")
        .and_then(serde_json::Value::as_f64)
        .expect("bounding_box.max must have x") as f32;
    let max_y = max
        .get("y")
        .and_then(serde_json::Value::as_f64)
        .expect("bounding_box.max must have y") as f32;
    let max_z = max
        .get("z")
        .and_then(serde_json::Value::as_f64)
        .expect("bounding_box.max must have z") as f32;

    let size_x = max_x - min_x;
    let size_y = max_y - min_y;
    let size_z = max_z - min_z;

    let max_size = size_x.max(size_y).max(size_z);

    // 120% of the longest side
    max_size * 1.2
}
