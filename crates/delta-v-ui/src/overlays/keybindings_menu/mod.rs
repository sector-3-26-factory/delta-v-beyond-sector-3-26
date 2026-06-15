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

//! Keybindings menu module.
//!
//! This module handles the in-game keybindings reference menu.
//! It is triggered by user interaction (F1 key press).

use bevy::prelude::*;
use delta_v_core::AppState;

pub mod components;
pub mod resources;
pub mod spawn;
pub mod systems;

pub use components::*;
pub use resources::KeybindingsMenuOpen;

/// Plugin for keybindings menu systems.
pub struct KeybindingsMenuPlugin;

impl Plugin for KeybindingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeybindingsMenuOpen>().add_systems(
            Update,
            systems::keybindings_menu_toggle_system.run_if(in_state(AppState::InGame)),
        );
    }
}
