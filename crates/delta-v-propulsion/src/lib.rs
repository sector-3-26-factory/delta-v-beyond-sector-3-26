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

//! Thrusters, hyperdrive, and propulsion systems.

use bevy::prelude::*;

/// Propulsion plugin for managing thrust and hyperdrive.
///
/// Note: Propulsion selection is handled by `delta_v_ships::propulsion_selection_system`
/// to avoid domain-to-domain crate dependencies (ADR-0051).
pub struct PropulsionPlugin;

impl Plugin for PropulsionPlugin {
    fn build(&self, _app: &mut App) {
        tracing::info!("PropulsionPlugin initialized");
    }
}
