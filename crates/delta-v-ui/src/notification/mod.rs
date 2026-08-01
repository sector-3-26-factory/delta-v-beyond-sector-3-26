// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Notification system for//!
//! Displays translated, temporary messages using the window framework.
//! The first use case is showing the active camera name after the player
//! switches cameras (F3 / Shift+F3).
//!
//! Notifications flicker in, display for a configurable duration, then flicker out.
//! Multiple notifications are stacked vertically.

use bevy::prelude::*;
use delta_v_core::AppState;

pub mod components;
pub mod resources;
pub mod spawn;
pub mod systems;

pub use resources::{NotificationConfig, NotificationStack};

/// Plugin for notification systems.
pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationConfig>()
            .init_resource::<NotificationStack>()
            .add_systems(
                Update,
                (
                    systems::notification_spawn_system.run_if(in_state(AppState::InGame)),
                    systems::targeting_mode_notification_system.run_if(in_state(AppState::InGame)),
                    systems::target_selected_notification_system.run_if(in_state(AppState::InGame)),
                    systems::weapon_selected_notification_system.run_if(in_state(AppState::InGame)),
                    systems::propulsion_selected_notification_system
                        .run_if(in_state(AppState::InGame)),
                    systems::notification_cleanup_system.run_if(in_state(AppState::InGame)),
                ),
            );
    }
}
