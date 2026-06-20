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

use super::components::KeybindingsMenuRoot;

// ============================================================================
// Lunex-based keybindings menu
// ============================================================================

/// Spawns the keybindings menu using Lunex UI.
///
/// Displays a 300x200px semi-transparent window centered on screen.
/// The title and close hint are positioned side by side in one row.
/// Press F1 to toggle visibility.
#[allow(clippy::too_many_lines)]
pub fn spawn_keybindings_menu(
    commands: &mut Commands<'_, '_>,
    i18n: &I18n,
    _keybindings: &KeybindingsResource,
) {
    let title = i18n.ui.menu.keybindings.title.clone();
    let close_hint = i18n.ui.menu.keybindings.close.clone();

    // Create UI root with Lunex components
    let menu_root = commands
        .spawn((
            KeybindingsMenuRoot,
            UiLayoutRoot::new_2d(),
            UiFetchFromCamera::<2>,
            RenderLayers::layer(2),
        ))
        .with_children(|ui| {
            // Main panel: Artificially narrowed to 200px
            ui.spawn((
                Name::new("Panel"),
                UiLayout::window()
                    .pos(Rl((50.0, 50.0)))
                    .size((Ab(300.0), Ab(200.0)))
                    .anchor(Anchor::CENTER)
                    .pack(),
                Sprite {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.8),
                    ..default()
                },
                RenderLayers::layer(2),
            ))
            .with_children(|ui| {
                // Header row container: 20px height at position (5, 5)
                ui.spawn((
                    Name::new("HeaderRow"),
                    UiLayout::window()
                        .pos((Ab(5.0), Ab(5.0)))
                        .size((Rl(100.0) - Ab(10.0), Ab(20.0)))
                        .pack(),
                ))
                .with_children(|ui| {
                    // Title container: 50% width, full height of parent
                    ui.spawn((
                        Name::new("TitleContainer"),
                        UiLayout::window().size((Rl(50.0), Rh(100.0))).pack(),
                    ))
                    .with_children(|ui| {
                        // Title text: Stays compact, anchors LEFT
                        ui.spawn((
                            Name::new("Title"),
                            UiLayout::window()
                                .pos((Rl(0.0), Rl(50.0))) // Left wall (0%), vertical center (50%)
                                .anchor(Anchor::CENTER_LEFT) // Lunex anchor left
                                .pack(),
                            // Scales matching 80% of the TitleContainer's height.
                            // If the window shrinks heavily, Rh shrinks, forcing the text down.
                            UiTextSize::from(Rh(80.0)),
                            Text2d::new(&title),
                            TextLayout {
                                justify: Justify::Left,
                                ..default()
                            },
                            TextFont {
                                font_size: 20.0, // Fallback base size
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE,
                            RenderLayers::layer(2),
                        ));
                    });

                    // Close hint container: 50% width, full height of parent
                    ui.spawn((
                        Name::new("CloseHintContainer"),
                        UiLayout::window()
                            .pos((Rl(50.0), Ab(0.0)))
                            .size((Rl(50.0), Rh(100.0)))
                            .pack(),
                    ))
                    .with_children(|ui| {
                        // Close hint text: Stays compact, anchors RIGHT
                        ui.spawn((
                            Name::new("CloseHint"),
                            UiLayout::window()
                                .pos((Rl(100.0), Rl(50.0))) // Right wall (100%), vertical center (50%)
                                .anchor(Anchor::CENTER_RIGHT) // Lunex anchor right
                                .pack(),
                            // Slightly smaller scaling factor (60% of row height)
                            UiTextSize::from(Rh(60.0)),
                            Text2d::new(&close_hint),
                            TextLayout {
                                justify: Justify::Right,
                                ..default()
                            },
                            TextFont {
                                font_size: 14.0, // Fallback base size
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                            Pickable::IGNORE,
                            RenderLayers::layer(2),
                        ));
                    });
                });

                // Content container: fills remaining space below header
                ui.spawn((
                    Name::new("Content"),
                    UiLayout::window()
                        .pos((Ab(0.0), Ab(25.0)))
                        .size((Rl(100.0), Rl(100.0) - Ab(25.0)))
                        .pack(),
                ))
                .with_children(|ui| {
                    spawn_content_text(ui);
                });
            });
        })
        .id();

    tracing::debug!("spawned keybindings menu root entity {menu_root:?}");
}

/// Recursively despawns the keybindings menu entity and all its children.
pub fn despawn_keybindings_menu(commands: &mut Commands<'_, '_>, entity: Entity) {
    commands.entity(entity).despawn();
}

/// Spawns the content text entity inside the keybindings menu.
fn spawn_content_text(ui: &mut ChildSpawnerCommands<'_>) {
    ui.spawn((
        Name::new("ContentText"),
        UiLayout::window()
            .pos((Rl(0.0), Rl(0.0)))
            .anchor(Anchor::TOP_LEFT)
            .pack(),
        UiTextSize::from(Ab(16.0)),
        Text2d::new("hello world hallo welt hello world hallo welt "),
        TextLayout {
            justify: Justify::Left,
            linebreak: LineBreak::AnyCharacter,
        },
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Pickable::IGNORE,
        RenderLayers::layer(2),
    ));
}
