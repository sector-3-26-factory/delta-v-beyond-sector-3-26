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
// MERCHANTABILITY OR FITNESS FOR A PARTICULAR PURPOSE.  See the
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
use tracing::info_span;

use super::components::{
    NavMenuDistanceText, NavMenuRowBackground, NavMenuRowEntity, NavigationMenuRoot,
};
use super::distance_format::format_distance;
use super::resources::{NavigationMenuOpen, NavigationMenuToggleState};
use super::spawn::spawn_navigation_menu;
use crate::window::UiTheme;

/// Toggles the navigation menu open/closed when the player presses the key.
///
/// Runs in `PreUpdate` during `AppState::InGame`.
/// Checks for `ToggleNavigationMenu` action via `InputMap`.
/// When opening: spawns the menu UI.
/// When closing: despawns the menu entity.
/// Uses custom edge detection via `NavigationMenuToggleState` for more reliable
/// detection when many keys are held simultaneously.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub fn navigation_menu_toggle_system(
    mut commands: Commands<'_, '_>,
    action_state: Single<'_, '_, &ActionState<LogicalAction>>,
    mut menu_open: ResMut<'_, NavigationMenuOpen>,
    mut toggle_state: ResMut<'_, NavigationMenuToggleState>,
    query: Query<'_, '_, Entity, With<NavigationMenuRoot>>,
    asset_server: Res<'_, AssetServer>,
    theme: Res<'_, UiTheme>,
    i18n: Res<'_, I18n>,
    list_data: Res<'_, delta_v_core::NavigationListData>,
    selected_target: Res<'_, delta_v_core::navigation::SelectedTarget>,
    selected_nav_object: Res<'_, delta_v_core::navigation::SelectedNavObject>,
    targeting_mode: Res<'_, delta_v_core::navigation::TargetingMode>,
) {
    let _span = info_span!("delta_v_ui::navigation_menu_toggle_system").entered();
    // Check if ToggleNavigationMenu action is pressed
    let toggle_pressed = action_state.pressed(&LogicalAction::ToggleNavigationMenu);

    // Custom edge detection: only trigger on the frame the key transitions from not-pressed to pressed
    let toggle_just_pressed = toggle_pressed && !toggle_state.prev_pressed;

    tracing::debug!(
        "[navigation_menu_toggle] pressed={} prev_pressed={} just_pressed={} menu_open={}",
        toggle_pressed,
        toggle_state.prev_pressed,
        toggle_just_pressed,
        menu_open.0
    );

    // Update previous state for next frame
    toggle_state.prev_pressed = toggle_pressed;

    if !toggle_just_pressed {
        return;
    }

    if menu_open.0 {
        // Close the menu: despawn the root entity
        if let Ok(entity) = query.single() {
            commands.entity(entity).despawn();
            tracing::debug!(
                "[navigation_menu_toggle] despawned menu entity {:?}",
                entity
            );
        } else {
            tracing::warn!(
                "[navigation_menu_toggle] menu_open=true but no NavigationMenuRoot entity found!"
            );
        }
        menu_open.0 = false;
        tracing::debug!("[navigation_menu_toggle] navigation menu: closed");
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
        tracing::debug!("[navigation_menu_toggle] navigation menu: opened");
    }
}

/// Updates the selection highlight in the navigation menu every frame when open.
///
/// Runs in `Update` during `AppState::InGame`.
/// Reads the current selection from `SelectedTarget`/`SelectedNavObject` resources
/// and updates the background color of the selected row directly without
/// despawning the entire menu. Does not consume events.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub fn navigation_menu_selection_refresh_system(
    menu_open: Res<'_, NavigationMenuOpen>,
    list_data: Res<'_, delta_v_core::NavigationListData>,
    selected_target: Res<'_, delta_v_core::navigation::SelectedTarget>,
    selected_nav_object: Res<'_, delta_v_core::navigation::SelectedNavObject>,
    targeting_mode: Res<'_, delta_v_core::navigation::TargetingMode>,
    mut row_bg_query: Query<
        '_,
        '_,
        (
            &mut BackgroundColor,
            &NavMenuRowBackground,
            &NavMenuRowEntity,
        ),
    >,
) {
    let _span = info_span!("delta_v_ui::navigation_menu_selection_refresh_system").entered();
    // Only update if menu is open
    if !menu_open.0 {
        return;
    }

    // Determine which entity is currently selected based on targeting mode
    let selected_entity = match targeting_mode.mode {
        delta_v_core::navigation::TargetingModeType::Combat => selected_target.0,
        delta_v_core::navigation::TargetingModeType::Nav => selected_nav_object.0,
    };

    // Update background color for all row background entities
    for (mut bg_color, marker, _row_entity) in &mut row_bg_query {
        let is_selected = selected_entity.is_some_and(|e| {
            list_data
                .entries
                .get(marker.index)
                .is_some_and(|entry| entry.entity == e)
        });
        bg_color.0 = if is_selected {
            UiTheme::SELECTED_ROW_COLOR
        } else {
            Color::NONE
        };
    }
}

