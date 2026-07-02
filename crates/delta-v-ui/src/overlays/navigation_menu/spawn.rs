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

//! Navigation menu spawning functions.

use bevy::prelude::*;

use super::components::NavigationMenuRoot;
use crate::window::{UiTheme, WindowConfig, spawn_window};

/// Spawns the navigation menu window.
///
/// Displays a list of navigable entities with distance and bearing.
/// Key `N` toggles visibility.
pub fn spawn_navigation_menu(
    commands: &mut Commands<'_, '_>,
    asset_server: &Res<'_, AssetServer>,
    theme: &Res<'_, UiTheme>,
    title: &str,
    hint: &str,
) {
    let menu_root = spawn_window(
        commands,
        &WindowConfig {
            title: title.to_string(),
            hint: hint.to_string(),
            ..default()
        },
        asset_server,
        theme,
        |_ui| {}, // Content populated by navigation_list_update_system
    );

    // Add the navigation menu-specific marker so the toggle system can find/despawn this entity.
    commands.entity(menu_root).insert(NavigationMenuRoot);

    tracing::debug!("spawned navigation menu root entity {menu_root:?}");
}

/// Recursively despawns the navigation menu entity and all its children.
pub fn despawn_navigation_menu(commands: &mut Commands<'_, '_>, entity: Entity) {
    commands.entity(entity).despawn();
}
