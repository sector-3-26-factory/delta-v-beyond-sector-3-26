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

use bevy::prelude::*;
use bevy::ui::{Overflow, ScrollPosition, ZIndex};
use delta_v_core::RenderLayer;

use super::animations::WindowAnimation;
use super::border::ScanLineDot;
use super::components::WindowBorder;
use super::components::WindowRoot;
use super::components::WindowScrollContainer;
use super::fade_images::{create_fade_bottom_image, create_fade_top_image};
use super::theme::UiTheme;

/// Configuration for a window entity.
pub struct WindowConfig {
    /// Title text displayed in the header row (left side).
    pub title: String,
    /// Hint text displayed in the header row (right side).
    pub hint: String,
    /// Window size in pixels.
    pub size: Vec2,
    /// Header row height. Use 0.0 for content-only windows (notifications).
    pub header_height: f32,
    /// Animation to apply to the window. Use `None` for no animation.
    /// When `Some`, the `WindowAnimation` component is automatically
    /// attached to the spawned window root entity.
    pub animation: Option<WindowAnimation>,
    /// Whether the content area should be scrollable.
    /// If `true`, a scroll container with fade zones is created.
    /// If `false`, content is placed directly without fade zones or scrolling.
    pub scrollable: bool,
}

// allow-default: WindowConfig is a pure Rust UI helper struct, not JSON-backed.
// ADR-0013/ADR-0039 Default ban applies only to JSON-deserialized types.
impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            hint: String::new(),
            size: UiTheme::DEFAULT_WINDOW_SIZE,
            header_height: UiTheme::HEADER_HEIGHT,
            animation: None,
            scrollable: true,
        }
    }
}

