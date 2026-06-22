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
use delta_v_core::{I18n, KeybindingsResource};
use delta_v_types::LogicalAction;

use super::components::KeybindingsMenuRoot;
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

/// Spawns the keybindings menu content inside the window content area.
///
/// Displays all current keybindings grouped by category (flight, combat, systems).
/// Each entry shows the translated action name and the translated key name(s).
///
/// Uses Bevy UI `Node` + `Text` pattern: each row is a `Node` with flex layout,
/// containing `Text` children for action name and key binding.
// This function is long because it manually constructs a multi-group, multi-row
// Bevy UI tree with repetitive per-element Node/Text boilerplate.
#[allow(clippy::too_many_lines)]
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
            ],
        ),
    ];

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
        ui.spawn((
            Name::new(format!("GroupHeader_{group_key}")),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|header| {
            header.spawn((
                Name::new(format!("GroupHeaderText_{group_key}")),
                Text::new(group_name),
                TextFont {
                    font: theme.font.clone(),
                    font_size: UiTheme::GROUP_HEADER_FONT_SIZE,
                    ..default()
                },
                TextColor(UiTheme::GROUP_HEADER_COLOR),
                TextLayout {
                    linebreak: LineBreak::NoWrap,
                    ..default()
                },
            ));
        });

        // -- Action rows --
        for action in *actions {
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

            // Action row — two columns
            ui.spawn((
                Name::new(format!("ActionRow_{action_name}")),
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
            ))
            .with_children(|row| {
                // Action name (left column, 50% width)
                row.spawn((
                    Name::new(format!("Action_{action_name}")),
                    Node {
                        width: Val::Percent(50.0),
                        ..default()
                    },
                ))
                .with_children(|action_col| {
                    action_col.spawn((
                        Name::new(format!("ActionText_{action_name}")),
                        Text::new(action_display),
                        TextFont {
                            font: theme.font.clone(),
                            font_size: UiTheme::TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(UiTheme::LABEL_COLOR),
                        TextLayout {
                            linebreak: LineBreak::NoWrap,
                            ..default()
                        },
                    ));
                });

                // Key binding (right column, 50% width)
                row.spawn((
                    Name::new(format!("Key_{action_name}")),
                    Node {
                        width: Val::Percent(50.0),
                        ..default()
                    },
                ))
                .with_children(|key_col| {
                    key_col.spawn((
                        Name::new(format!("KeyText_{action_name}")),
                        Text::new(key_text),
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
            });
        }
    }
}
