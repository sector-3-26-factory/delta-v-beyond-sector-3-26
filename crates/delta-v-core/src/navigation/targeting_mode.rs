// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Targeting mode resource for combat vs navigation mode.

use bevy::prelude::*;

/// Resource tracking the current targeting mode.
///
/// Determines which list (combat targets or nav objects) is displayed.
/// This resource is used by both `delta-v-ships` (to update the navigation list)
/// and `delta-v-ui` (to refresh the navigation menu when mode changes).
// allow-default: Bevy requires Default on resources for init_resource. Defaults to Combat mode.
#[derive(Resource, Default)]
pub struct TargetingMode {
    /// Current mode: Combat (targeting) or Nav (navigation).
    pub mode: TargetingModeType,
}

/// The targeting mode type.
// allow-default: Bevy requires Default on resources for init_resource. Combat is the default mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetingModeType {
    /// Combat mode: targeting enemies.
    #[default]
    Combat,
    /// Navigation mode: selecting objects to navigate to.
    Nav,
}
