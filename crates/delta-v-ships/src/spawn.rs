// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Ship entity spawning from templates.
//!
//! Per ADR-0038 (entity template system), ships are spawned from templates
//! loaded and validated by delta-v-json. Templates define
//! cameras, physical properties (mass, inertia), and propulsion configuration.
//! All meshes come from glTF files (ADR-0019).
//!
//! Per ADR-0014, all gameplay values (mass, thrust, torque) come from JSON
//! — never from Rust constants. The template `Value` is deserialized into
//! [`PlayerShipTemplate`] via `serde_json::from_value` (a one-liner per ADR-0040).
//!
//! See also ADR-0005 (plugin architecture) and ADR-0006 (coordinate system).

use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_core::{
    ChaseCameraOffset, DebugAxes, DebugAxesEligible, FlightAssist, PlayerShipEntity,
    PlayerShipTemplate, ShipCollisionShape, ShipPropulsionConfig, SpawnEntity, StaticShipTemplate,
};
use delta_v_physics::{CollisionShape, RigidBody};

/// Marker component for a pending ship entity waiting for its mesh to load.
#[derive(Component)]
pub(crate) struct PendingShipMesh {
    /// Handle to the glTF asset being loaded.
    gltf_handle: Handle<Gltf>,
}

/// Spawns ship entities in response to `SpawnEntity` events.
///
/// Listens for events with `entity_type` matching known ship types:
/// - `"player_controlled_ship"`: Player-controlled ship
/// - `"ship"`: Non-player ship (static/NPC)
/// - `"npc_ship"`: NPC-controlled ship (future)
///
/// The event's `template` field contains validated template JSON from delta-v-json.
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_ship_from_template(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    mut events: EventReader<'_, '_, SpawnEntity>,
) {
    for event in events.read() {
        match event.entity_type.as_str() {
            "player_controlled_ship" => spawn_player_ship(&mut commands, &asset_server, event),
            "ship" => spawn_static_ship(&mut commands, &asset_server, event),
            "npc_ship" => {
                // NPC ships: future implementation
                log::warn!("NPC ship spawning not yet implemented");
            }
            // Other entity types (e.g. "asteroid") are handled by other plugins.
            // Silently skip — a single plugin cannot know whether another plugin
            // will handle the event.
            _ => {}
        }
    }
}

/// Deserializes the template JSON into a [`PlayerShipTemplate`] struct.
///
/// Per ADR-0040, the template `Value` has already been validated and
/// filled with schema defaults by `delta-v-json`, so deserialization
/// into the struct is a one-liner.
#[allow(clippy::expect_used)] // INVARIANT: template validated by delta-v-json; cannot fail
fn deserialize_template(event: &SpawnEntity) -> PlayerShipTemplate {
    serde_json::from_value(event.template.clone())
        .expect("template deserialization must succeed (validated by delta-v-json, ADR-0040)")
}

/// Creates a [`CollisionShape`] from a [`ShipCollisionShape`] template, scaled by the given factor.
///
/// Supports sphere and box shapes. Panics on unknown shape types per ADR-0013.
#[allow(clippy::expect_used, clippy::panic)]
fn create_collision_shape(shape: &ShipCollisionShape, scale: f32) -> CollisionShape {
    let offset = shape
        .offset
        .as_ref()
        .map_or(Vec3::ZERO, |o| Vec3::new(o.x, o.y, o.z) * scale);
    match shape.shape_type.as_str() {
        "sphere" => {
            let radius = shape
                .radius
                .as_ref()
                .expect("sphere collision_shape must have radius")
                .value;
            CollisionShape::sphere(radius * scale, offset)
        }
        "box" => {
            let he = shape
                .half_extents
                .as_ref()
                .expect("box collision_shape must have half_extents");
            let half_extents = Vec3::new(he.x, he.y, he.z) * scale;
            CollisionShape::box_shape(half_extents, offset)
        }
        _ => panic!("unsupported collision shape type: {}", shape.shape_type),
    }
}

/// Spawns the player-controlled ship from a template event.
///
/// Template is validated by delta-v-json; structure is guaranteed.
/// Mass, inertia, and propulsion values are read from the template JSON
/// per ADR-0014 (gameplay values in JSON, not Rust constants).
///
/// All 8 cameras are defined in the template with position, target, and availability.
/// Only cameras with `available: true` are spawned as camera entities.
///
/// Debug axes length is computed from the bounding box stored in the template JSON.
#[allow(
    clippy::option_if_let_else,
    clippy::indexing_slicing,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]
