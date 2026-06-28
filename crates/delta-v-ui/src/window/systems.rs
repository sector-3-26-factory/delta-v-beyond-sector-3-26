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

//! Window scrolling and animation systems.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::ScrollPosition;

use super::animations::{WindowAnimation, WindowAnimationFlickerPhase};
use super::components::WindowRoot;
use super::components::WindowScrollContainer;

/// Reads mouse wheel input and updates `ScrollPosition` on window scroll containers.
///
/// Runs in `Update`. Applies scroll delta with proper bounds checking using
/// `ComputedNode` measurements. This prevents accumulating offset past content boundaries.
#[allow(clippy::needless_pass_by_value)]
// `MessageReader<MouseWheel>` must be passed by value per Bevy's API design.
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

            let at_boundary = if delta_y > 0.0 {
                scroll_position.y >= max_offset.y
            } else {
                scroll_position.y <= 0.0
            };

            if !at_boundary {
                scroll_position.y += delta_y;
                scroll_position.y = scroll_position.y.clamp(0.0, max_offset.y);
            }
        }
    }
}

/// Drives the flicker animation on window entities with `WindowAnimation`.
///
/// Runs in `Update`. Advances the animation phase and modulates the opacity
/// of ALL visual components (`BackgroundColor`, `TextColor`) in the entire
/// window subtree to create a flicker-in -> hold -> flicker-out effect.
#[allow(
    clippy::needless_pass_by_value,
    irrefutable_let_patterns,
    clippy::cast_precision_loss
)]
pub fn window_animation_system(
    time: Res<'_, Time>,
    mut anim_query: Query<'_, '_, (&mut WindowAnimation, &Children), With<WindowRoot>>,
    mut bg_query: Query<'_, '_, &mut BackgroundColor>,
    mut text_query: Query<'_, '_, &mut TextColor>,
    children_query: Query<'_, '_, &Children>,
) {
    let _span = tracing::info_span!("delta_v_ui::window_animation_system").entered();

    let delta = time.delta_secs();

    for (mut animation, children) in &mut anim_query {
        let WindowAnimation::Flicker(flicker) = animation.as_mut() else {
            continue;
        };

        flicker.phase_elapsed += delta;

        let count = flicker.flicker_count as f32;

        let opacity = match flicker.phase {
            WindowAnimationFlickerPhase::FlickerIn => {
                if flicker.flicker_in_duration <= 0.0 {
                    1.0
                } else {
                    let t = (flicker.phase_elapsed / flicker.flicker_in_duration).min(1.0);
                    // Cubic ease-out so the window stays invisible longer
                    t * t * t
                }
            }
            WindowAnimationFlickerPhase::Hold => 1.0,
            WindowAnimationFlickerPhase::FlickerOut => {
                if flicker.flicker_out_duration <= 0.0 {
                    0.0
                } else {
                    let t = (flicker.phase_elapsed / flicker.flicker_out_duration).min(1.0);
                    (1.0 - t) * 0.5f32.mul_add((std::f32::consts::TAU * count * t).sin(), 0.5)
                }
            }
            WindowAnimationFlickerPhase::Done => continue,
        };

        // Collect all descendant entities (breadth-first)
        let mut descendants = Vec::new();
        let mut queue: Vec<Entity> = children.iter().collect();
        while let Some(entity) = queue.pop() {
            descendants.push(entity);
            if let Ok(child_children) = children_query.get(entity) {
                queue.extend(child_children.iter());
            }
        }

        // Modulate every BackgroundColor and TextColor in the subtree
        for entity in &descendants {
            if let Ok(mut bg) = bg_query.get_mut(*entity) {
                bg.0.set_alpha(opacity);
            }
            if let Ok(mut tc) = text_query.get_mut(*entity) {
                tc.0.set_alpha(opacity);
            }
        }

        // Phase transitions
        let phase_duration = match flicker.phase {
            WindowAnimationFlickerPhase::FlickerIn => flicker.flicker_in_duration,
            WindowAnimationFlickerPhase::Hold => flicker.hold_duration,
            WindowAnimationFlickerPhase::FlickerOut => flicker.flicker_out_duration,
            WindowAnimationFlickerPhase::Done => continue,
        };

        if flicker.phase_elapsed >= phase_duration && phase_duration > 0.0 {
            flicker.phase = match flicker.phase {
                WindowAnimationFlickerPhase::FlickerIn => WindowAnimationFlickerPhase::Hold,
                WindowAnimationFlickerPhase::Hold => WindowAnimationFlickerPhase::FlickerOut,
                WindowAnimationFlickerPhase::FlickerOut | WindowAnimationFlickerPhase::Done => {
                    WindowAnimationFlickerPhase::Done
                }
            };
            flicker.phase_elapsed = 0.0;
        }
    }
}
