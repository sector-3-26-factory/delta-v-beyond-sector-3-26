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

/// Spawns the 2-D UI camera required for rendering the cockpit overlay PNG.
///
/// Renders on `Layer(1)` with `order: 1`. This camera sees the cockpit
/// interior view (`.png` with alpha transparency). The `ActiveMainCamera`'s
/// 3D world is visible through the transparent areas.
pub fn spawn_ui_camera(mut commands: Commands<'_, '_>) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            is_active: true,
            ..default()
        },
        Transform::default(),
        Visibility::default(),
        RenderLayers::layer(1),
        bevy::ui::IsDefaultUiCamera,
    ));
    log::info!("UI camera (Camera2d) spawned with IsDefaultUiCamera");
}

/// Spawns the Lunex menu camera.
///
/// Renders on layers 2, 3, 4 with `order: 3`. Layer 2 has Lunex UI elements
/// (keybindings menu, etc.). Layers 3 and 4 are for Lunex debug gizmos
/// (2D and 3D outlines). Carries `UiSourceCamera::<2>` so that Lunex
/// `UiFetchFromCamera::<2>` widgets render correctly. Uses ID 2 to avoid
/// collision with Bevy's default internal camera ID 0.
pub fn spawn_menu_camera(mut commands: Commands<'_, '_>) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 3,
            is_active: true,
            ..default()
        },
        Transform::default(),
        Visibility::default(),
        RenderLayers::from_layers(&[2, 3, 4]),
        bevy_lunex::UiSourceCamera::<2>,
    ));
    log::info!("Menu camera (Lunex) spawned with UiSourceCamera::<2>");
}
