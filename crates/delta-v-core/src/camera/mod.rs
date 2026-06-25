// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Camera follow component and chase-camera system.
//!
//! See ADR-0005 (plugin architecture) and ADR-0018 (state management).

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

/// Camera template types for deserializing camera positions from ship template JSON.
pub mod types;

pub use types::{CameraDefinition, ShipCamerasTemplate};

/// Named render layer indices for the camera/layer system.
///
/// | Layer | Name | What it renders |
/// |-------|------|-----------------|
/// | 0 | Gameplay | 3D world, ship cameras, gameplay entities |
/// | 1 | Ui | Cockpit overlay PNG, Bevy UI |
/// | 2 | Menu | Bevy UI windows, keybindings menu |
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderLayer {
    /// Layer 0 — Gameplay: 3D world, ship cameras, all gameplay entities.
    Gameplay,
    /// Layer 1 — UI: Cockpit overlay PNG, Bevy UI.
    Ui,
    /// Layer 2 — Menu: Bevy UI windows, keybindings menu.
    Menu,
}

impl RenderLayer {
    /// Returns the raw layer index as `usize`.
    pub const fn index(self) -> usize {
        match self {
            Self::Gameplay => 0,
            Self::Ui => 1,
            Self::Menu => 2,
        }
    }

    /// Returns a `RenderLayers` containing only this layer.
    pub const fn render_layers(self) -> RenderLayers {
        RenderLayers::layer(self.index())
    }
}

impl From<RenderLayer> for RenderLayers {
    fn from(layer: RenderLayer) -> Self {
        layer.render_layers()
    }
}

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
        RenderLayer::Ui.render_layers(),
        bevy::ui::IsDefaultUiCamera,
    ));
    tracing::info!("UI camera (Camera2d) spawned with IsDefaultUiCamera");
}

/// Spawns the menu camera for Bevy UI windows and menus.
///
/// Renders on `Layer(2)` with `order: 3`. This camera sees the keybindings
/// menu, window overlays, and other UI elements that should render on top
/// of the cockpit and 3D world.
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
        RenderLayer::Menu.render_layers(),
    ));
    tracing::info!("Menu camera (Bevy UI) spawned on layer 2");
}
