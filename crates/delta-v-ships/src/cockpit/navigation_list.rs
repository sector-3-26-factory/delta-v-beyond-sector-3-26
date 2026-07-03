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

//! Navigation list data and systems.
//!
//! Provides the player-controlled targeting & navigation list functionality.

use bevy::prelude::*;

use super::components::SelectedNavObject;
use super::components::SelectedTarget;
use super::components::Targetable;
use delta_v_core::events::NavigationListChanged;
use delta_v_core::navigation::{
    EntityType, NavigationListData, TargetingMode, TargetingModeType, WorldEntityId,
};

/// Internal function to rebuild the navigation list entries.
///
/// Shared by `init_navigation_list_system` and `update_navigation_list_system`.
#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::type_complexity
)]
fn rebuild_navigation_list(
    player_ship: Res<'_, delta_v_core::PlayerShipEntity>,
    targeting_mode: Res<'_, TargetingMode>,
    mut list_data: ResMut<'_, NavigationListData>,
    targetable_query: Query<
        '_,
        '_,
        (
            Entity,
            &Transform,
            Option<&Name>,
            &EntityType,
            &WorldEntityId,
        ),
        With<Targetable>,
    >,
    navigable_query: Query<
        '_,
        '_,
        (
            Entity,
            &Transform,
            Option<&Name>,
            &EntityType,
            &WorldEntityId,
        ),
    >,
) {
    // Get player position - find the player ship by checking all entities with Transform
    // We use the player_ship resource to identify which entity is the player
    let player_pos = targetable_query
        .iter()
        .find_map(|(entity, transform, _, _, _)| {
            if entity == player_ship.0 {
                Some(transform.translation)
            } else {
                None
            }
        })
        .unwrap_or(Vec3::ZERO);

    // Collect entries based on mode
    let mut entries: Vec<delta_v_core::NavEntry> = match targeting_mode.mode {
        TargetingModeType::Combat => targetable_query
            .iter()
            .filter_map(|(entity, transform, name, entity_type, entity_id)| {
                if entity == player_ship.0 {
                    return None;
                }
                let distance = (transform.translation - player_pos).length();
                let _name_str = name.map_or_else(|| entity_id.0.clone(), ToString::to_string);
                Some(delta_v_core::NavEntry {
                    entity,
                    entity_type: entity_type.0.clone(),
                    entity_id: entity_id.0.clone(),
                    distance,
                })
            })
            .collect(),
        TargetingModeType::Nav => navigable_query
            .iter()
            .filter_map(|(entity, transform, name, entity_type, entity_id)| {
                if entity == player_ship.0 {
                    return None;
                }
                let distance = (transform.translation - player_pos).length();
                let _name_str = name.map_or_else(|| entity_id.0.clone(), ToString::to_string);
                Some(delta_v_core::NavEntry {
                    entity,
                    entity_type: entity_type.0.clone(),
                    entity_id: entity_id.0.clone(),
                    distance,
                })
            })
            .collect(),
    };

    // Sort by distance
    entries.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    list_data.entries = entries;
    tracing::debug!(
        "[nav_list] updated with {} entries (mode: {:?})",
        list_data.entries.len(),
        targeting_mode.mode
    );
    for entry in &list_data.entries {
        tracing::debug!(
            "[nav_list]   entry: entity={:?} type={} id={} distance={:.1}",
            entry.entity,
            entry.entity_type,
            entry.entity_id,
            entry.distance
        );
    }
}

