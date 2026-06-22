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
use bevy::ui::{Overflow, ScrollPosition, ZIndex};

use super::components::{BACKGROUND_COLOR, FADE_ZONE_HEIGHT, WindowScrollContainer};
use super::fade_images::{create_fade_bottom_image, create_fade_top_image};

/// Render layer for window UI elements.
pub(crate) const RENDER_LAYER: usize = 2;

/// Configuration for a window entity.
pub struct WindowConfig {
    /// Title text displayed in the header row (left side).
    pub title: String,
    /// Hint text displayed in the header row (right side).
    pub hint: String,
    /// Window size in pixels.
    pub size: Vec2,
}

// allow-default: WindowConfig is a pure Rust UI helper struct, not JSON-backed.
// ADR-0013/ADR-0039 Default ban applies only to JSON-deserialized types.
impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            hint: String::new(),
            size: Vec2::new(300.0, 200.0),
        }
    }
}

/// Spawns a generic window with a title bar, hint, and content area.
///
/// Uses pure Bevy UI (`Node` + `Text` + `ScrollPosition`) for layout and clipping.
///
/// The window consists of:
/// - A semi-transparent black panel centered on screen.
/// - A header row with the title on the left and hint on the right.
/// - A content area that fills the remaining space below the header.
/// - Fade-out zones at the top and bottom of the content area.
/// - Mouse wheel scrolling for overflow content.
///
/// The `content_fn` callback is called with the content area's child spawner,
/// allowing the caller to spawn arbitrary content inside the window.
///
/// Returns the spawned root entity.
#[allow(clippy::too_many_lines)]
pub fn spawn_window(
    commands: &mut Commands<'_, '_>,
    config: &WindowConfig,
    asset_server: &Res<'_, AssetServer>,
    content_fn: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) -> Entity {
    // Load fade gradient images into the asset server
    let fade_top_image = asset_server.add(create_fade_top_image());
    let fade_bottom_image = asset_server.add(create_fade_bottom_image());

    let size = config.size;

    commands
        .spawn((
            Name::new("WindowRoot"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            RenderLayers::layer(RENDER_LAYER),
        ))
        .with_children(|ui| {
            // Main panel — semi-transparent black background
            ui.spawn((
                Name::new("Panel"),
                Node {
                    width: Val::Px(size.x),
                    height: Val::Px(size.y),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(BACKGROUND_COLOR),
                RenderLayers::layer(RENDER_LAYER),
            ))
            .with_children(|ui| {
                // Header row — 20px tall, fixed height
                ui.spawn((
                    Name::new("HeaderRow"),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(20.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(5.0)),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    RenderLayers::layer(RENDER_LAYER),
                ))
                .with_children(|ui| {
                    // Title text
                    ui.spawn((
                        Name::new("Title"),
                        Text::new(&config.title),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        RenderLayers::layer(RENDER_LAYER),
                    ));

                    // Hint text
                    ui.spawn((
                        Name::new("Hint"),
                        Text::new(&config.hint),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        RenderLayers::layer(RENDER_LAYER),
                    ));
                });

                // Content area — fills remaining space, clips overflow on X.
                // Y scrolling is handled by the inner ScrollContainer.
                ui.spawn((
                    Name::new("ContentContainer"),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(size.y - 20.0),
                        overflow: Overflow {
                            x: OverflowAxis::Clip,
                            y: OverflowAxis::Visible,
                        },
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    RenderLayers::layer(RENDER_LAYER),
                ))
                .with_children(|ui| {
                    // Scrollable content — handles Y scrolling via Bevy's layout system
                    ui.spawn((
                        Name::new("ScrollContainer"),
                        WindowScrollContainer,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            overflow: Overflow {
                                x: OverflowAxis::Clip,
                                y: OverflowAxis::Scroll,
                            },
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        ScrollPosition(Vec2::ZERO),
                        RenderLayers::layer(RENDER_LAYER),
                    ))
                    .with_children(|ui| {
                        // Top padding line
                        ui.spawn((
                            Name::new("PaddingTop"),
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(FADE_ZONE_HEIGHT),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            RenderLayers::layer(RENDER_LAYER),
                        ));

                        // Actual content from the caller
                        content_fn(ui);

                        // Bottom padding line
                        ui.spawn((
                            Name::new("PaddingBottom"),
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(FADE_ZONE_HEIGHT),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            RenderLayers::layer(RENDER_LAYER),
                        ));
                    });

                    // Fade-out zone: top — absolutely positioned, NOT affected by scroll
                    ui.spawn((
                        Name::new("FadeTop"),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(FADE_ZONE_HEIGHT),
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.0),
                            left: Val::Px(0.0),
                            ..default()
                        },
                        ZIndex(100),
                        ImageNode {
                            image: fade_top_image,
                            color: Color::WHITE,
                            ..default()
                        },
                        RenderLayers::layer(RENDER_LAYER),
                    ));

                    // Fade-out zone: bottom — absolutely positioned, NOT affected by scroll
                    ui.spawn((
                        Name::new("FadeBottom"),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(FADE_ZONE_HEIGHT),
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(0.0),
                            left: Val::Px(0.0),
                            ..default()
                        },
                        ZIndex(100),
                        ImageNode {
                            image: fade_bottom_image,
                            color: Color::WHITE,
                            ..default()
                        },
                        RenderLayers::layer(RENDER_LAYER),
                    ));
                });
            });
        })
        .id()
}
