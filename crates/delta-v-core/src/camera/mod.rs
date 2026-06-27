// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Camera switching system for ship cameras.
//!
//! See ADR-0005 (plugin architecture) and ADR-0018 (state management).

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::input::ActionState;
use delta_v_types::LogicalAction;

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

/// Name identifier for a ship camera.
///
/// Stored on camera entities spawned by `spawn_cameras` in delta-v-ships.
/// Used by `camera_switch_system` to identify and query cameras by name.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct CameraName(pub &'static str);

/// Resource tracking the currently active camera name.
///
/// Defaults to `"cockpit"`. Updated by `camera_switch_system` when the player
/// switches cameras.
// allow-default: Bevy requires Default on resources for init_resource.
// This is runtime state, not configuration.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct ActiveCameraName(pub String);

/// Fixed order for camera switching.
/// Cameras are cycled in this order: cockpit → front → rear → left → right → top → bottom → drone
const CAMERA_ORDER: [&str; 8] = [
    "cockpit", "front", "rear", "left", "right", "top", "bottom", "drone",
];

/// Cycles between available ship cameras when the player presses the switch keys.
///
/// Runs in `Update` during `AppState::InGame`.
/// Uses edge detection to fire once per key press.
/// Camera order is fixed: cockpit → front → rear → left → right → top → bottom.
/// Only cameras with `available: true` that were actually spawned are in the query results.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub fn camera_switch_system(
    mut commands: Commands<'_, '_>,
    action_state: Res<'_, ActionState<LogicalAction>>,
    mut active_camera: ResMut<'_, ActiveCameraName>,
    mut cycle_state: ResMut<'_, CameraSwitchCycleState>,
    mut query: Query<'_, '_, (Entity, &CameraName, &Camera3d, &'static mut Camera)>,
) {
    use LogicalAction;

    let next_pressed = action_state.just_pressed(&LogicalAction::CameraSwitchNext);
    let prev_pressed = action_state.just_pressed(&LogicalAction::CameraSwitchPrev);

    // Edge detection: only switch when key is first pressed
    let switching = if next_pressed && !cycle_state.next_pressed {
        1
    } else if prev_pressed && !cycle_state.prev_pressed {
        -1
    } else {
        cycle_state.next_pressed = next_pressed;
        cycle_state.prev_pressed = prev_pressed;
        return;
    };

    cycle_state.next_pressed = next_pressed;
    cycle_state.prev_pressed = prev_pressed;

    // Collect all available cameras in the fixed order
    let mut cameras: Vec<_> = query
        .iter()
        .filter(|(_, name, _, _)| CAMERA_ORDER.contains(&name.0))
        .map(|(entity, name, _camera, _transform)| (entity, name.0))
        .collect();
    // Sort by the fixed order
    cameras.sort_by_key(|(_, name)| {
        CAMERA_ORDER
            .iter()
            .position(|&n| n == *name)
            .unwrap_or(usize::MAX)
    });

    if cameras.is_empty() {
        return;
    }

    // Find current camera index
    let current_idx = cameras
        .iter()
        .position(|(_, name)| *name == active_camera.0)
        .unwrap_or(0);

    // Calculate new index with wrapping
    let new_idx = if switching > 0 {
        (current_idx + 1) % cameras.len()
    } else {
        (current_idx + cameras.len() - 1) % cameras.len()
    };

    // SAFETY: new_idx is always valid due to modulo operation above.
    let (new_entity, new_name) = cameras.get(new_idx).copied().unwrap_or_else(|| {
        // This should never happen - cameras.len() > 0 and new_idx is modulo'd
        unreachable!("camera index out of bounds: {new_idx} >= {}", cameras.len())
    });

    // Deactivate old camera
    if let Some((old_entity, _, _, mut old_camera)) = query
        .iter_mut()
        .find(|(_, name, _, _)| *name.0 == active_camera.0)
    {
        old_camera.is_active = false;
        commands.entity(old_entity).remove::<ActiveMainCamera>();
    }

    // Activate new camera
    if let Ok((_, _, _, mut new_camera)) = query.get_mut(new_entity) {
        new_camera.is_active = true;
        commands.entity(new_entity).insert(ActiveMainCamera);
    }

    active_camera.0 = new_name.to_string();
    tracing::debug!("switched to camera '{}' (entity {new_entity:?})", new_name);
}

/// Tracks which camera switch keys were already consumed to prevent repeated firing.
// allow-default: Bevy requires Default on resources for init_resource.
// This is runtime state, not configuration.
#[derive(Resource, Default)]
pub struct CameraSwitchCycleState {
    /// Whether `CameraSwitchNext` was active last tick.
    next_pressed: bool,
    /// Whether `CameraSwitchPrev` was active last tick.
    prev_pressed: bool,
}

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