/// Refreshes the navigation menu content when the navigation list changes.
///
/// Runs in `Update` during `AppState::InGame`.
/// Processes `NavigationListChanged` events and updates the menu content
/// by comparing the new entry list with the currently rendered rows.
/// Only rebuilds rows when the set or order actually changes, avoiding
/// the 60 Hz despawn/re-spawn storm that broke fade animations.
#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    clippy::type_complexity
)]
pub fn navigation_menu_refresh_system(
    _commands: Commands<'_, '_>,
    menu_open: ResMut<'_, NavigationMenuOpen>,
    query: Query<'_, '_, Entity, With<NavigationMenuRoot>>,
    _asset_server: Res<'_, AssetServer>,
    _theme: Res<'_, UiTheme>,
    i18n: Res<'_, I18n>,
    list_data: Res<'_, delta_v_core::NavigationListData>,
    selected_target: Res<'_, delta_v_core::navigation::SelectedTarget>,
    selected_nav_object: Res<'_, delta_v_core::navigation::SelectedNavObject>,
    targeting_mode: Res<'_, delta_v_core::navigation::TargetingMode>,
    mut events: MessageReader<'_, '_, NavigationListChanged>,
    mut row_bg_query: Query<'_, '_, (Entity, &mut BackgroundColor, &NavMenuRowBackground)>,
    mut distance_text_query: Query<'_, '_, (Entity, &mut Text, &NavMenuDistanceText)>,
    mut text_queries: ParamSet<
        '_,
        '_,
        (
            Query<
                '_,
                '_,
                (Entity, &mut Text, &NavMenuRowBackground, &NavMenuRowEntity),
                Without<NavMenuDistanceText>,
            >,
            Query<
                '_,
                '_,
                (Entity, &mut Text, &NavMenuRowBackground, &NavMenuRowEntity),
                Without<NavMenuDistanceText>,
            >,
        ),
    >,
) {
    let _span = info_span!("delta_v_ui::navigation_menu_refresh_system").entered();

    // Only process if there's a navigation list change event
    let Some(_event) = events.read().next() else {
        return;
    };

    tracing::debug!(
        "[navigation_menu] received NavigationListChanged event, menu_open={}",
        menu_open.0
    );

    // Menu is closed - nothing to update
    if !menu_open.0 {
        tracing::debug!(
            "[navigation_menu] navigation list change event received but menu is closed"
        );
        return;
    }

    // Get the grid entity (child of NavigationMenuRoot)
    let Ok(_menu_root) = query.single() else {
        tracing::warn!("[navigation_menu] no NavigationMenuRoot entity found");
        return;
    };

    // Collect current row entities and their indices from the UI
    let mut current_rows: std::collections::HashMap<usize, Entity> =
        std::collections::HashMap::new();
    for (entity, _, marker) in row_bg_query.iter() {
        current_rows.insert(marker.index, entity);
    }

    // Build a map of new entries by index
    let new_entries = &list_data.entries;

    // Determine which entity is currently selected based on targeting mode
    let selected_entity = match targeting_mode.mode {
        delta_v_core::navigation::TargetingModeType::Combat => selected_target.0,
        delta_v_core::navigation::TargetingModeType::Nav => selected_nav_object.0,
    };

    // Update existing rows and track which indices are still present
    let mut seen_indices = std::collections::HashSet::new();

    for (index, entry) in new_entries.iter().enumerate() {
        seen_indices.insert(index);

        let distance_str = format_distance(entry.distance);

        // Get i18n'd entity type
        let entity_type_str = match entry.entity_type.as_str() {
            "ship" => i18n.entity_types.ship.clone(),
            "asteroid" => i18n.entity_types.asteroid.clone(),
            "station" => i18n.entity_types.station.clone(),
            "sun" => i18n.entity_types.sun.clone(),
            "planet" => i18n.entity_types.planet.clone(),
            "moon" => i18n.entity_types.moon.clone(),
            _ => entry.entity_type.clone(),
        };

        // Check if this entry is the selected one
        let is_selected = selected_entity.is_some_and(|e| e == entry.entity);

        // Background color for selected row
        let row_bg_color = if is_selected {
            UiTheme::SELECTED_ROW_COLOR
        } else {
            Color::NONE
        };

        // Update type column
        if let Some((_entity, mut text, _, _)) = text_queries
            .p0()
            .iter_mut()
            .find(|(_, _, m, _)| m.index == index)
        {
            text.0 = entity_type_str;
            if let Some((_entity, mut bg, _)) =
                row_bg_query.iter_mut().find(|(_, _, m)| m.index == index)
            {
                bg.0 = row_bg_color;
            }
        }

        // Update name column (display_name)
        if let Some((_entity, mut text, _, _)) = text_queries
            .p1()
            .iter_mut()
            .find(|(_, _, m, _)| m.index == index)
        {
            text.0.clone_from(&entry.display_name);
        }

        // Update distance column
        if let Some((_entity, mut text, _)) = distance_text_query
            .iter_mut()
            .find(|(_, _, m)| m.index == index)
        {
            text.0 = distance_str;
        }
    }

    // Remove rows for indices that no longer exist
    for index in current_rows.keys() {
        if !seen_indices.contains(index) {
            // The row entities will be cleaned up when the menu is closed
            // For now, we just clear their content
            if let Some((_entity, mut text, _, _)) = text_queries
                .p0()
                .iter_mut()
                .find(|(_, _, m, _)| m.index == *index)
            {
                text.0.clear();
            }
            if let Some((_entity, mut text, _, _)) = text_queries
                .p1()
                .iter_mut()
                .find(|(_, _, m, _)| m.index == *index)
            {
                text.0.clear();
            }
            if let Some((_entity, mut text, _)) = distance_text_query
                .iter_mut()
                .find(|(_, _, m)| m.index == *index)
            {
                text.0.clear();
            }
        }
    }

    tracing::debug!(
        "[navigation_menu] updated {} rows in place",
        new_entries.len()
    );
}

