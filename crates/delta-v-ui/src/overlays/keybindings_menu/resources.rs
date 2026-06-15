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

//! Keybindings menu resources.

use bevy::prelude::*;

/// Resource tracking whether the keybindings menu is currently open.
///
/// This resource is toggled by the `keybindings_menu_toggle_system`.
/// When `true`, the menu UI is visible; when `false`, it is hidden.
// INVARIANT: Bevy ECS resource requires Default trait for resource initialization.
// The default value is `false` (menu closed).
#[derive(Resource, Default)]
pub struct KeybindingsMenuOpen(pub bool);
