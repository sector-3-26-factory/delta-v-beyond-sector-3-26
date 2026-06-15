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

use std::collections::HashMap;

use bevy::prelude::*;
use delta_v_core::{I18n, KeybindingsResource};

use super::components::KeybindingsMenuRoot;

/// Spawns the keybindings menu UI.
///
/// Creates a full-screen semi-transparent background with:
/// - Title bar using i18n translation
/// - Close hint using i18n translation
/// - Scrollable content area with grouped keybinding entries
///
/// Each action is displayed with its translated name and the translated
/// key names for its bindings.
#[allow(clippy::too_many_lines)]
pub fn spawn_keybindings_menu(
    commands: &mut Commands<'_, '_>,
    i18n: &I18n,
    keybindings: &KeybindingsResource,
) {
    let title = i18n.ui.menu.keybindings.title.clone();
    let close_hint = i18n.ui.menu.keybindings.close.clone();
    let groups = i18n.ui.menu.keybindings.group.clone();
    let actions = i18n.ui.menu.keybindings.action.clone();
    let keys = i18n.ui.menu.keybindings.key.clone();

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                ..default()
            },
            KeybindingsMenuRoot,
        ))
        .with_children(|parent| {
            // Title bar at the top
            parent
                .spawn(TextBundle::from_section(
                    title,
                    TextStyle {
                        font_size: 24.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ))
                .insert(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                });

            // Close hint below title
            parent
                .spawn(TextBundle::from_section(
                    close_hint,
                    TextStyle {
                        font_size: 14.0,
                        color: Color::srgb(0.7, 0.7, 0.7),
                        ..default()
                    },
                ))
                .insert(Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                });

            // Content area with scrollable keybindings
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|content_parent| {
                    // Group actions by category
                    let mut grouped: HashMap<&str, Vec<(&String, &Vec<String>)>> = HashMap::new();
                    for (action_name, bindings) in &keybindings.0 {
                        let group_key = get_action_group(action_name);
                        grouped
                            .entry(group_key)
                            .or_default()
                            .push((action_name, &bindings.keyboard));
                    }

                    // Create a section for each group
                    for (group_key, actions_in_group) in &grouped {
                        // Group header
                        if let Some(group_name) = groups.get(*group_key) {
                            content_parent
                                .spawn(TextBundle::from_section(
                                    group_name.clone(),
                                    TextStyle {
                                        font_size: 18.0,
                                        color: Color::srgb(0.9, 0.9, 0.9),
                                        ..default()
                                    },
                                ))
                                .insert(Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(30.0),
                                    margin: UiRect::bottom(Val::Px(10.0)),
                                    ..default()
                                });
                        }

                        // Actions in this group
                        for (action_name, key_bindings) in actions_in_group {
                            // Get translated action name
                            let action_display = actions
                                .get(*action_name)
                                .cloned()
                                .unwrap_or_else(|| (*action_name).clone());

                            // Get translated key names
                            let key_names: Vec<String> = key_bindings
                                .iter()
                                .filter_map(|k| keys.get(k).cloned())
                                .collect();

                            let key_display = if key_names.is_empty() {
                                "None".to_string()
                            } else {
                                key_names.join(" + ")
                            };

                            // Create the keybinding entry row
                            content_parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(25.0),
                                        justify_content: JustifyContent::SpaceBetween,
                                        margin: UiRect::bottom(Val::Px(5.0)),
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|row| {
                                    row.spawn(TextBundle::from_section(
                                        action_display,
                                        TextStyle {
                                            font_size: 14.0,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    ));

                                    row.spawn(TextBundle::from_section(
                                        key_display,
                                        TextStyle {
                                            font_size: 14.0,
                                            color: Color::srgb(0.8, 0.8, 0.8),
                                            ..default()
                                        },
                                    ));
                                });
                        }
                    }
                });
        });
}

/// Determines the action group for a given action name.
///
/// Groups are based on the action name prefix:
/// - `thrust_*`, `pitch_*`, `yaw_*`, `roll_*`, `strafe_*` → `flight`
/// - `fire_*` → `combat`
/// - `toggle_*` → `systems`
/// - `cockpit_*` → `systems`
/// - Everything else → `other`
fn get_action_group(action_name: &str) -> &'static str {
    if action_name.starts_with("thrust_")
        || action_name.starts_with("pitch_")
        || action_name.starts_with("yaw_")
        || action_name.starts_with("roll_")
        || action_name.starts_with("strafe_")
    {
        "flight"
    } else if action_name.starts_with("fire_") {
        "combat"
    } else if action_name.starts_with("toggle_") || action_name.starts_with("cockpit_") {
        "systems"
    } else {
        "other"
    }
}

/// Recursively despawns the keybindings menu entity and all its children.
pub fn despawn_keybindings_menu(commands: &mut Commands<'_, '_>, entity: Entity) {
    commands.entity(entity).despawn_recursive();
}
