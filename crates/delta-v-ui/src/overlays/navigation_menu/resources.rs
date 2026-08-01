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
// MERCHANTABILITY OR FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Navigation menu resources.

use bevy::prelude::*;

/// Resource tracking whether the navigation menu is open.
///
/// This resource is toggled by the `ToggleNavigationMenu` action.
/// This starts as false (menu closed).
// allow-default: Bevy ECS Resource trait requires Default for resource initialization.
#[derive(Resource, Default)]
pub struct NavigationMenuOpen(pub bool);

/// Resource tracking the previous state of the `ToggleNavigationMenu` action for edge detection.
///
/// Used by `navigation_menu_toggle_system` to detect key press transitions
/// (on press, not on hold). This provides more reliable edge detection than
/// relying solely on `ActionState::just_pressed`, which can be unreliable
/// when many keys are held simultaneously.
// allow-default: Bevy ECS Resource trait requires Default for resource initialization.
#[derive(Resource, Default)]
pub struct NavigationMenuToggleState {
    /// Whether `ToggleNavigationMenu` was pressed last frame.
    pub prev_pressed: bool,
}