fn spawn_player_ship(
    commands: &mut Commands<'_, '_>,
    asset_server: &Res<'_, AssetServer>,
    event: &SpawnEntity,
) {
    // Deserialize template JSON into typed struct (ADR-0040 one-liner).
    let template = deserialize_template(event);

    // Extract propulsion values from the active main thruster.
    let active_index = 0_usize; // M2: single active thruster
    let main = &template.propulsion.main_thrusters[active_index];
    let maneuvering = &template.propulsion.maneuvering_thruster;

    // Derive mesh path from the mesh template path (always mesh.glb in the template directory).
    let mesh_path = event.mesh_template_path.replace("ship.json", "mesh.glb");

    // Queue glTF mesh load.
    let gltf_handle = asset_server.load::<Gltf>(&mesh_path);

    // Extract uniform scale from event (use max of x, y, z for uniform scaling).
    let scale = event.scale.x.max(event.scale.y).max(event.scale.z);

    // Compute debug axes length from the bounding box in the template JSON, scaled.
    // The bounding_box is already in the template JSON (computed by tooling).
    let half_extent = Vec3::new(
        (template.bounding_box.max.x - template.bounding_box.min.x) / 2.0,
        (template.bounding_box.max.y - template.bounding_box.min.y) / 2.0,
        (template.bounding_box.max.z - template.bounding_box.min.z) / 2.0,
    );
    let axis_length = half_extent.x.max(half_extent.y).max(half_extent.z) * 2.0 * scale;

    log::debug!("spawn_player_ship: axis_length={axis_length:.1} from bounding_box in template (scale={scale})");

    // Build the ship entity spawn command.
    let collision_shape = create_collision_shape(&template.collision_shape, scale);

    let ship_entity = commands
        .spawn((
            Transform {
                translation: event.position,
                rotation: event.rotation,
                scale: event.scale,
            },
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            PendingShipMesh { gltf_handle },
            DebugAxesEligible::new(event.id.clone(), axis_length),
            // Physics components: mass and inertia from template JSON (ADR-0014)
            RigidBody::new(template.mass.value, template.inertia_scale),
            FlightAssist,
            collision_shape,
        ))
        .id();

    // Spawn cameras for each available camera definition, scaled by the entity scale.
    for (name, camera) in [
        ("cockpit", &template.cameras.cockpit),
        ("chase", &template.cameras.chase),
        ("rear", &template.cameras.rear),
        ("front", &template.cameras.front),
        ("left", &template.cameras.left),
        ("right", &template.cameras.right),
        ("top", &template.cameras.top),
        ("bottom", &template.cameras.bottom),
    ] {
        if camera.available {
            let position =
                Vec3::new(camera.position.x, camera.position.y, camera.position.z) * scale;
            let target = Vec3::new(camera.target.x, camera.target.y, camera.target.z) * scale;
            commands.entity(ship_entity).with_children(|parent| {
                let _camera_entity = parent.spawn((
                    Transform::from_translation(position).looking_at(target, Vec3::Y),
                    delta_v_core::CameraFollow {
                        target: ship_entity,
                        offset: position,
                    },
                ));
                log::debug!("spawned {name} camera at {position:?}");
            });
        }
    }

    // Store player ship ID and chase camera offset for camera tracking.
    // Use the chase camera's position and target to compute the offset, scaled.
    let chase_offset = Vec3::new(
        template.cameras.chase.position.x,
        template.cameras.chase.position.y,
        template.cameras.chase.position.z,
    ) * scale;
    commands.insert_resource(PlayerShipEntity(ship_entity));
    commands.insert_resource(ChaseCameraOffset(chase_offset));

    // Insert propulsion configuration from template JSON (ADR-0014).
    // These values are read by the input → forces pipeline each tick.
    commands.insert_resource(ShipPropulsionConfig {
        max_forward_thrust: main.max_forward_thrust.value,
        max_backward_thrust: main.max_backward_thrust.value,
        max_torque: maneuvering.max_torque.value,
        max_strafe_thrust: maneuvering.max_strafe_thrust.value,
        active_main_thruster_index: active_index,
    });

    log::info!(
        "player controlled ship spawned at position ({:.1}, {:.1}, {:.1}) from {} (mass={}kg, forward_thrust={}N, backward_thrust={}N)",
        event.position.x,
        event.position.y,
        event.position.z,
        mesh_path,
        template.mass.value,
        main.max_forward_thrust.value,
        main.max_backward_thrust.value,
    );
}

