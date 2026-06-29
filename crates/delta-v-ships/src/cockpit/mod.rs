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

//! Cockpit overlay system for ship-specific HUD views.
//!
//! This module handles:
//! - Cockpit overlay PNG rendering with alpha transparency
//! - Station switching (F2 / Shift+F2)
//! - Velocity vector indicator (direction of ship movement)
//!
//! See M6 -- HUD and Feel plan.

use bevy::prelude::*;
use delta_v_core::AppState;

pub mod components;
pub mod spawn;
pub mod systems;
pub mod velocity_indicator;

pub use components::*;
pub use spawn::CockpitOverlayResource;

/// Plugin for cockpit overlay systems.
pub struct CockpitPlugin;

impl Plugin for CockpitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<systems::ArrowTextureCache>()
            .init_resource::<systems::CockpitCycleState>()
            .add_systems(OnEnter(AppState::InGame), spawn::spawn_cockpit_overlay)
            .add_systems(
                OnEnter(AppState::InGame),
                spawn::spawn_velocity_vector_indicator,
            )
            .add_systems(
                Update,
                (
                    systems::cockpit_station_cycle_system,
                    systems::cockpit_visibility_system,
                    systems::velocity_vector_system,
                )
                    .run_if(in_state(AppState::InGame))
                    .chain(),
            );
    }
}
