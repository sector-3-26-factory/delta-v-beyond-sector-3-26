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

/// Marker component for the currently active main camera.
#[derive(Component)]
pub struct ActiveMainCamera;

/// Render layers for gameplay objects — belongs to ALL layers so every camera can see them.
pub fn gameplay_render_layers() -> RenderLayers {
    RenderLayers::layer(0)
        .with(1)
        .with(2)
        .with(3)
        .with(4)
        .with(5)
        .with(6)
        .with(7)
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
        RenderLayers::layer(8),
        bevy::ui::IsDefaultUiCamera,
        bevy_lunex::UiSourceCamera::<0>,
    ));
    log::info!("UI camera (Camera2d) spawned with IsDefaultUiCamera");
}
