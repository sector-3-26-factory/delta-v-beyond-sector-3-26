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

//! Navigation menu module.
//!
//! This module handles the player-controlled navigation list UI.
//! It is triggered by user interaction (N key press).

use bevy::prelude::*;
use delta_v_core::AppState;

pub mod components;
pub mod resources;
pub mod spawn;
pub mod systems;

pub use components::*;
pub use resources::NavigationMenuOpen;

/// Plugin for navigation menu systems.
pub struct NavigationMenuPlugin;

impl Plugin for NavigationMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NavigationMenuOpen>().add_systems(
            Update,
            systems::navigation_menu_toggle_system.run_if(in_state(AppState::InGame)),
        );
        app.add_systems(
            Update,
            systems::navigation_menu_refresh_system.run_if(in_state(AppState::InGame)),
        );
    }
}
