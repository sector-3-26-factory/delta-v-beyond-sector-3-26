// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Ship entity spawning from templates.
//!
//! Per ADR-0038 (entity template system), ships are spawned from templates
//! loaded and validated by delta-v-json. Templates define mesh paths (glTF),
//! cameras, and other static properties. All meshes come from glTF files (ADR-0019).
//!
//! See also ADR-0005 (plugin architecture) and ADR-0006 (coordinate system).

use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_core::{ChaseCameraOffset, DebugAxesEligible, PlayerShipEntity};
use delta_v_world::SpawnEntity;

/// Marker component for a pending ship entity waiting for its mesh to load.
#[derive(Component)]
pub(crate) struct PendingShipMesh {
    /// Handle to the glTF asset being loaded.
    gltf_handle: Handle<Gltf>,
}

/// Spawns ship entities in response to `SpawnEntity` events.
///
/// Listens for events with `entity_type` matching known ship types:
/// - `"local_player_ship"`: Player-controlled ship
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
        if event.entity_type == "local_player_ship" {
            spawn_player_ship(&mut commands, &asset_server, event);
        }
        // Other ship types (e.g., "npc_ship") handled in future
    }
}

/// Spawns the player-controlled ship from a template event.
///
/// Template is validated by delta-v-json; structure is guaranteed.
#[allow(
    clippy::indexing_slicing,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]
fn spawn_player_ship(
    commands: &mut Commands<'_, '_>,
    asset_server: &Res<'_, AssetServer>,
    event: &SpawnEntity,
) {
    // Extract mesh path from validated template JSON (owned String for 'static lifetime).
    let mesh_path = event.template["mesh"]["path"]
        .as_str()
        .expect("mesh.path must be a string")
        .to_string();

    // Extract camera positions from validated template JSON.
    let cockpit_x = event.template["cameras"]["cockpit"]["x"]
        .as_f64()
        .expect("x must be a number") as f32;
    let cockpit_y = event.template["cameras"]["cockpit"]["y"]
        .as_f64()
        .expect("y must be a number") as f32;
    let cockpit_z = event.template["cameras"]["cockpit"]["z"]
        .as_f64()
        .expect("z must be a number") as f32;

    let chase_x = event.template["cameras"]["chase"]["x"]
        .as_f64()
        .expect("x must be a number") as f32;
    let chase_y = event.template["cameras"]["chase"]["y"]
        .as_f64()
        .expect("y must be a number") as f32;
    let chase_z = event.template["cameras"]["chase"]["z"]
        .as_f64()
        .expect("z must be a number") as f32;

    // Queue glTF mesh load.
    let gltf_handle = asset_server.load::<Gltf>(&mesh_path);

    // Calculate axis length as 2× the largest expansion along any axis (ADR-0006: coordinate system).
    let axis_length = (event
        .scale
        .x
        .abs()
        .max(event.scale.y.abs())
        .max(event.scale.z.abs()))
        * 2.0;

    log::debug!(
        "spawn_player_ship: calculated axis_length={} from scale {:?}",
        axis_length,
        event.scale
    );

    // Spawn ship entity with transform from world definition.
    // Mark it as pending mesh attachment and eligible for debug axes (ADR-0022, ADR-0005).
    // Use event.id (unique entity identifier) for debug filtering, not entity_type (ADR-0038).
    let ship_entity = commands
        .spawn((
            Transform {
                translation: event.position,
                rotation: event.rotation,
                scale: event.scale,
            },
            GlobalTransform::default(),
            PendingShipMesh { gltf_handle },
            DebugAxesEligible::new(event.id.clone(), axis_length),
        ))
        .with_children(|parent| {
            // Spawn cockpit and chase cameras as child entities.
            parent.spawn(Transform::from_translation(Vec3::new(
                cockpit_x, cockpit_y, cockpit_z,
            )));
            parent.spawn(Transform::from_translation(Vec3::new(
                chase_x, chase_y, chase_z,
            )));
        })
        .id();

    // Store player ship ID and chase camera offset for camera tracking.
    commands.insert_resource(PlayerShipEntity(ship_entity));
    commands.insert_resource(ChaseCameraOffset(Vec3::new(chase_x, chase_y, chase_z)));

    log::info!(
        "local player ship spawned at position ({:.1}, {:.1}, {:.1}) from {}",
        event.position.x,
        event.position.y,
        event.position.z,
        mesh_path
    );
}

/// Attaches loaded glTF meshes to pending ship entities.
///
/// Once the glTF asset finishes loading, this system extracts the first mesh
/// from the glTF and attaches it to the ship entity with a `SceneBundle`.
#[allow(clippy::indexing_slicing, clippy::needless_pass_by_value)]
pub(crate) fn attach_ship_meshes(
    mut commands: Commands<'_, '_>,
    gltf_assets: Res<'_, Assets<Gltf>>,
    query: Query<'_, '_, (Entity, &PendingShipMesh)>,
) {
    for (entity, pending) in query.iter() {
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

                // Remove the pending marker now that mesh is attached.
                commands.entity(entity).remove::<PendingShipMesh>();

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