/// Updates the distance text in the navigation menu every frame.
///
/// Runs in `Update` during `AppState::InGame`.
/// Only runs when the menu is open. Updates the distance text for each
/// entry by reading the current `NavigationListData` and updating the
/// corresponding `NavMenuDistanceText` entities.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub fn update_navigation_menu_distances_system(
    menu_open: Res<'_, NavigationMenuOpen>,
    list_data: Res<'_, delta_v_core::NavigationListData>,
    mut distance_text_query: Query<'_, '_, (&mut Text, &NavMenuDistanceText)>,
) {
    let _span = info_span!("delta_v_ui::update_navigation_menu_distances_system").entered();
    // Only update if menu is open
    if !menu_open.0 {
        return;
    }

    for (mut text, marker) in &mut distance_text_query {
        if let Some(entry) = list_data.entries.get(marker.index) {
            text.0 = format_distance(entry.distance);
        }
    }
}

/// Handles click interactions on navigation menu rows.
///
/// Runs in `Update` during `AppState::InGame`.
/// When a row is clicked, emits a `TargetSelected` event with the entity
/// associated with that row. The row entities have `NavMenuRowEntity` components
/// that store the target entity.
#[allow(clippy::needless_pass_by_value)]
pub fn navigation_menu_click_system(
    menu_open: Res<'_, NavigationMenuOpen>,
    mut interaction_query: Query<
        '_,
        '_,
        (&Interaction, &NavMenuRowEntity, &NavMenuRowBackground),
        Changed<Interaction>,
    >,
    mut events: MessageWriter<'_, TargetSelected>,
    targeting_mode: Res<'_, delta_v_core::navigation::TargetingMode>,
) {
    let _span = info_span!("delta_v_ui::navigation_menu_click_system").entered();
    // Only process clicks if menu is open
    if !menu_open.0 {
        return;
    }

    for (interaction, row_entity, row_bg) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            let target_entity = row_entity.entity;
            let mode = targeting_mode.mode;

            tracing::debug!(
                "[navigation_menu] row clicked: index={} entity={:?} mode={:?}",
                row_bg.index,
                target_entity,
                mode
            );

            // Emit TargetSelected event
            events.write(TargetSelected {
                target: target_entity,
                mode,
            });
        }
    }
}
