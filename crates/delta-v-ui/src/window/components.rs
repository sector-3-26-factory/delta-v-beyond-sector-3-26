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

//! Window components.

use bevy::prelude::*;

/// Height of the fade-out zones in pixels.
///
/// This constant controls both the height of the gradient overlay sprites
/// and the padding lines added at the top and bottom of the scroll content.
pub const FADE_ZONE_HEIGHT: f32 = 30.0;

/// Window background color (semi-transparent black).
///
/// Used for the panel background and as the fade target color for scroll
/// fade zones. Content fades to this color at the top and bottom edges.
pub const BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);

/// Marker component for a window root entity.
///
/// This component is added to the root node of a window UI tree.
/// It is used to find and despawn the window when closing.
#[derive(Component)]
pub struct WindowRoot;

/// Marker component for window scroll containers.
///
/// Added to the inner scroll container entity to identify it for the scroll system.
#[derive(Component)]
pub struct WindowScrollContainer;
