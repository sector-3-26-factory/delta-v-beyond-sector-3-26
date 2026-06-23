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

//! Window scrolling systems.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::ScrollPosition;

use super::components::WindowScrollContainer;

/// Reads mouse wheel input and updates `ScrollPosition` on window scroll containers.
///
/// Runs in `Update`. Applies scroll delta with proper bounds checking using
/// `ComputedNode` measurements. This prevents accumulating offset past content boundaries.
#[allow(clippy::needless_pass_by_value)]
// MessageReader<MouseWheel> is a Bevy message reader that must be passed by value
// per Bevy's API design; it cannot be borrowed.
pub fn window_scroll_system(
    mut mouse_wheel_reader: MessageReader<'_, '_, MouseWheel>,
    mut query: Query<
        '_,
        '_,
        (&mut ScrollPosition, &Node, &ComputedNode),
        With<WindowScrollContainer>,
    >,
) {
    let _span = tracing::info_span!("delta_v_ui::window_scroll_system").entered();

    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta_y = -mouse_wheel.y;
        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta_y *= 20.0;
        }

        if delta_y == 0.0 {
            continue;
        }

        for (mut scroll_position, node, computed) in &mut query {
            if node.overflow.y != OverflowAxis::Scroll {
                continue;
            }

            let max_offset =
                (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

            // Only apply delta if not at boundary
            let at_boundary = if delta_y > 0.0 {
                scroll_position.y >= max_offset.y
            } else {
                scroll_position.y <= 0.0
            };

            if !at_boundary {
                scroll_position.y += delta_y;
                // Clamp to valid range
                scroll_position.y = scroll_position.y.clamp(0.0, max_offset.y);
            }
        }
    }
}
