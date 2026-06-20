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

//! Window spawning functions.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_lunex::prelude::*;

use super::components::WindowRoot;

/// Configuration for a window entity.
pub struct WindowConfig {
    /// Title text displayed in the header row (left side).
    pub title: String,
    /// Hint text displayed in the header row (right side).
    pub hint: String,
    /// Window size in pixels.
    pub size: Vec2,
    /// Render layer for the window.
    pub render_layer: usize,
}

// allow-default: WindowConfig is a pure Rust UI helper struct, not JSON-backed.
// ADR-0013/ADR-0039 Default ban applies only to JSON-deserialized types.
impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            hint: String::new(),
            size: Vec2::new(300.0, 200.0),
            render_layer: 2,
        }
    }
}

/// Spawns a generic window with a title bar, hint, and content area.
///
/// The window consists of:
/// - A semi-transparent black panel centered on screen.
/// - A header row with the title on the left and hint on the right.
/// - A content area that fills the remaining space below the header.
///
/// The `content_fn` callback is called with the content area's child spawner,
/// allowing the caller to spawn arbitrary content inside the window.
///
/// Returns the spawned root entity.
#[allow(clippy::too_many_lines)]
pub fn spawn_window(
    commands: &mut Commands<'_, '_>,
    config: &WindowConfig,
    content_fn: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) -> Entity {
    commands
        .spawn((
            WindowRoot,
            UiLayoutRoot::new_2d(),
            UiFetchFromCamera::<2>,
            RenderLayers::layer(config.render_layer),
        ))
        .with_children(|ui| {
            // Main panel
            ui.spawn((
                Name::new("Panel"),
                UiLayout::window()
                    .pos(Rl((50.0, 50.0)))
                    .size((Ab(config.size.x), Ab(config.size.y)))
                    .anchor(Anchor::CENTER)
                    .pack(),
                Sprite {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.8),
                    ..default()
                },
                RenderLayers::layer(config.render_layer),
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
                                .pos((Rl(0.0), Rl(50.0)))
                                .anchor(Anchor::CENTER_LEFT)
                                .pack(),
                            UiTextSize::from(Rh(80.0)),
                            Text2d::new(&config.title),
                            TextLayout {
                                justify: Justify::Left,
                                ..default()
                            },
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE,
                            RenderLayers::layer(config.render_layer),
                        ));
                    });

                    // Hint container: 50% width, full height of parent
                    ui.spawn((
                        Name::new("HintContainer"),
                        UiLayout::window()
                            .pos((Rl(50.0), Ab(0.0)))
                            .size((Rl(50.0), Rh(100.0)))
                            .pack(),
                    ))
                    .with_children(|ui| {
                        // Hint text: Stays compact, anchors RIGHT
                        ui.spawn((
                            Name::new("Hint"),
                            UiLayout::window()
                                .pos((Rl(100.0), Rl(50.0)))
                                .anchor(Anchor::CENTER_RIGHT)
                                .pack(),
                            UiTextSize::from(Rh(60.0)),
                            Text2d::new(&config.hint),
                            TextLayout {
                                justify: Justify::Right,
                                ..default()
                            },
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                            Pickable::IGNORE,
                            RenderLayers::layer(config.render_layer),
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
                    content_fn(ui);
                });
            });
        })
        .id()
}
