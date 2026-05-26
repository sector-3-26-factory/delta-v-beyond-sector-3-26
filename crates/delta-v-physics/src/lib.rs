// See AGENTS.md
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

//! Newtonian physics with gravity, floating origin, and avian3d integration.
//!
//! See ADR-0007 (Floating origin), ADR-0009 (Newtonian physics with gravity),
//! and ADR-0017 (Fixed timestep and determinism).

use bevy::prelude::*;

/// Physics plugin providing Newtonian dynamics and collision detection.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, _app: &mut App) {
        log::info!("PhysicsPlugin initialized");
    }
}
