// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Camera follow component and chase-camera system.
//!
//! The chase camera is intentionally simple at M1: fixed offset, no
//! lag, no spring damping. Those are M6 concerns.
//!
//! See ADR-0005 (plugin architecture) and ADR-0018 (state management).

use bevy::prelude::*;

/// Stores the entity ID of the player-controlled ship.
///
/// Inserted by `ShipsPlugin` when the player ship is spawned.
/// Used by the chase camera system to know which entity to follow.
#[derive(Resource)]
pub struct PlayerShipEntity(pub Entity);

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

/// Spawns the 3-D chase camera after the player ship entity exists.
///
/// Runs during `OnEnter(AppState::InGame)`. Reads the `PlayerShipEntity`
/// resource to determine which entity to follow. Panics if the resource
/// is not present (programming error in plugin sequencing).
pub fn spawn_chase_camera(mut commands: Commands<'_, '_>, ship_entity: Res<'_, PlayerShipEntity>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 8.0, 20.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraFollow {
            target: ship_entity.0,
            offset: Vec3::new(0.0, 8.0, 20.0),
        },
    ));

    log::info!("chase camera spawned, following player ship");
}

/// Moves the camera to maintain its offset behind the followed entity.
///
/// Runs every frame in `Update` when `AppState::InGame`.
/// Rotates the offset by the ship's current rotation so the camera
/// stays behind the ship as it turns.
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
        cam_transform.look_at(target_transform.translation, Vec3::Y);
    }
}
