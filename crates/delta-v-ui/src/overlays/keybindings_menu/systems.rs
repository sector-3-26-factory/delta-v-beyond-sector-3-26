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
use delta_v_core::{I18n, KeybindingsResource};

use super::components::KeybindingsMenuRoot;
use super::resources::KeybindingsMenuOpen;
use super::spawn::spawn_keybindings_menu;

/// Toggles the keybindings menu open/closed when the player presses the key.
///
/// Runs in `Update` during `AppState::InGame`.
/// Checks for `F1` key press.
/// When opening: spawns the menu UI.
/// When closing: despawns the menu entity.
#[allow(clippy::needless_pass_by_value)]
pub fn keybindings_menu_toggle_system(
    mut commands: Commands<'_, '_>,
    keyboard: Res<'_, ButtonInput<KeyCode>>,
    mut menu_open: ResMut<'_, KeybindingsMenuOpen>,
    query: Query<'_, '_, Entity, With<KeybindingsMenuRoot>>,
    i18n: Res<'_, I18n>,
    keybindings: Res<'_, KeybindingsResource>,
    asset_server: Res<'_, AssetServer>,
) {
    // Check if F1 is pressed
    let f1_pressed = keyboard.just_pressed(KeyCode::F1);

    if !f1_pressed {
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
        spawn_keybindings_menu(&mut commands, &i18n, &keybindings, &asset_server);
        menu_open.0 = true;
        tracing::debug!("keybindings menu: opened");
    }
}
