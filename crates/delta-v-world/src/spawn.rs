// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Asteroid spawning system.
//!
//! Listens for [`SpawnEntity`] events with `entity_type: "asteroid"` and spawns
//! dynamic rigid body asteroids with collision shapes. Asteroids are affected
//! by collisions based on their mass: heavy asteroids barely move when struck
//! by a ship, while lightweight asteroids are displaced realistically.

use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_core::{DebugAxesEligible, SpawnEntity};
use delta_v_physics::{DynamicBody, RigidBody};
use delta_v_spawn::collision::shape_from_json;
use delta_v_spawn::template_extraction::{
    compute_debug_axis_length, extract_bounding_box, extract_mass,
};
use delta_v_types::CollisionShapeJson;
use serde_json::Value;

/// Marker component for a pending asteroid mesh waiting for its glTF to load.
#[derive(Component)]
pub(crate) struct PendingAsteroidMesh {
    /// Handle to the glTF asset being loaded.
    gltf_handle: Handle<Gltf>,
}

impl delta_v_spawn::mesh_attachment::PendingMesh for PendingAsteroidMesh {
    fn gltf_handle(&self) -> &Handle<Gltf> {
        &self.gltf_handle
    }
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

        // Extract mass (required) using delta-v-spawn utilities (ADR-0047)
        let mass = extract_mass(template);

        // Extract collision shape (required) using delta-v-spawn utilities (ADR-0047)
        let collision_shape = get_collision_shape_from_template(template);

        // Extract bounding box (required) for debug axes computation using delta-v-spawn utilities
        let bbox = extract_bounding_box(template);
        let axis_length = compute_debug_axis_length(&bbox);

        // Extract scale from event
        let scale = event.scale;

        // Derive mesh path from the mesh template path (always mesh.glb in the template directory).
        let mesh_path = event
            .mesh_template_path
            .replace("asteroid.json", "mesh.glb");

        // Queue glTF mesh load.
        let gltf_handle = asset_server.load::<Gltf>(&mesh_path);

        // Spawn the asteroid as a dynamic body with a pending mesh marker.
        // The mesh will be attached asynchronously once the glTF is loaded.
        // Dynamic asteroids respond to collisions based on their mass.
        commands.spawn((
            DynamicBody,
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

/// Extracts collision shape from the asteroid template using delta-v-spawn utilities.
///
/// Uses `delta_v_spawn::collision::shape_from_json` for centralized collision shape
/// conversion (ADR-0047).
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
fn get_collision_shape_from_template(template: &Value) -> delta_v_physics::CollisionShape {
    let shape = template
        .get("collision_shape")
        .expect("asteroid template must have collision_shape");

    // Deserialize the collision shape JSON into CollisionShapeJson
    let collision_shape_json: CollisionShapeJson = serde_json::from_value(shape.clone())
        .expect("collision_shape must be valid JSON (ADR-0013)");

    // Use the centralized shape_from_json function from delta-v-spawn
    shape_from_json(&collision_shape_json, 1.0).expect("collision shape must be valid (ADR-0013)")
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
