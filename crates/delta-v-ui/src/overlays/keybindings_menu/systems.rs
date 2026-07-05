// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Keybindings menu systems.

use bevy::prelude::*;
use delta_v_core::I18n;
use delta_v_core::input::KeybindingsResource;
use delta_v_types::LogicalAction;
use leafwing_input_manager::prelude::ActionState;

use super::components::KeybindingsMenuRoot;
use super::resources::KeybindingsMenuOpen;
use super::spawn::spawn_keybindings_menu;
use crate::window::UiTheme;

/// Toggles the keybindings menu open/closed when the player presses the key.
///
/// Runs in `Update` during `AppState::InGame`.
/// Checks for `ToggleKeybindingsMenu` action via `InputMap`.
/// When opening: spawns the menu UI.
/// When closing: despawns the menu entity.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub fn keybindings_menu_toggle_system(
    mut commands: Commands<'_, '_>,
    action_state: Res<'_, ActionState<LogicalAction>>,
    mut menu_open: ResMut<'_, KeybindingsMenuOpen>,
    query: Query<'_, '_, Entity, With<KeybindingsMenuRoot>>,
    i18n: Res<'_, I18n>,
    keybindings: Res<'_, KeybindingsResource>,
    asset_server: Res<'_, AssetServer>,
    theme: Res<'_, UiTheme>,
) {
    // Check if ToggleKeybindingsMenu action is pressed
    let toggle_pressed = action_state.just_pressed(&LogicalAction::ToggleKeybindingsMenu);

    if !toggle_pressed {
        return;
    }

    if menu_open.0 {
        // Close the menu: despawn the root entity
        if let Ok(entity) = query.single() {
            commands.entity(entity).despawn();
        }
        menu_open.0 = false;
        tracing::debug!("keybindings menu: closed");
    } else {
        // Open the menu: spawn the menu UI
        spawn_keybindings_menu(&mut commands, &i18n, &keybindings, &asset_server, &theme);
        menu_open.0 = true;
        tracing::debug!("keybindings menu: opened");
    }
}