/// Spawns a non-player ship from a template event.
///
/// This is for `entity_type: "ship"` templates — ships that don't require
/// player input, cameras, or propulsion configuration.
///
/// Template is validated by delta-v-json; structure is guaranteed.
#[allow(
    clippy::option_if_let_else,
    clippy::indexing_slicing,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]
fn spawn_static_ship(
    commands: &mut Commands<'_, '_>,
    asset_server: &Res<'_, AssetServer>,
    event: &SpawnEntity,
) {
    // Deserialize template JSON into typed struct (ADR-0040 one-liner).
    let template: StaticShipTemplate = serde_json::from_value(event.template.clone())
        .expect("template deserialization must succeed (validated by delta-v-json, ADR-0040)");

    // Derive mesh path from the mesh template path (always mesh.glb in the template directory).
    let mesh_path = event.mesh_template_path.replace("ship.json", "mesh.glb");

    // Queue glTF mesh load.
    let gltf_handle = asset_server.load::<Gltf>(&mesh_path);

    // Extract uniform scale from event (use max of x, y, z for uniform scaling).
    let scale = event.scale.x.max(event.scale.y).max(event.scale.z);

    // Compute debug axes length from the bounding box in the template JSON, scaled.
    let half_extent = Vec3::new(
        (template.bounding_box.max.x - template.bounding_box.min.x) / 2.0,
        (template.bounding_box.max.y - template.bounding_box.min.y) / 2.0,
        (template.bounding_box.max.z - template.bounding_box.min.z) / 2.0,
    );
    let axis_length = half_extent.x.max(half_extent.y).max(half_extent.z) * 2.0 * scale;

    log::debug!("spawn_static_ship: axis_length={axis_length:.1} from bounding_box in template (scale={scale})");

    // Build the ship entity spawn command.
    let collision_shape = create_collision_shape(&template.collision_shape, scale);

    commands.spawn((
        Transform {
            translation: event.position,
            rotation: event.rotation,
            scale: event.scale,
        },
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        PendingShipMesh { gltf_handle },
        DebugAxesEligible::new(event.id.clone(), axis_length),
        // Physics components: mass and inertia from template JSON (ADR-0014)
        RigidBody::new(template.mass.value, template.inertia_scale),
        FlightAssist,
        collision_shape,
    ));

    log::info!(
        "static ship spawned at position ({:.1}, {:.1}, {:.1}) from {} (mass={}kg)",
        event.position.x,
        event.position.y,
        event.position.z,
        mesh_path,
        template.mass.value,
    );
}

/// Attaches loaded glTF meshes to pending ship entities.
///
/// Once the glTF asset finishes loading, this system extracts the first scene
/// from the glTF and attaches it to the ship entity.
///
/// Debug axes are already configured with the correct length from the template JSON
/// (per ADR-0014: bounding box is the single source of truth).
///
/// Note: We only insert the `Handle<Scene>` to avoid overwriting the entity's
/// existing `Transform` and `GlobalTransform` that were set during spawning.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn attach_ship_meshes(
    mut commands: Commands<'_, '_>,
    gltf_assets: Res<'_, Assets<Gltf>>,
    query: Query<
        '_,
        '_,
        (
            Entity,
            &PendingShipMesh,
            &DebugAxesEligible,
            Option<&DebugAxes>,
        ),
    >,
) {
    for (entity, pending, debug_eligible, _debug_axes) in query.iter() {
        if let Some(gltf) = gltf_assets.get(&pending.gltf_handle) {
            // Get the first scene from the glTF (should contain the mesh).
            if let Some(scene_handle) = gltf.scenes.first().cloned() {
                // Insert only the scene handle to preserve the entity's existing transform.
                // The entity already has Transform/GlobalTransform from spawning.
                commands.entity(entity).insert(scene_handle);

                // Debug axes are already configured from the template JSON.
                // No need to recompute from glTF.
                log::debug!(
                    "attached glTF mesh to ship entity (debug axes from template: entity_id={}, axis_length={:.1})",
                    debug_eligible.entity_id,
                    debug_eligible.axis_length
                );

                // Remove the pending marker now that mesh is attached.
                commands.entity(entity).remove::<PendingShipMesh>();
            }
        }
    }
}

/// Spawns lighting for the 3-D scene.
///
/// Runs once per world load. Creates:
/// - A directional light simulating a distant sun.
/// - Ambient light for general illumination.
pub fn setup_scene_lighting(mut commands: Commands<'_, '_>) {
    // Directional light (sun-like): rotated to create interesting shadows.
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
        ..default()
    });

    // Ambient light for general scene fill.
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
    });

    log::info!("scene lighting initialized");
}
