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

//! Keybindings menu spawning functions.

use bevy::prelude::*;
use bevy::ui::GridTrack;
use delta_v_core::I18n;
use delta_v_core::input::KeybindingsResource;
use delta_v_types::LogicalAction;

use super::components::KeybindingsMenuRoot;
use crate::layout::spawn_group_header;
use crate::window::{UiTheme, WindowConfig, spawn_window};

// ============================================================================
// Bevy UI-based keybindings menu
// ============================================================================

/// Spawns the keybindings menu using Bevy UI.
///
/// Displays a semi-transparent window centered on screen.
/// The title and hint are positioned side by side in one row.
/// Press F1 to toggle visibility.
pub fn spawn_keybindings_menu(
    commands: &mut Commands<'_, '_>,
    i18n: &I18n,
    keybindings: &KeybindingsResource,
    asset_server: &Res<'_, AssetServer>,
    theme: &Res<'_, UiTheme>,
) {
    let title = i18n.ui.menu.keybindings.title.clone();
    let hint = i18n.ui.menu.keybindings.close.clone();

    let menu_root = spawn_window(
        commands,
        &WindowConfig {
            title,
            hint,
            ..default()
        },
        asset_server,
        theme,
        |ui| keybindings_menu_content(ui, i18n, keybindings, theme),
    );

    // Add the keybindings-specific marker so the toggle system can find/despawn this entity.
    commands.entity(menu_root).insert(KeybindingsMenuRoot);

    tracing::debug!("spawned keybindings menu root entity {menu_root:?}");
}

/// Recursively despawns the keybindings menu entity and all its children.
pub fn despawn_keybindings_menu(commands: &mut Commands<'_, '_>, entity: Entity) {
    commands.entity(entity).despawn();
}

/// Spawns a single keybinding row with label and key text.
fn spawn_keybinding_row(
    grid: &mut ChildSpawnerCommands<'_>,
    i18n: &I18n,
    keybindings: &KeybindingsResource,
    theme: &Res<'_, UiTheme>,
    action: LogicalAction,
) {
    let action_name = action.as_str();

    // Translated action display name.
    let action_display = i18n
        .ui
        .menu
        .keybindings
        .action
        .get(action_name)
        .map_or_else(|| action_name.to_owned(), Clone::clone);

    // Look up bound keys and translate each one.
    let key_text = keybindings
        .0
        .get(action_name)
        .map(|bindings| {
            bindings
                .keyboard
                .iter()
                .map(|key_name| {
                    i18n.ui
                        .menu
                        .keybindings
                        .key
                        .get(key_name)
                        .map_or_else(|| key_name.clone(), Clone::clone)
                })
                .collect::<Vec<_>>()
                .join(" + ")
        })
        .unwrap_or_default();

    // Label in column 1 (takes remaining space, wraps)
    grid.spawn((
        Name::new(format!("Label_{action_name}")),
        Node {
            grid_row: GridPlacement::auto(),
            ..default()
        },
    ))
    .with_children(|label_col| {
        label_col.spawn((
            Name::new(format!("LabelText_{action_name}")),
            Text::new(&action_display),
            TextFont {
                font: theme.font.clone(),
                font_size: UiTheme::TEXT_FONT_SIZE,
                ..default()
            },
            TextColor(UiTheme::LABEL_COLOR),
            // Text wraps within the column
            TextLayout::default(),
        ));
    });

    // Value in column 2 (sizes to content, stays on one line)
    grid.spawn((
        Name::new(format!("Value_{action_name}")),
        Node {
            grid_row: GridPlacement::auto(),
            ..default()
        },
    ))
    .with_children(|value_col| {
        value_col.spawn((
            Name::new(format!("ValueText_{action_name}")),
            Text::new(&key_text),
            TextFont {
                font: theme.font.clone(),
                font_size: UiTheme::TEXT_FONT_SIZE,
                ..default()
            },
            TextColor(UiTheme::VALUE_COLOR),
            TextLayout {
                linebreak: LineBreak::NoWrap,
                ..default()
            },
        ));
    });
}

/// Spawns the keybindings menu content inside the window content area.
///
/// Displays all current keybindings grouped by category (flight, combat, systems).
/// Each entry shows the translated action name and the translated key name(s).
///
/// Uses a CSS Grid layout for i18n-safe label/value pairs:
/// - Column 1 (1fr): Label column — text wraps within the column
/// - Column 2 (auto): Value column — sizes to content, stays on one line
///
/// Group headers span both columns and use the window's default styling.
fn keybindings_menu_content(
    ui: &mut ChildSpawnerCommands<'_>,
    i18n: &I18n,
    keybindings: &KeybindingsResource,
    theme: &Res<'_, UiTheme>,
) {
    // Group definitions: (group_key, actions in group)
    // The group key maps to `i18n.ui.menu.keybindings.group.<group_key>`.
    let groups: &[(&str, &[LogicalAction])] = &[
        (
            "flight",
            &[
                LogicalAction::ThrustForward,
                LogicalAction::ThrustBackward,
                LogicalAction::PitchUp,
                LogicalAction::PitchDown,
                LogicalAction::YawLeft,
                LogicalAction::YawRight,
                LogicalAction::RollLeft,
                LogicalAction::RollRight,
                LogicalAction::StrafeLeft,
                LogicalAction::StrafeRight,
                LogicalAction::StrafeUp,
                LogicalAction::StrafeDown,
            ],
        ),
        ("combat", &[LogicalAction::FirePrimary]),
        (
            "systems",
            &[
                LogicalAction::ToggleFlightAssist,
                LogicalAction::CockpitCycleNext,
                LogicalAction::CockpitCyclePrev,
                LogicalAction::CameraSwitchNext,
                LogicalAction::CameraSwitchPrev,
            ],
        ),
    ];

    // Spawn the grid container with label/value rows as children
    ui.spawn((
        Name::new("LabelValueGrid"),
        Node {
            width: Val::Percent(100.0),
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::fr(1.0), GridTrack::auto()],
            grid_auto_rows: vec![GridTrack::auto()],
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(4.0),
            ..default()
        },
    ))
    .with_children(|grid| {
        for (group_key, actions) in groups {
            // Look up the translated group name, fall back to uppercase key.
            let group_name = i18n
                .ui
                .menu
                .keybindings
                .group
                .get(*group_key)
                .map_or_else(|| group_key.to_uppercase(), Clone::clone);

            // -- Group header --
            // Spans both columns of the grid
            spawn_group_header(grid, theme, &group_name);

            // -- Action rows --
            for action in *actions {
                spawn_keybinding_row(grid, i18n, keybindings, theme, *action);
            }
        }
    });
}
