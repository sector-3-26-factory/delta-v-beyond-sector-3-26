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
use delta_v_core::events::{NavigationListChanged, TargetSelected};
use delta_v_types::LogicalAction;
use leafwing_input_manager::prelude::ActionState;

use super::components::NavigationMenuRoot;
use super::resources::NavigationMenuOpen;
use super::spawn::spawn_navigation_menu;
use crate::window::UiTheme;

/// Toggles the navigation menu open/closed when the player presses the key.
///
/// Runs in `Update` during `AppState::InGame`.
/// Checks for `ToggleNavigationMenu` action via `InputMap`.
/// When opening: spawns the menu UI.
/// When closing: despawns the menu entity.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub fn navigation_menu_toggle_system(
    mut commands: Commands<'_, '_>,
    action_state: Res<'_, ActionState<LogicalAction>>,
    mut menu_open: ResMut<'_, NavigationMenuOpen>,
    query: Query<'_, '_, Entity, With<NavigationMenuRoot>>,
    asset_server: Res<'_, AssetServer>,
    theme: Res<'_, UiTheme>,
    i18n: Res<'_, I18n>,
    list_data: Res<'_, delta_v_core::NavigationListData>,
    selected_target: Res<'_, delta_v_core::navigation::SelectedTarget>,
    selected_nav_object: Res<'_, delta_v_core::navigation::SelectedNavObject>,
    targeting_mode: Res<'_, delta_v_core::navigation::TargetingMode>,
) {
    // Check if ToggleNavigationMenu action is pressed
    let toggle_pressed = action_state.just_pressed(&LogicalAction::ToggleNavigationMenu);

    if !toggle_pressed {
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
            selected_target.0,
            selected_nav_object.0,
            targeting_mode.mode,
        );
        menu_open.0 = true;
        tracing::debug!("navigation menu: opened");
    }
}

/// Refreshes the navigation menu when the selected target changes.
///
/// Runs in `Update` during `AppState::InGame`.
/// Processes `TargetSelected` events and re-spawns the menu
/// if it is currently open, ensuring the selection highlight is updated.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub fn navigation_menu_selection_refresh_system(
    mut commands: Commands<'_, '_>,
    mut menu_open: ResMut<'_, NavigationMenuOpen>,
    query: Query<'_, '_, Entity, With<NavigationMenuRoot>>,
    asset_server: Res<'_, AssetServer>,
    theme: Res<'_, UiTheme>,
    i18n: Res<'_, I18n>,
    list_data: Res<'_, delta_v_core::NavigationListData>,
    selected_target: Res<'_, delta_v_core::navigation::SelectedTarget>,
    selected_nav_object: Res<'_, delta_v_core::navigation::SelectedNavObject>,
    targeting_mode: Res<'_, delta_v_core::navigation::TargetingMode>,
    mut events: MessageReader<'_, '_, TargetSelected>,
) {
    // Only process if there's a target selection change event
    let Some(_event) = events.read().next() else {
        return;
    };

    tracing::debug!(
        "[navigation_menu] received TargetSelected event, menu_open={}",
        menu_open.0
    );

    // Menu is open - despawn and re-spawn with updated selection highlight
    if menu_open.0 {
        if let Ok(entity) = query.single() {
            commands.entity(entity).despawn();
        }
        menu_open.0 = false;

        // Re-open the menu with refreshed selection highlight
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
            selected_target.0,
            selected_nav_object.0,
            targeting_mode.mode,
        );
        menu_open.0 = true;
        tracing::debug!("[navigation_menu] refreshed after target selection change");
    } else {
        tracing::debug!(
            "[navigation_menu] target selection change event received but menu is closed"
        );
    }
}

/// Refreshes the navigation menu content when the navigation list changes.
///
/// Runs in `Update` during `AppState::InGame`.
/// Processes `NavigationListChanged` events and re-spawns the menu
/// if it is currently open, ensuring the content reflects the current list.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub fn navigation_menu_refresh_system(
    mut commands: Commands<'_, '_>,
    mut menu_open: ResMut<'_, NavigationMenuOpen>,
    query: Query<'_, '_, Entity, With<NavigationMenuRoot>>,
    asset_server: Res<'_, AssetServer>,
    theme: Res<'_, UiTheme>,
    i18n: Res<'_, I18n>,
    list_data: Res<'_, delta_v_core::NavigationListData>,
    selected_target: Res<'_, delta_v_core::navigation::SelectedTarget>,
    selected_nav_object: Res<'_, delta_v_core::navigation::SelectedNavObject>,
    targeting_mode: Res<'_, delta_v_core::navigation::TargetingMode>,
    mut events: MessageReader<'_, '_, NavigationListChanged>,
) {
    // Only process if there's a navigation list change event
    let Some(_event) = events.read().next() else {
        return;
    };

    tracing::debug!(
        "[navigation_menu] received NavigationListChanged event, menu_open={}",
        menu_open.0
    );

    // Menu is open - despawn and re-spawn with updated content
    if menu_open.0 {
        if let Ok(entity) = query.single() {
            commands.entity(entity).despawn();
        }
        menu_open.0 = false;

        // Re-open the menu with refreshed content
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
            selected_target.0,
            selected_nav_object.0,
            targeting_mode.mode,
        );
        menu_open.0 = true;
        tracing::debug!("[navigation_menu] refreshed after navigation list change");
    } else {
        tracing::debug!(
            "[navigation_menu] navigation list change event received but menu is closed"
        );
    }
}
