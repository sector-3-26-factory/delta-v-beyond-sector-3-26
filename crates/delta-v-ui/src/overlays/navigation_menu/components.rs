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

//! Navigation menu components.

use bevy::prelude::*;

/// Marker component for the navigation menu root entity.
#[derive(Component)]
pub struct NavigationMenuRoot;

/// Marker component for distance text entities in the navigation menu.
///
/// Each distance text entity corresponds to a navigation list entry by index.
/// The `update_navigation_menu_distances_system` uses this to find and update
/// the distance text for each entry without despawning the entire menu.
#[derive(Component)]
pub struct NavMenuDistanceText {
    /// Index of the entry in the navigation list this text corresponds to.
    pub index: usize,
}

/// Marker component for row background entities in the navigation menu.
///
/// Each row (type, id, distance columns) has this component with the same index.
/// The `update_navigation_menu_selection_system` uses this to update the
/// background color for the selected row without despawning the entire menu.
#[derive(Component)]
pub struct NavMenuRowBackground {
    /// Index of the entry in the navigation list this row corresponds to.
    pub index: usize,
}

/// Component linking a navigation menu row to its target entity.
///
/// This is used for click handling - when a row is clicked, we need to know
/// which entity it represents to emit a `TargetSelected` event.
#[derive(Component)]
pub struct NavMenuRowEntity {
    /// The Bevy Entity this row represents.
    pub entity: Entity,
}