/// Updates the navigation list based on the current targeting mode.
///
/// Runs in `Update` during `AppState::InGame`.
/// - Combat mode: queries entities with `Targetable` + `Transform` (ships, stations)
/// - Nav mode: queries ALL entities with `EntityType` + `EntityId` + `Transform` (all entities are navigatable)
///
/// Triggered by `TargetingModeChanged` events. Emits `NavigationListChanged` after updating.
#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_lines
)]
pub fn update_navigation_list_system(
    player_ship: Res<'_, delta_v_core::PlayerShipEntity>,
    targeting_mode: Res<'_, TargetingMode>,
    list_data: ResMut<'_, NavigationListData>,
    targetable_query: Query<
        '_,
        '_,
        (
            Entity,
            &Transform,
            Option<&Name>,
            &EntityType,
            &WorldEntityId,
        ),
        With<Targetable>,
    >,
    // Nav mode: all entities with EntityType + WorldEntityId are navigatable
    navigable_query: Query<
        '_,
        '_,
        (
            Entity,
            &Transform,
            Option<&Name>,
            &EntityType,
            &WorldEntityId,
        ),
    >,
    mut events: MessageReader<'_, '_, delta_v_core::TargetingModeChanged>,
    mut nav_list_events: MessageWriter<'_, NavigationListChanged>,
) {
    // Only run when targeting mode changes
    if events.read().next().is_none() {
        return;
    }
    let _span = tracing::info_span!("delta_v_ships::update_navigation_list_system").entered();

    // Debug: log all entities found by each query
    tracing::debug!(
        "[nav_list] Combat query found {} entities",
        targetable_query.iter().count()
    );
    for (entity, _, name, entity_type, entity_id) in targetable_query.iter() {
        tracing::debug!(
            "[nav_list] Combat entity: {:?} name={:?} type={} id={}",
            entity,
            name,
            entity_type.0,
            entity_id.0
        );
    }

    tracing::debug!(
        "[nav_list] Nav query found {} entities",
        navigable_query.iter().count()
    );
    for (entity, _, name, entity_type, entity_id) in navigable_query.iter() {
        tracing::debug!(
            "[nav_list] Nav entity: {:?} name={:?} type={} id={}",
            entity,
            name,
            entity_type.0,
            entity_id.0
        );
    }

    // Store the mode before moving targeting_mode into rebuild_navigation_list
    let current_mode = targeting_mode.mode;

    // Rebuild the navigation list using shared function
    rebuild_navigation_list(
        player_ship,
        targeting_mode,
        list_data,
        targetable_query,
        navigable_query,
    );

    // Debug: log all entities found by the navigable query
    if current_mode == TargetingModeType::Nav {
        tracing::debug!("[nav_list] Nav mode - all entities with EntityType+EntityId:");
        for (entity, _, name, entity_type, entity_id) in navigable_query.iter() {
            tracing::debug!(
                "[nav_list]   entity={:?} name={:?} type={} id={}",
                entity,
                name,
                entity_type.0,
                entity_id.0
            );
        }
    }

    // Emit event to notify UI that the navigation list has changed
    nav_list_events.write(NavigationListChanged);
    tracing::debug!("[nav_list] emitted NavigationListChanged event");
}

/// Updates the selected target/nav object based on the current selection.
///
/// This system is called after the navigation menu is opened to ensure
/// the selection is valid.
#[allow(clippy::needless_pass_by_value)]
pub fn update_selection_system(
    mut selected_target: ResMut<'_, SelectedTarget>,
    mut selected_nav_object: ResMut<'_, SelectedNavObject>,
    targeting_mode: Res<'_, TargetingMode>,
    list_data: Res<'_, NavigationListData>,
) {
    // If no selection exists and list has entries, select the first one
    match targeting_mode.mode {
        TargetingModeType::Combat => {
            if selected_target.0.is_none()
                && let Some(first) = list_data.entries.first()
            {
                selected_target.0 = Some(first.entity);
            }
        }
        TargetingModeType::Nav => {
            if selected_nav_object.0.is_none()
                && let Some(first) = list_data.entries.first()
            {
                selected_nav_object.0 = Some(first.entity);
            }
        }
    }
}

/// Initializes the navigation list on game start.
///
/// Runs in `OnEnter(AppState::InGame)` to populate the initial navigation list
/// before any mode changes occur. Emits `NavigationListChanged` after updating.
#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_lines
)]
pub fn init_navigation_list_system(
    player_ship: Res<'_, delta_v_core::PlayerShipEntity>,
    targeting_mode: Res<'_, TargetingMode>,
    list_data: ResMut<'_, NavigationListData>,
    targetable_query: Query<
        '_,
        '_,
        (
            Entity,
            &Transform,
            Option<&Name>,
            &EntityType,
            &WorldEntityId,
        ),
        With<Targetable>,
    >,
    // Nav mode: all entities with EntityType + WorldEntityId are navigatable
    navigable_query: Query<
        '_,
        '_,
        (
            Entity,
            &Transform,
            Option<&Name>,
            &EntityType,
            &WorldEntityId,
        ),
    >,
    mut nav_list_events: MessageWriter<'_, NavigationListChanged>,
) {
    let _span = tracing::info_span!("delta_v_ships::init_navigation_list_system").entered();

    // Rebuild the navigation list using shared function
    rebuild_navigation_list(
        player_ship,
        targeting_mode,
        list_data,
        targetable_query,
        navigable_query,
    );

    // Emit event to notify UI that the navigation list has changed
    nav_list_events.write(NavigationListChanged);
    tracing::debug!("[nav_list] emitted NavigationListChanged event (init)");
}
