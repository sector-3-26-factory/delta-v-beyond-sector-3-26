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

use std::collections::BTreeMap;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_lunex::prelude::*;
use delta_v_core::{I18n, KeybindingsResource};

use super::components::KeybindingsMenuRoot;

// ============================================================================
// OLD IMPLEMENTATION – kept for reference / comparison
// ============================================================================

/// Original bevy_ui-based keybindings menu (preserved for comparison).
#[allow(clippy::too_many_lines)]
pub fn spawn_keybindings_menu_old(
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
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            KeybindingsMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ));

            parent.spawn((
                Text::new(close_hint),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));

            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|content_parent| {
                    let mut grouped: BTreeMap<&str, Vec<(&String, &Vec<String>)>> = BTreeMap::new();
                    for (action_name, bindings) in &keybindings.0 {
                        let group_key = get_action_group(action_name);
                        grouped
                            .entry(group_key)
                            .or_default()
                            .push((action_name, &bindings.keyboard));
                    }

                    for (group_key, actions_in_group) in &grouped {
                        if let Some(group_name) = groups.get(*group_key) {
                            content_parent.spawn((
                                Text::new(group_name.clone()),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(30.0),
                                    margin: UiRect::bottom(Val::Px(10.0)),
                                    ..default()
                                },
                            ));
                        }

                        for (action_name, key_bindings) in actions_in_group {
                            let action_display = actions
                                .get(*action_name)
                                .cloned()
                                .unwrap_or_else(|| (*action_name).clone());
                            let key_names: Vec<String> = key_bindings
                                .iter()
                                .filter_map(|k| keys.get(k).cloned())
                                .collect();
                            let key_display = if key_names.is_empty() {
                                "None".to_string()
                            } else {
                                key_names.join(" + ")
                            };

                            content_parent
                                .spawn(Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(25.0),
                                    justify_content: JustifyContent::SpaceBetween,
                                    margin: UiRect::bottom(Val::Px(5.0)),
                                    ..default()
                                })
                                .with_children(|row| {
                                    row.spawn((
                                        Text::new(action_display),
                                        TextFont {
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                    row.spawn((
                                        Text::new(key_display),
                                        TextFont {
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                                    ));
                                });
                        }
                    }
                });
        });
}

// ============================================================================
// NEW IMPLEMENTATION – Lunex retained-layout example
// ============================================================================

/// Spawns a Lunex UI example (button with text).
///
/// This is a 1:1 translation of the Lunex README button example:
/// <https://github.com/bytestring-net/bevy_lunex/blob/main/README.md>
///
/// Press F1 to see it rendered.
pub fn spawn_keybindings_menu(
    commands: &mut Commands<'_, '_>,
    _i18n: &I18n,
    _keybindings: &KeybindingsResource,
) {
    // Create UI root
    let menu_root = commands
        .spawn((
            KeybindingsMenuRoot,
            UiLayoutRoot::new_2d(),
            UiFetchFromCamera::<2>,
            RenderLayers::layer(2),
        ))
        .with_children(|ui| {
            // Spawn a button in the middle of the screen
            ui.spawn((
                Name::new("My Button"),
                // Specify the position and size of the button
                UiLayout::window()
                    .pos(Rl((50.0, 50.0)))
                    .size((200.0, 50.0))
                    .pack(),
                RenderLayers::layer(2),
            ))
            .with_children(|ui| {
                // Spawn a child node with a background
                ui.spawn((
                    // Fill the parent
                    UiLayout::window().full().pack(),
                    Sprite {
                        color: Color::srgba(0.8, 0.2, 0.2, 0.5),
                        ..default()
                    },
                    RenderLayers::layer(2),
                ))
                .with_children(|ui| {
                    // Spawn the text
                    ui.spawn((
                        // For text always use window layout to position it
                        UiLayout::window()
                            .pos((Rh(40.0), Rl(50.0)))
                            .anchor(Anchor::CENTER_LEFT)
                            .pack(),
                        // Text height proportional to the parent node
                        UiTextSize::from(Rh(60.0)),
                        Text2d::new("Click me!"),
                        TextFont {
                            font_size: 64.0,
                            ..default()
                        },
                        RenderLayers::layer(2),
                    ));
                });
            });
        })
        .id();

    log::debug!("spawned keybindings menu root entity {menu_root:?}");
}

// ============================================================================
// Helpers
// ============================================================================

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
    commands.entity(entity).despawn();
}
