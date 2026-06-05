// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Camera follow component and chase-camera system.
//!
//! The chase camera is intentionally simple at M1: fixed offset, no
//! lag, no spring damping. Those are M6 concerns.
//!
//! This module also owns the camera-related template structs used to
//! deserialize camera positions from the ship template JSON.
//!
//! See ADR-0005 (plugin architecture) and ADR-0018 (state management).

use bevy::prelude::*;
use serde::Deserialize;

/// Stores the entity ID of the player-controlled ship.
///
/// Inserted by `ShipsPlugin` when the player ship is spawned.
/// Used by the chase camera system to know which entity to follow.
#[derive(Resource)]
pub struct PlayerShipEntity(pub Entity);

/// Stores the chase camera offset from the template.
///
/// Inserted by `ShipsPlugin` when the player ship is spawned.
/// Used by `spawn_chase_camera` to position the camera correctly.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ChaseCameraOffset(pub Vec3);

/// Instructs the chase-camera system to follow a target entity.
///
/// Attach to the camera entity. Set `target` to the entity to follow.
#[derive(Component, Debug)]
pub struct CameraFollow {
    /// The entity to track.
    pub target: Entity,
    /// Offset from the target's position in the target's local space.
    /// Default: 20 m behind, 8 m above (Vec3 in target-local coords).
    pub offset: Vec3,
}

/// A single camera definition with position, target (look-at point), and availability.
#[derive(Debug, Deserialize)]
pub struct CameraDefinition {
    /// Camera position relative to ship center (metres).
    pub position: Vec3Json,
    /// Point the camera looks at, relative to ship center (metres).
    /// Direction = normalize(target - position).
    pub target: Vec3Json,
    /// If true, this camera is physically present and accessible.
    /// If false, the position/target are computed but not available to the player.
    pub available: bool,
}

/// Camera definitions from the ship template.
///
/// All 8 cameras are required in the schema. Each has a position, target, and availability flag.
#[derive(Debug, Deserialize)]
pub struct ShipCamerasTemplate {
    /// Cockpit camera (inside the cockpit, typically front-upper-center).
    pub cockpit: CameraDefinition,
    /// Chase camera (behind and above the ship).
    pub chase: CameraDefinition,
    /// Rear view camera (behind at cockpit height).
    pub rear: CameraDefinition,
    /// Front/nose camera (forward view).
    pub front: CameraDefinition,
    /// Left side view camera.
    pub left: CameraDefinition,
    /// Right side view camera.
    pub right: CameraDefinition,
    /// Top-down view camera.
    pub top: CameraDefinition,
    /// Bottom-up view camera.
    pub bottom: CameraDefinition,
}

/// A 3-component vector deserialized from JSON `{"x": N, "y": N, "z": N}`.
#[derive(Debug, Deserialize)]
pub struct Vec3Json {
    /// X component in metres.
    pub x: f32,
    /// Y component in metres.
    pub y: f32,
    /// Z component in metres.
    pub z: f32,
}

/// Spawns the 3-D chase camera after the player ship entity exists.
///
/// Runs during `OnEnter(AppState::InGame)`. Reads the `PlayerShipEntity`
/// resource to determine which entity to follow. Panics if the resource
/// is not present (programming error in plugin sequencing).
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_chase_camera(
    mut commands: Commands<'_, '_>,
    ship_entity: Res<'_, PlayerShipEntity>,
    camera_offset: Res<'_, ChaseCameraOffset>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(camera_offset.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraFollow {
            target: ship_entity.0,
            offset: camera_offset.0,
        },
    ));

    log::info!(
        "chase camera spawned at offset {:?}, following player ship",
        camera_offset.0
    );
}

/// Moves the camera to maintain its offset behind the followed entity.
///
/// Runs every frame in `Update` when `AppState::InGame`.
/// Rotates the offset by the ship's current rotation so the camera
/// stays behind the ship as it turns.
///
/// Uses the ship's local up vector (+Y in ship space) as the `look_at`
/// up reference to avoid the gimbal lock singularity that occurs with
/// a fixed world `Vec3::Y` when the camera is directly above or below
/// the target (e.g. at 90° pitch).
#[allow(clippy::needless_pass_by_value)]
pub fn chase_camera_system(
    mut camera_query: Query<'_, '_, (&mut Transform, &CameraFollow)>,
    target_query: Query<'_, '_, &Transform, Without<CameraFollow>>,
) {
    for (mut cam_transform, follow) in &mut camera_query {
        let Ok(target_transform) = target_query.get(follow.target) else {
            continue;
        };

        // Rotate offset by the ship's current rotation so camera
        // stays behind the ship as it turns.
        let world_offset = target_transform.rotation * follow.offset;
        cam_transform.translation = target_transform.translation + world_offset;

        // Use the ship's local up vector as the look_at up reference.
        // This avoids gimbal lock when the camera is above/below the ship.
        let ship_up = target_transform.rotation * Vec3::Y;
        cam_transform.look_at(target_transform.translation, ship_up);
    }
}