/// Spawns a scan line dot entity.
fn spawn_scan_line_dot(
    commands: &mut ChildSpawnerCommands<'_>,
    start_corner: u8,
    is_main: bool,
    trail_index: usize,
    parent_entity: Entity,
) {
    let dot_size = if is_main {
        UiTheme::SCAN_DOT_RADIUS * 2.0
    } else {
        UiTheme::SCAN_DOT_RADIUS * 1.5
    };

    commands.spawn((
        Name::new(format!("ScanDot_{start_corner}_{is_main}_{trail_index}")),
        Node {
            width: Val::Px(dot_size),
            height: Val::Px(dot_size),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(UiTheme::SCAN_LINE_COLOR),
        ZIndex(200),
        ScanLineDot {
            start_corner,
            is_main,
            trail_index,
            parent_entity,
        },
    ));
}

/// Spawns corner bracket entities for a specific corner.
fn spawn_corner_bracket(commands: &mut ChildSpawnerCommands<'_>, corner: u8, size: Vec2) {
    let bracket_len = UiTheme::BRACKET_LENGTH;
    let bracket_width = UiTheme::BRACKET_WIDTH;

    // Calculate positions based on corner
    let (h_left, h_top) = match corner {
        0 => (0.0, 0.0),                                     // top-left
        1 => (size.x - bracket_len, 0.0),                    // top-right
        2 => (0.0, size.y - bracket_width),                  // bottom-left
        3 => (size.x - bracket_len, size.y - bracket_width), // bottom-right
        _ => unreachable!(),
    };

    let (v_left, v_top) = match corner {
        0 => (0.0, 0.0),                                     // top-left
        1 => (size.x - bracket_width, 0.0),                  // top-right
        2 => (0.0, size.y - bracket_len),                    // bottom-left
        3 => (size.x - bracket_width, size.y - bracket_len), // bottom-right
        _ => unreachable!(),
    };

    // Horizontal line of bracket
    commands.spawn((
        Name::new(format!("BracketH_{corner}")),
        Node {
            width: Val::Px(bracket_len),
            height: Val::Px(bracket_width),
            position_type: PositionType::Absolute,
            left: Val::Px(h_left),
            top: Val::Px(h_top),
            ..default()
        },
        BackgroundColor(UiTheme::BRACKET_COLOR),
        ZIndex(200),
    ));

    // Vertical line of bracket
    commands.spawn((
        Name::new(format!("BracketV_{corner}")),
        Node {
            width: Val::Px(bracket_width),
            height: Val::Px(bracket_len),
            position_type: PositionType::Absolute,
            left: Val::Px(v_left),
            top: Val::Px(v_top),
            ..default()
        },
        BackgroundColor(UiTheme::BRACKET_COLOR),
        ZIndex(200),
    ));
}

/// Spawns a generic window with a title bar, hint, and content area.
///
/// Uses pure Bevy UI (`Node` + `Text` + `ScrollPosition`) for layout and clipping.
///
/// The window consists of:
/// - A semi-transparent black panel centered on screen.
/// - A header row with the title on the left and hint on the right.
/// - A content area that fills the remaining space below the header.
/// - Optional fade-out zones at the top and bottom of the content area (if scrollable).
/// - Optional mouse wheel scrolling for overflow content (if scrollable).
/// - Animated corner brackets and scan lines on the border.
///
/// The `content_fn` callback is called with the content area's child spawner,
/// allowing the caller to spawn arbitrary content inside the window.
///
/// Returns the spawned root entity.
// Window layout has many required fields (header, content, fade zones, brackets, scan lines).
#[allow(clippy::too_many_lines)]
pub fn spawn_window(
    commands: &mut Commands<'_, '_>,
    config: &WindowConfig,
    asset_server: &Res<'_, AssetServer>,
    theme: &Res<'_, UiTheme>,
    content_fn: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) -> Entity {
    // Load fade gradient images into the asset server
    let fade_top_image = asset_server.add(create_fade_top_image());
    let fade_bottom_image = asset_server.add(create_fade_bottom_image());

    let size = config.size;
    let header_height = config.header_height;
    // Bounds relative to panel origin (top-left of panel)
    let panel_bounds = Vec4::new(0.0, 0.0, size.x, size.y);

    // Spawn panel first so we can get its entity
    let panel_entity = commands
        .spawn((
            Name::new("Panel"),
            Node {
                width: Val::Px(size.x),
                height: Val::Px(size.y),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(UiTheme::BACKGROUND_COLOR),
            WindowBorder {
                anim_time: 0.0,
                bounds: panel_bounds,
            },
        ))
        .id();

    // Spawn window root and add panel as child
    let window_root = commands
        .spawn((
            Name::new("WindowRoot"),
            WindowRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            RenderLayer::Menu.render_layers(),
        ))
        .add_child(panel_entity)
        .id();

    // Spawn corner brackets and scan line dots as children of the panel
    commands.entity(panel_entity).with_children(|ui| {
        // Spawn corner brackets
        for corner in 0..4u8 {
            spawn_corner_bracket(ui, corner, size);
        }

        // Spawn scan line dots (4 corners x (1 main + trail dots))
        for corner in 0..4u8 {
            // Main dot
            spawn_scan_line_dot(ui, corner, true, 0, panel_entity);

            // Trail dots
            for trail_idx in 0..UiTheme::SCAN_TRAIL_LENGTH {
                spawn_scan_line_dot(ui, corner, false, trail_idx, panel_entity);
            }
        }

        // Header row — only when header_height > 0.0
        if header_height > 0.0 {
            ui.spawn((
                Name::new("HeaderRow"),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(header_height),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(UiTheme::HEADER_PADDING)),
                    flex_shrink: 0.0,
                    ..default()
                },
            ))
            .with_children(|ui| {
                // Title text
                ui.spawn((
                    Name::new("Title"),
                    Text::new(&config.title),
                    TextFont {
                        font: theme.font.clone(),
                        font_size: UiTheme::TITLE_FONT_SIZE,
                        ..default()
                    },
                    TextColor(UiTheme::TITLE_COLOR),
                ));

                // Hint text
                ui.spawn((
                    Name::new("Hint"),
                    Text::new(&config.hint),
                    TextFont {
                        font: theme.font.clone(),
                        font_size: UiTheme::HINT_FONT_SIZE,
                        ..default()
                    },
                    TextColor(UiTheme::HINT_COLOR),
                ));
            });
        }

        // Content area — fills remaining space, clips overflow on X.
        // Y scrolling is handled by the inner ScrollContainer (if scrollable).
        ui.spawn((
            Name::new("ContentContainer"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(size.y - header_height),
                overflow: Overflow {
                    x: OverflowAxis::Clip,
                    y: OverflowAxis::Visible,
                },
                position_type: PositionType::Relative,
                ..default()
            },
        ))
        .with_children(|ui| {
            if config.scrollable {
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
                ))
                .with_children(|ui| {
                    // Top padding line (fade zone)
                    ui.spawn((
                        Name::new("PaddingTop"),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(UiTheme::FADE_ZONE_HEIGHT),
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));

                    // Actual content from the caller
                    content_fn(ui);

                    // Bottom padding line (fade zone)
                    ui.spawn((
                        Name::new("PaddingBottom"),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(UiTheme::FADE_ZONE_HEIGHT),
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));
                });

                // Fade-out zone: top — absolutely positioned, NOT affected by scroll
                ui.spawn((
                    Name::new("FadeTop"),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(UiTheme::FADE_ZONE_HEIGHT),
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
                ));

                // Fade-out zone: bottom — absolutely positioned, NOT affected by scroll
                ui.spawn((
                    Name::new("FadeBottom"),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(UiTheme::FADE_ZONE_HEIGHT),
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
                ));
            } else {
                // Non-scrollable content — no fade zones, no scroll container
                content_fn(ui);
            }
        });
    });

    // Insert animation component if configured
    if let Some(anim) = &config.animation {
        commands.entity(window_root).insert(anim.clone());
    }

    window_root
}
