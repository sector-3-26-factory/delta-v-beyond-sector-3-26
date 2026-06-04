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

use bevy::gltf::{Gltf, GltfMesh};
use bevy::prelude::*;
use bevy::render::mesh::{Mesh, VertexAttributeValues};
use delta_v_core::{
    CameraFollow, ChaseCameraOffset, DebugAxes, DebugAxesEligible, FlightAssist, PlayerShipEntity,
    PlayerShipTemplate, ShipPropulsionConfig,
};
use delta_v_physics::RigidBody;
use delta_v_world::SpawnEntity;

/// Marker component for a pending ship entity waiting for its mesh to load.
#[derive(Component)]
pub(crate) struct PendingShipMesh {
    /// Handle to the glTF asset being loaded.
    gltf_handle: Handle<Gltf>,
}

/// Marker component for a ship whose chase camera offset needs to be computed
/// from the glTF bounding box.
///
/// When the chase camera position is not specified in the template JSON, this
/// component is inserted at spawn time. The `attach_ship_meshes` system reads
/// it and computes the offset from the bounding box once the mesh loads.
#[derive(Component)]
pub(crate) struct PendingChaseCamera;

/// Computes the bounding box half-extent from all meshes in a glTF asset.
///
/// Iterates through every [`GltfMesh`] and its primitives in the loaded glTF,
/// reads the `ATTRIBUTE_POSITION` vertex data from each primitive's [`Mesh`]
/// asset, and returns the maximum absolute value along each axis.
///
/// The returned `Vec3` contains the half-extents (max absolute value per axis).
/// Returns `None` if no meshes/primitives/position data are found.
fn compute_gltf_half_extent(
    gltf: &Gltf,
    gltf_meshes: &Assets<GltfMesh>,
    meshes: &Assets<Mesh>,
) -> Option<Vec3> {
    let mut max_extent = Vec3::ZERO;
    let mut found_any = false;

    for gltf_mesh_handle in &gltf.meshes {
        let Some(gltf_mesh) = gltf_meshes.get(gltf_mesh_handle) else {
            continue;
        };
        for primitive in &gltf_mesh.primitives {
            let Some(mesh) = meshes.get(&primitive.mesh) else {
                continue;
            };
            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                continue;
            };
            found_any = true;
            for pos in positions {
                max_extent.x = max_extent.x.max(pos[0].abs());
                max_extent.y = max_extent.y.max(pos[1].abs());
                max_extent.z = max_extent.z.max(pos[2].abs());
            }
        }
    }

    if found_any {
        Some(max_extent)
    } else {
        None
    }
}

/// Computes a default chase camera offset from the bounding box half-extents.
///
/// Places the camera behind and above the ship at a distance proportional
/// to the ship's size. The Z distance (behind) is 2× the largest half-extent,
/// and the Y distance (above) is 0.5× the largest half-extent.
fn compute_chase_offset(half_extent: Vec3) -> Vec3 {
    let max_extent = half_extent.x.max(half_extent.y).max(half_extent.z);
    Vec3::new(0.0, max_extent * 0.5, max_extent * 2.0)
}

/// Spawns ship entities in response to `SpawnEntity` events.
///
/// Listens for events with `entity_type` matching known ship types:
/// - `"player_controlled_ship"`: Player-controlled ship
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
            "npc_ship" => {
                // NPC ships: future implementation
                log::warn!("NPC ship spawning not yet implemented");
            }
            other => log::warn!("Unknown entity_type: {other}"),
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

/// Spawns the player-controlled ship from a template event.
///
/// Template is validated by delta-v-json; structure is guaranteed.
/// Mass, inertia, and propulsion values are read from the template JSON
/// per ADR-0014 (gameplay values in JSON, not Rust constants).
///
/// If the template specifies a chase camera position, it is used directly.
/// If the chase camera is omitted, a [`PendingChaseCamera`] marker is inserted
/// and the offset is computed from the glTF bounding box when the mesh loads.
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
    // The mesh_template_path points to the template that contains the mesh, which for
    // player_controlled_ship is the referenced ship_template, and for standalone ships
    // is the template itself.
    let mesh_path = event
        .mesh_template_path
        .replace("template.json", "mesh.glb");

    // Extract camera positions from the typed template struct.
    let cockpit = &template.cameras.cockpit;
    let chase = &template.cameras.chase;

    // Determine chase camera offset: use JSON value if present, otherwise
    // use a placeholder that will be overwritten when the mesh loads.
    let (chase_offset, needs_chase_computation) = if let Some(c) = chase {
        (Vec3::new(c.x, c.y, c.z), false)
    } else {
        // Placeholder; will be replaced by bounding-box-based value
        // in attach_ship_meshes.
        (Vec3::new(0.0, 5.0, 20.0), true)
    };

    // Queue glTF mesh load.
    let gltf_handle = asset_server.load::<Gltf>(&mesh_path);

    // Use a placeholder axis length based on world scale.
    // The real axis length is computed from the glTF bounding box once the
    // mesh finishes loading (see `attach_ship_meshes`).
    let placeholder_axis_length = (event
        .scale
        .x
        .abs()
        .max(event.scale.y.abs())
        .max(event.scale.z.abs()))
        * 2.0;

    log::debug!(
        "spawn_player_ship: placeholder axis_length={} from scale {:?} (real length computed after mesh load)",
        placeholder_axis_length,
        event.scale
    );

    // Build the ship entity spawn command.
    let mut ship_spawner = commands.spawn((
        Transform {
            translation: event.position,
            rotation: event.rotation,
            scale: event.scale,
        },
        GlobalTransform::default(),
        PendingShipMesh { gltf_handle },
        DebugAxesEligible::new(event.id.clone(), placeholder_axis_length),
        // Physics components: mass and inertia from template JSON (ADR-0014)
        RigidBody::new(template.mass.value, template.inertia_scale),
        FlightAssist,
    ));

    // If chase camera needs to be computed from bounding box, add marker.
    if needs_chase_computation {
        ship_spawner.insert(PendingChaseCamera);
    }

    let ship_entity = ship_spawner
        .with_children(|parent| {
            // Spawn cockpit camera as a child entity.
            parent.spawn(Transform::from_translation(Vec3::new(
                cockpit.x, cockpit.y, cockpit.z,
            )));
            // Spawn chase camera as a child entity.
            parent.spawn(Transform::from_translation(chase_offset));
        })
        .id();

    // Store player ship ID and chase camera offset for camera tracking.
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

