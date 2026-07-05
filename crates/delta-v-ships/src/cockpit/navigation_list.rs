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
use tracing::info_span;

use super::components::SelectedNavObject;
use super::components::SelectedTarget;
use super::components::Targetable;
use delta_v_core::events::{NavigationListChanged, TargetSelected};
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
    #[allow(clippy::expect_used)]
    let player_pos = targetable_query
        .iter()
        .find_map(|(entity, transform, _, _, _)| {
            if entity == player_ship.0 {
                Some(transform.translation)
            } else {
                None
            }
        })
        .expect("PlayerShipEntity resource references an entity not found in targetable_query. This indicates the player ship is missing the Targetable component or the entity was despawned.");
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
    let _span = info_span!("delta_v_ships::update_selection_system").entered();
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

/// Updates navigation list distances at fixed timestep while the menu is open.
///
/// Runs in `FixedUpdate` during `AppState::InGame` (60 Hz).
/// Recalculates distances from player to all tracked entities and re-sorts
/// the list if any distance changed significantly (>1m per tick).
/// Emits `NavigationListChanged` when the list is re-sorted.
///
/// The 1.0m threshold is evaluated at fixed 60 Hz timestep, making the
/// re-sort behavior deterministic regardless of render frame rate.
///
/// # Panics
/// Panics if `PlayerShipEntity` resource references an entity not found in
/// the `targetable_query` (i.e., player ship missing `Targetable` component
/// or entity was despawned). This indicates a world definition or spawn bug.
#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::missing_panics_doc
)]
pub fn update_navigation_list_distances_system(
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
    mut nav_list_events: MessageWriter<'_, NavigationListChanged>,
) {
    let _span = info_span!("delta_v_ships::update_navigation_list_distances_system").entered();
    // Get player position
    #[allow(clippy::expect_used)]
    let player_pos = targetable_query
        .iter()
        .find_map(|(entity, transform, _, _, _)| {
            if entity == player_ship.0 {
                Some(transform.translation)
            } else {
                None
            }
        })
        .expect("PlayerShipEntity resource references an entity not found in targetable_query. This indicates the player ship is missing the Targetable component or the entity was despawned.");

    let mut any_distance_changed = false;

    // Update distances for existing entries
    for entry in &mut list_data.entries {
        // Find the entity in the appropriate query
        let new_distance = match targeting_mode.mode {
            TargetingModeType::Combat => {
                targetable_query
                    .iter()
                    .find_map(|(entity, transform, _, _, _)| {
                        if entity == entry.entity {
                            Some((transform.translation - player_pos).length())
                        } else {
                            None
                        }
                    })
            }
            TargetingModeType::Nav => {
                navigable_query
                    .iter()
                    .find_map(|(entity, transform, _, _, _)| {
                        if entity == entry.entity {
                            Some((transform.translation - player_pos).length())
                        } else {
                            None
                        }
                    })
            }
        };

        if let Some(new_distance) = new_distance {
            // Check if distance changed significantly (more than 1 meter)
            if (entry.distance - new_distance).abs() > 1.0 {
                entry.distance = new_distance;
                any_distance_changed = true;
            }
        }
    }

    // Re-sort by distance if any changed
    if any_distance_changed {
        list_data.entries.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        nav_list_events.write(NavigationListChanged);
        tracing::debug!(
            "[nav_list] distances updated and re-sorted ({} entries)",
            list_data.entries.len()
        );
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

/// Handles `TargetSelected` events from the navigation menu click system.
///
/// Runs in `Update` during `AppState::InGame`.
/// When a `TargetSelected` event is received, updates the appropriate
/// `SelectedTarget` or `SelectedNavObject` resource based on the targeting mode.
/// This ensures that clicking a row in the navigation menu actually selects the target.
#[allow(clippy::needless_pass_by_value)]
pub fn handle_target_selected_system(
    mut events: MessageReader<'_, '_, TargetSelected>,
    mut selected_target: ResMut<'_, SelectedTarget>,
    mut selected_nav_object: ResMut<'_, SelectedNavObject>,
) {
    let _span = info_span!("delta_v_ships::handle_target_selected_system").entered();
    for event in events.read() {
        tracing::debug!(
            "[nav_list] received TargetSelected event: target={:?} mode={:?}",
            event.target,
            event.mode
        );

        match event.mode {
            TargetingModeType::Combat => {
                selected_target.0 = Some(event.target);
            }
            TargetingModeType::Nav => {
                selected_nav_object.0 = Some(event.target);
            }
        }
    }
}
