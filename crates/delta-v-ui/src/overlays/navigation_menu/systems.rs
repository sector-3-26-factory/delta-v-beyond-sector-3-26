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

//! Navigation menu systems.

use bevy::prelude::*;
use delta_v_core::I18n;

use super::components::NavigationMenuRoot;
use super::resources::NavigationMenuOpen;
use super::spawn::spawn_navigation_menu;
use crate::window::UiTheme;

/// Toggles the navigation menu open/closed when the player presses the key.
///
/// Runs in `Update` during `AppState::InGame`.
/// Checks for `N` key press.
/// When opening: spawns the menu UI.
/// When closing: despawns the menu entity.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub fn navigation_menu_toggle_system(
    mut commands: Commands<'_, '_>,
    keyboard: Res<'_, ButtonInput<KeyCode>>,
    mut menu_open: ResMut<'_, NavigationMenuOpen>,
    query: Query<'_, '_, Entity, With<NavigationMenuRoot>>,
    asset_server: Res<'_, AssetServer>,
    theme: Res<'_, UiTheme>,
    i18n: Res<'_, I18n>,
    list_data: Res<'_, delta_v_core::NavigationListData>,
) {
    // Check if N is pressed
    let n_pressed = keyboard.just_pressed(KeyCode::KeyN);

    if !n_pressed {
        return;
    }

    if menu_open.0 {
        // Close the menu: despawn the root entity
        if let Ok(entity) = query.single() {
            commands.entity(entity).despawn();
        }
        menu_open.0 = false;
        tracing::debug!("navigation menu: closed");
    } else {
        // Open the menu: spawn the menu UI
        let title = &i18n.ui.menu.navigation.title;
        let hint = &i18n.ui.menu.navigation.close;
        spawn_navigation_menu(
            &mut commands,
            &asset_server,
            &theme,
            &i18n,
            title,
            hint,
            &list_data.entries,
        );
        menu_open.0 = true;
        tracing::debug!("navigation menu: opened");
    }
}