/// Attaches loaded glTF meshes to pending ship entities.
///
/// Once the glTF asset finishes loading, this system extracts the first scene
/// from the glTF and attaches it to the ship entity with a `SceneBundle`.
///
/// After attaching the mesh, it computes the bounding box from the glTF vertex
/// data and:
/// - Updates [`DebugAxesEligible`] and [`DebugAxes`] with the correct axis length
/// - If the entity has [`PendingChaseCamera`], computes and sets the chase camera
///   offset from the bounding box and updates the [`CameraFollow`] component
///
/// Re-inserting `DebugAxes` triggers `Changed<DebugAxes>`, which causes
/// `update_debug_axes_on_change` to despawn the old axis root and spawn a new one.
#[allow(
    clippy::indexing_slicing,
    clippy::needless_pass_by_value,
    clippy::type_complexity
)]
pub(crate) fn attach_ship_meshes(
    mut commands: Commands<'_, '_>,
    gltf_assets: Res<'_, Assets<Gltf>>,
    gltf_mesh_assets: Res<'_, Assets<GltfMesh>>,
    mesh_assets: Res<'_, Assets<Mesh>>,
    mut chase_offset: ResMut<'_, ChaseCameraOffset>,
    mut camera_follow_query: Query<'_, '_, &'static mut CameraFollow>,
    query: Query<
        '_,
        '_,
        (
            Entity,
            &PendingShipMesh,
            &DebugAxesEligible,
            Option<&DebugAxes>,
            Option<&PendingChaseCamera>,
        ),
    >,
) {
    for (entity, pending, debug_eligible, debug_axes, pending_chase) in query.iter() {
        if let Some(gltf) = gltf_assets.get(&pending.gltf_handle) {
            // Get the first scene from the glTF (should contain the mesh).
            if !gltf.scenes.is_empty() {
                let scene_handle = gltf.scenes[0].clone();

                // Attach the scene as a child to the ship entity.
                commands.entity(entity).insert(SceneBundle {
                    scene: scene_handle,
                    transform: Transform::default(),
                    global_transform: GlobalTransform::default(),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::default(),
                    view_visibility: ViewVisibility::default(),
                });

                // Compute the bounding box from the glTF mesh data.
                if let Some(half_extent) =
                    compute_gltf_half_extent(gltf, &gltf_mesh_assets, &mesh_assets)
                {
                    // Update debug axis length to match the actual mesh size.
                    let axis_length = half_extent.x.max(half_extent.y).max(half_extent.z) * 2.0;
                    log::info!(
                        "attach_ship_meshes: computed axis_length={axis_length:.1} from glTF bounding box (half_extent={half_extent:?})"
                    );
                    // Re-insert DebugAxesEligible with the correct length.
                    commands.entity(entity).insert(DebugAxesEligible::new(
                        debug_eligible.entity_id.clone(),
                        axis_length,
                    ));
                    // If the entity already has DebugAxes (spawned during
                    // SpawningEntities with placeholder length), re-insert with
                    // the correct length so Changed<DebugAxes> fires and the
                    // visual axes are re-spawned at the correct scale.
                    if let Some(existing) = debug_axes {
                        commands
                            .entity(entity)
                            .insert(DebugAxes::new(existing.entity_id.clone(), axis_length));
                    }

                    // If the chase camera offset needs to be computed from
                    // the bounding box, compute it and update both the resource
                    // and the CameraFollow component on the camera entity.
                    if pending_chase.is_some() {
                        let computed_offset = compute_chase_offset(half_extent);
                        log::info!(
                            "attach_ship_meshes: computed chase offset {computed_offset:?} from bounding box"
                        );
                        chase_offset.0 = computed_offset;
                        // Update the CameraFollow component so the chase camera
                        // system uses the new offset starting this frame.
                        for mut follow in &mut camera_follow_query {
                            follow.offset = computed_offset;
                        }
                    }
                } else {
                    log::warn!(
                        "attach_ship_meshes: could not compute bounding box for glTF mesh on entity {entity:?}"
                    );
                }

                // Remove the pending markers now that mesh is attached.
                commands.entity(entity).remove::<PendingShipMesh>();
                commands.entity(entity).remove::<PendingChaseCamera>();

                log::debug!("attached glTF mesh to ship entity");
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
