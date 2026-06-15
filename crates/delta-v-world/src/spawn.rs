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
use delta_v_physics::{CollisionLayersComponent, CollisionShape, DynamicBody, RigidBody};
use delta_v_spawn::collision::shape_from_json;
use delta_v_spawn::template_extraction::{
    compute_debug_axis_length, extract_bounding_box, extract_collision_shape, extract_mass,
};
use delta_v_types::collision::layers;

/// Marker component for a pending asteroid mesh waiting for its glTF to load.
#[derive(Component)]
pub struct PendingAsteroidMesh {
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
#[allow(clippy::needless_pass_by_value, clippy::expect_used)]
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
        // INVARIANT: collision_shape is required by schema and validated by delta-v-json (ADR-0013)
        let collision_shape_json = extract_collision_shape(template);
        let collision_shape_data = shape_from_json(&collision_shape_json, 1.0)
            .expect("collision shape must be valid (ADR-0013)");

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
            CollisionShape(collision_shape_data),
            CollisionLayersComponent::new(layers::ASTEROID),
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
