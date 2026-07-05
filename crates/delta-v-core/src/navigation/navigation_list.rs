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

//! Navigation list data for player-controlled targeting & navigation.
//!
//! This module provides the shared resource for the navigation/targeting list
//! that is updated by `delta-v-ships` and displayed by `delta-v-ui`.
//!
//! `EntityType` and `WorldEntityId` components are defined here per ADR-0046,
//! which forbids Component derives in delta-v-types.

use bevy::prelude::*;

/// Component storing the entity type for navigation list display.
///
/// The type is used for i18n display in the navigation menu.
/// All entities that should appear in the navigation list must have this component.
#[derive(Component, Debug, Clone)]
pub struct EntityType(pub String);

/// Component storing the entity ID from the world definition.
///
/// The ID is displayed in the navigation menu for entity identification.
/// All entities that should appear in the navigation list must have this component.
/// This is the string ID from the world definition (e.g., "ship-1", "asteroid-5"),
/// not to be confused with `delta_v_types::ids::EntityId` which wraps Bevy's Entity.
#[derive(Component, Debug, Clone)]
pub struct WorldEntityId(pub String);

/// A single entry in the navigation list.
#[derive(Debug, Clone)]
pub struct NavEntry {
    /// Entity ID of the navigable/targetable entity.
    pub entity: Entity,
    /// Entity type (e.g., "ship", "asteroid") for i18n display.
    pub entity_type: String,
    /// Entity ID from the world definition.
    pub entity_id: String,
    /// Distance from the player ship in meters.
    pub distance: f32,
}

/// Resource holding the current navigation list data.
///
/// Populated by systems in `delta-v-ships` based on the current targeting mode
/// (Combat vs Nav). Read by `delta-v-ui` to display the navigation menu.
// allow-default: Bevy requires Default on resources for init_resource
#[derive(Resource, Default)]
pub struct NavigationListData {
    /// List of entries, sorted by distance.
    pub entries: Vec<NavEntry>,
}

/// Resource holding the currently selected target entity (Combat mode).
///
/// The target reticle will appear around this entity when it's on-screen.
/// The navigation menu highlights this entity's row.
/// Updated by `delta-v-ships` when the player cycles targets.
/// Read by `delta-v-ui` to highlight the selected row.
#[derive(Resource, Default)]
pub struct SelectedTarget(pub Option<Entity>);

/// Resource holding the currently selected navigation object entity (Nav mode).
///
/// Used in Nav mode for navigation purposes.
/// The navigation menu highlights this entity's row.
/// Updated by `delta-v-ships` when the player cycles nav objects.
/// Read by `delta-v-ui` to highlight the selected row.
#[derive(Resource, Default)]
pub struct SelectedNavObject(pub Option<Entity>);
