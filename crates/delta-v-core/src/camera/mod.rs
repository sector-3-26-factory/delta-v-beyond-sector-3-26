// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Camera follow component and chase-camera system.
//!
//! See ADR-0005 (plugin architecture) and ADR-0018 (state management).

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

/// Camera template types for deserializing camera positions from ship template JSON.
pub mod types;

pub use types::{CameraDefinition, ShipCamerasTemplate};

/// Stores the entity ID of the player-controlled ship.
#[derive(Resource)]
pub struct PlayerShipEntity(pub Entity);

/// Stores the chase camera offset from the template.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ChaseCameraOffset(pub Vec3);

/// Marker component for the currently active main camera.
#[derive(Component)]
pub struct ActiveMainCamera;

/// Instructs the chase-camera system to follow a target entity.
#[derive(Component, Debug)]
pub struct CameraFollow {
    /// The entity to track.
    pub target: Entity,
    /// Offset from the target's position in the target's local space.
    pub offset: Vec3,
}

/// Render layers for gameplay objects — belongs to ALL layers so every camera can see them.
pub fn gameplay_render_layers() -> RenderLayers {
    RenderLayers::layer(0).with(1).with(2).with(3)
}

/// Spawns the 3-D chase camera on layer 1 with `ActiveMainCamera` marker.
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_chase_camera(
    mut commands: Commands<'_, '_>,
    ship_entity: Res<'_, PlayerShipEntity>,
    camera_offset: Res<'_, ChaseCameraOffset>,
) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Transform::from_translation(camera_offset.0).looking_at(Vec3::ZERO, Vec3::Y),
        Visibility::default(),
        CameraFollow {
            target: ship_entity.0,
            offset: camera_offset.0,
        },
        RenderLayers::layer(1),
        ActiveMainCamera,
    ));

    log::info!("chase camera spawned at offset {:?}", camera_offset.0);
}

/// Spawns the 2-D UI camera required for rendering UI elements.
pub fn spawn_ui_camera(mut commands: Commands<'_, '_>) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        Transform::default(),
        Visibility::default(),
        bevy::ui::IsDefaultUiCamera,
    ));
    log::info!("UI camera (Camera2d) spawned with IsDefaultUiCamera");
}

/// Moves the camera to maintain its offset behind the followed entity.
#[allow(clippy::needless_pass_by_value)]
pub fn chase_camera_system(
    mut camera_query: Query<'_, '_, (&mut Transform, &CameraFollow)>,
    target_query: Query<'_, '_, &Transform, Without<CameraFollow>>,
) {
    for (mut cam_transform, follow) in &mut camera_query {
        let Ok(target_transform) = target_query.get(follow.target) else {
            continue;
        };
        let world_offset = target_transform.rotation * follow.offset;
        cam_transform.translation = target_transform.translation + world_offset;
        let ship_up = target_transform.rotation * Vec3::Y;
        cam_transform.look_at(target_transform.translation, ship_up);
    }
}

/// Debug system that logs camera positions each frame.
#[allow(clippy::needless_pass_by_value)]
pub fn debug_camera_positions(
    camera_query: Query<'_, '_, (&Transform, &CameraFollow)>,
    target_query: Query<'_, '_, (&Transform, &Name)>,
) {
    for (cam_transform, follow) in &camera_query {
        if let Ok((target_transform, target_name)) = target_query.get(follow.target) {
            log::debug!(
                "Camera: pos={:?}, looking at '{}' at {:?}",
                cam_transform.translation,
                target_name,
                target_transform.translation
            );
        }
    }
}
