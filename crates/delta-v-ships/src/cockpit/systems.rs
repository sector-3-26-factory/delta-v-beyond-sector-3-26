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

//! Cockpit-related systems.

use bevy::prelude::*;

/// Cycles to the next cockpit station when the player presses the key.
///
/// Runs in `Update` during `AppState::InGame`.
#[allow(clippy::needless_pass_by_value)]
pub fn cockpit_station_cycle_next_system(keyboard: Res<'_, ButtonInput<KeyCode>>) {
    if !keyboard.pressed(KeyCode::F2) {
        return;
    }

    // TODO: Implement station switching logic
    // This will cycle through stations in JSON order
    log::debug!("cockpit: cycle next station");
}

/// Cycles to the previous cockpit station when the player presses the key.
///
/// Runs in `Update` during `AppState::InGame`.
#[allow(clippy::needless_pass_by_value)]
pub fn cockpit_station_cycle_prev_system(keyboard: Res<'_, ButtonInput<KeyCode>>) {
    if !keyboard.pressed(KeyCode::AltLeft) || !keyboard.pressed(KeyCode::F2) {
        return;
    }

    // TODO: Implement station switching logic
    // This will cycle backward through stations in JSON order
    log::debug!("cockpit: cycle prev station");
}
