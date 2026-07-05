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
use bevy::ui::{GridPlacement, GridTrack};

use super::components::NavigationMenuRoot;
use crate::window::{UiTheme, WindowConfig, spawn_window};

/// Spawns the navigation menu window.
///
/// Displays a list of navigable entities with type, ID, and distance.
/// Key `N` toggles visibility.
#[allow(clippy::too_many_arguments)]
pub fn spawn_navigation_menu(
    commands: &mut Commands<'_, '_>,
    asset_server: &Res<'_, AssetServer>,
    theme: &Res<'_, UiTheme>,
    i18n: &delta_v_core::I18n,
    title: &str,
    hint: &str,
    list_data: &[delta_v_core::NavEntry],
    selected_target: Option<Entity>,
    selected_nav_object: Option<Entity>,
    targeting_mode: delta_v_core::navigation::TargetingModeType,
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
        |ui| {
            navigation_menu_content(
                ui,
                theme,
                i18n,
                list_data,
                selected_target,
                selected_nav_object,
                targeting_mode,
            );
        },
    );

    // Add the navigation menu-specific marker so the toggle system can find/despawn this entity.
    commands.entity(menu_root).insert(NavigationMenuRoot);

    tracing::debug!("spawned navigation menu root entity {menu_root:?}");
}

/// Spawns the navigation menu content inside the window.
///
/// Displays a scrollable 3-column grid of navigable/targetable entities:
/// - Column 1: Entity type (i18n'd)
/// - Column 2: Entity ID
/// - Column 3: Distance from player
#[allow(clippy::too_many_arguments)]
fn navigation_menu_content(
    ui: &mut ChildSpawnerCommands<'_>,
    theme: &Res<'_, UiTheme>,
    i18n: &delta_v_core::I18n,
    list_data: &[delta_v_core::NavEntry],
    selected_target: Option<Entity>,
    selected_nav_object: Option<Entity>,
    targeting_mode: delta_v_core::navigation::TargetingModeType,
) {
    // Determine which entity is currently selected based on targeting mode
    let selected_entity = match targeting_mode {
        delta_v_core::navigation::TargetingModeType::Combat => selected_target,
        delta_v_core::navigation::TargetingModeType::Nav => selected_nav_object,
    };

    // Spawn the grid container with 3 columns
    ui.spawn((
        Name::new("NavListGrid"),
        Node {
            width: Val::Percent(100.0),
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::fr(1.0), GridTrack::fr(1.0), GridTrack::fr(1.0)],
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(4.0),
            ..default()
        },
    ))
    .with_children(|grid| {
        // Spawn entries
        for (index, entry) in list_data.iter().enumerate() {
            let distance_str = if entry.distance < 1000.0 {
                format!("{:.0} m", entry.distance)
            } else {
                format!("{:.1} km", entry.distance / 1000.0)
            };

            // Get i18n'd entity type, fallback to the raw type string
            let entity_type_str = match entry.entity_type.as_str() {
                "ship" => i18n.entity_types.ship.clone(),
                "asteroid" => i18n.entity_types.asteroid.clone(),
                "station" => i18n.entity_types.station.clone(),
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

            // Type column
            grid.spawn((
                Name::new(format!("Type_{index}")),
                Node {
                    grid_column: GridPlacement::start(1),
                    ..default()
                },
                BackgroundColor(row_bg_color),
                Text::new(&entity_type_str),
                TextFont {
                    font: theme.font.clone(),
                    font_size: UiTheme::TEXT_FONT_SIZE,
                    ..default()
                },
                TextColor(UiTheme::LABEL_COLOR),
            ));

            // ID column
            grid.spawn((
                Name::new(format!("Id_{index}")),
                Node {
                    grid_column: GridPlacement::start(2),
                    ..default()
                },
                BackgroundColor(row_bg_color),
                Text::new(&entry.entity_id),
                TextFont {
                    font: theme.font.clone(),
                    font_size: UiTheme::TEXT_FONT_SIZE,
                    ..default()
                },
                TextColor(UiTheme::VALUE_COLOR),
            ));

            // Distance column
            grid.spawn((
                Name::new(format!("Distance_{index}")),
                Node {
                    grid_column: GridPlacement::start(3),
                    ..default()
                },
                BackgroundColor(row_bg_color),
                Text::new(&distance_str),
                TextFont {
                    font: theme.font.clone(),
                    font_size: UiTheme::TEXT_FONT_SIZE,
                    ..default()
                },
                TextColor(UiTheme::HINT_COLOR),
            ));
        }

        // If no entries, show a placeholder
        if list_data.is_empty() {
            grid.spawn((
                Name::new("EmptyList"),
                Text::new(i18n.ui.menu.navigation.empty_list.clone()),
                TextFont {
                    font: theme.font.clone(),
                    font_size: UiTheme::TEXT_FONT_SIZE,
                    ..default()
                },
                TextColor(UiTheme::HINT_COLOR),
            ));
        }
    });
}

/// Recursively despawns the navigation menu entity and all its children.
pub fn despawn_navigation_menu(commands: &mut Commands<'_, '_>, entity: Entity) {
    commands.entity(entity).despawn();
}
