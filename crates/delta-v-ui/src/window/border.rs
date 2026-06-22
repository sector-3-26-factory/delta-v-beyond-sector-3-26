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

//! Window border animation system.
//!
//! Handles the animated scan lines that travel around the window border.
//! Four scan lines start from each corner and travel clockwise, taking
//! the same time to reach the next corner regardless of edge length.

use bevy::prelude::*;

use super::components::WindowBorder;
use super::theme::UiTheme;

/// System set for border animation systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BorderSystemSet;

/// Marker component for scan line dot entities.
#[derive(Component)]
pub struct ScanLineDot {
    /// Which corner this scan line starts from (0=top-left, 1=top-right, 2=bottom-right, 3=bottom-left).
    pub start_corner: u8,
    /// Whether this is the main dot (true) or a trail dot (false).
    pub is_main: bool,
    /// Trail index (0 = oldest/furthest back, higher = closer to main dot).
    pub trail_index: usize,
}

/// Updates the animation time for window border scan lines.
///
/// Runs in `Update`. Increments the `anim_time` field on all `WindowBorder`
/// components based on the frame delta time.
pub fn update_border_anim_time(time: Res<Time>, mut query: Query<&mut WindowBorder>) {
    let _span = tracing::info_span!("delta_v_ui::update_border_anim_time").entered();

    for mut border in &mut query {
        border.anim_time += time.delta_secs_f64();
    }
}

/// Calculates the position of a scan line on a rectangle perimeter.
///
/// The scan line travels clockwise around the rectangle, taking
/// `CORNER_TO_CORNER_TIME` seconds to traverse each edge.
///
/// Edge order: 0=top (left->right), 1=right (top->bottom), 2=bottom (right->left), 3=left (bottom->top)
fn get_scan_position(anim_time: f64, start_corner: u8, bounds: Vec4) -> Vec2 {
    let min_x = bounds.x;
    let min_y = bounds.y;
    let max_x = bounds.z;
    let max_y = bounds.w;

    let top_len = max_x - min_x;
    let right_len = max_y - min_y;

    // Time for one full loop (4 edges)
    let total_time = UiTheme::SCAN_CORNER_TO_CORNER_TIME * 4.0;
    let t = anim_time.rem_euclid(total_time);

    // Which edge are we on? (0=top, 1=right, 2=bottom, 3=left)
    let edge_idx = (t / UiTheme::SCAN_CORNER_TO_CORNER_TIME) as u8 % 4;
    let edge_time = t % UiTheme::SCAN_CORNER_TO_CORNER_TIME;
    let edge_progress = (edge_time / UiTheme::SCAN_CORNER_TO_CORNER_TIME) as f32;

    // Actual edge index based on starting corner
    // start_corner 0 = top-left, so edge 0 starts at top-left going right
    // start_corner 1 = top-right, so edge 1 starts at top-right going down
    // etc.
    let actual_edge = (start_corner + edge_idx) % 4;

    match actual_edge {
        0 => {
            // Top edge: left to right
            Vec2::new(min_x + edge_progress * top_len, min_y)
        }
        1 => {
            // Right edge: top to bottom
            Vec2::new(max_x, min_y + edge_progress * right_len)
        }
        2 => {
            // Bottom edge: right to left
            Vec2::new(max_x - edge_progress * top_len, max_y)
        }
        3 => {
            // Left edge: bottom to top
            Vec2::new(min_x, max_y - edge_progress * right_len)
        }
        _ => unreachable!(),
    }
}

/// Calculates trail positions for a scan line.
///
/// The trail extends behind the main dot (in the counter-clockwise direction).
/// For a scan line starting at corner 0 (top-left):
/// - Main dot starts at top-left, moves right along top edge
/// - Trail starts on left edge (bottom to top), reaches corner, then turns right
///
/// The trail spans `trail_duration` seconds of movement time.
fn get_trail_positions(anim_time: f64, start_corner: u8, bounds: Vec4) -> Vec<Vec2> {
    let num_trail_points = UiTheme::SCAN_TRAIL_LENGTH;
    let trail_duration = UiTheme::SCAN_TRAIL_DURATION;

    // The trail needs to start far enough back that when the main dot is at the corner,
    // the trail head is on the previous edge (coming into the corner).
    //
    // For corner 0 (top-left): main dot moves right on top edge
    // Trail should start on left edge (moving up), reach corner, then turn right
    //
    // We need the trail to span from (corner - trail_duration) to corner
    // This means the oldest trail point should be trail_duration seconds behind

    let mut positions = Vec::with_capacity(num_trail_points);

    for i in 0..num_trail_points {
        // Time offset: trail starts trail_duration behind main dot
        // and extends to the main dot position
        let fraction = i as f64 / num_trail_points as f64;
        let time_offset = trail_duration * (1.0 - fraction);
        let trail_time = anim_time - time_offset;
        let pos = get_scan_position(trail_time, start_corner, bounds);
        positions.push(pos);
    }

    positions
}

/// Updates the positions of scan line dots based on the current animation time.
///
/// Runs in `Update`. Updates the position and color of all scan line dot
/// entities to create the animated border effect.
pub fn update_scan_line_dots(
    mut query: Query<(&mut Node, &mut BackgroundColor, &ScanLineDot)>,
    border_query: Query<&WindowBorder>,
) {
    let _span = tracing::info_span!("delta_v_ui::update_scan_line_dots").entered();

    let Some(border) = border_query.iter().next() else {
        return;
    };

    let bounds = border.bounds;
    let scan_color = UiTheme::SCAN_LINE_COLOR;

    for (mut node, mut bg_color, dot) in &mut query {
        let anim_time = border.anim_time;

        if dot.is_main {
            // Main dot
            let pos = get_scan_position(anim_time, dot.start_corner, bounds);
            node.left = Val::Px(pos.x - UiTheme::SCAN_DOT_RADIUS);
            node.top = Val::Px(pos.y - UiTheme::SCAN_DOT_RADIUS);
            bg_color.0 = scan_color;
        } else {
            // Trail dot
            let trail = get_trail_positions(anim_time, dot.start_corner, bounds);
            if dot.trail_index < trail.len() {
                let pos = trail[dot.trail_index];
                node.left = Val::Px(pos.x - UiTheme::SCAN_DOT_RADIUS);
                node.top = Val::Px(pos.y - UiTheme::SCAN_DOT_RADIUS);

                // Alpha: oldest (index 0) is most transparent, newest is most opaque
                let alpha = (dot.trail_index as f32 + 1.0) / trail.len() as f32;
                bg_color.0 = Color::srgba(
                    scan_color.to_linear().red,
                    scan_color.to_linear().green,
                    scan_color.to_linear().blue,
                    alpha,
                );
            }
        }
    }
}

/// Plugin for window border effects.
pub struct WindowBorderPlugin;

impl Plugin for WindowBorderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_border_anim_time.in_set(BorderSystemSet),
                update_scan_line_dots.in_set(BorderSystemSet),
            ),
        );
    }
}
