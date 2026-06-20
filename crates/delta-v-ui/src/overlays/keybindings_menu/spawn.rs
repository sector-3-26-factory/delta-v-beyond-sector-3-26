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

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_lunex::prelude::*;
use delta_v_core::{I18n, KeybindingsResource};
use delta_v_types::LogicalAction;

use super::components::KeybindingsMenuRoot;
use crate::window::{WindowConfig, spawn_window};

// ============================================================================
// Lunex-based keybindings menu
// ============================================================================

/// Spawns the keybindings menu using Lunex UI.
///
/// Displays a 300x200px semi-transparent window centered on screen.
/// The title and hint are positioned side by side in one row.
/// Press F1 to toggle visibility.
pub fn spawn_keybindings_menu(
    commands: &mut Commands<'_, '_>,
    i18n: &I18n,
    keybindings: &KeybindingsResource,
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
        |ui| keybindings_menu_content(ui, i18n, keybindings),
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
/// Uses the Lunex container pattern from [`crate::window::spawn_window`]: each visual element is wrapped in a `UiLayout` container with a fixed size;
/// the text is spawned as a child with `.pack()` so it fills the container.
#[allow(clippy::too_many_lines)]
fn keybindings_menu_content(
    ui: &mut ChildSpawnerCommands<'_>,
    i18n: &I18n,
    keybindings: &KeybindingsResource,
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

    // Layout constants (in pixels / percentage units).
    let row_height = 18.0_f32;
    let header_offset = 4.0_f32;
    let group_gap = 8.0_f32;
    let action_col_pct = 50.0_f32; // percentage of width for action name column
    let action_col_pad = 4.0_f32; // left padding inside action column
    let key_col_start_pct = action_col_pct + 5.0; // percentage offset for key column
    let key_col_pad = 2.0_f32; // right padding inside key column

    // Track current vertical position as we stack rows.
    let mut current_y = header_offset;

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
            UiLayout::window()
                .pos((Rl(2.0), Ab(current_y)))
                .size((Rl(96.0), Ab(row_height)))
                .anchor(Anchor::TOP_LEFT)
                .pack(),
            RenderLayers::layer(2),
        ))
        .with_children(|header| {
            header.spawn((
                Name::new(format!("GroupHeaderText_{group_key}")),
                UiLayout::window().pack(),
                UiTextSize::from(Rh(80.0)),
                Text2d::new(group_name),
                TextLayout {
                    justify: Justify::Left,
                    linebreak: LineBreak::WordBoundary,
                },
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.0)),
                Pickable::IGNORE,
                RenderLayers::layer(2),
            ));
        });
        current_y += row_height;

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
                        .join(" / ")
                })
                .unwrap_or_default();

            // Action name container (left column).
            ui.spawn((
                Name::new(format!("Action_{action_name}")),
                UiLayout::window()
                    .pos((Rl(action_col_pad), Ab(current_y)))
                    .size((Rl(action_col_pct - action_col_pad), Ab(row_height)))
                    .anchor(Anchor::TOP_LEFT)
                    .pack(),
                RenderLayers::layer(2),
            ))
            .with_children(|action_row| {
                action_row.spawn((
                    Name::new(format!("ActionText_{action_name}")),
                    UiLayout::window().pack(),
                    UiTextSize::from(Rh(80.0)),
                    Text2d::new(action_display),
                    TextLayout {
                        justify: Justify::Left,
                        linebreak: LineBreak::WordBoundary,
                    },
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                    RenderLayers::layer(2),
                ));
            });

            // Key binding container (right column).
            ui.spawn((
                Name::new(format!("Key_{action_name}")),
                UiLayout::window()
                    .pos((Rl(key_col_start_pct), Ab(current_y)))
                    .size((Rl(100.0 - key_col_start_pct - key_col_pad), Ab(row_height)))
                    .anchor(Anchor::TOP_LEFT)
                    .pack(),
                RenderLayers::layer(2),
            ))
            .with_children(|key_row| {
                key_row.spawn((
                    Name::new(format!("KeyText_{action_name}")),
                    UiLayout::window().pack(),
                    UiTextSize::from(Rh(80.0)),
                    Text2d::new(key_text),
                    TextLayout {
                        justify: Justify::Left,
                        linebreak: LineBreak::WordBoundary,
                    },
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 1.0, 0.7)),
                    Pickable::IGNORE,
                    RenderLayers::layer(2),
                ));
            });

            current_y += row_height;
        }

        // Gap between groups.
        current_y += group_gap;
    }
}
