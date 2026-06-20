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

use delta_v_core::input::ActiveActions;
use delta_v_core::input::LogicalAction;

use super::ActiveCockpitStation;
use super::components::CockpitOverlay;
use super::spawn::CockpitOverlayResource;

/// Cycles to the next cockpit station when the player presses the key.
///
/// Runs in `Update` during `AppState::InGame`.
/// Reads `ActiveActions` for `CockpitCycleNext`.
/// Cycles through stations in JSON order, loading the new station's PNG texture.
#[allow(clippy::needless_pass_by_value)]
pub fn cockpit_station_cycle_next_system(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    cockpit: Res<'_, CockpitOverlayResource>,
    mut active_station: ResMut<'_, ActiveCockpitStation>,
    query: Query<'_, '_, Entity, With<CockpitOverlay>>,
    active_actions: Res<'_, ActiveActions>,
) {
    // Check if the cycle next action is pressed
    if !active_actions.0.contains(&LogicalAction::CockpitCycleNext) {
        return;
    }

    let Ok(entity) = query.single() else {
        return;
    };

    // Find current station index
    let current_idx = cockpit
        .stations
        .iter()
        .position(|s| s.id == active_station.station_id)
        .unwrap_or(0);

    // Calculate next index (wrap around)
    let next_idx = (current_idx + 1) % cockpit.stations.len();
    let Some(next_station) = cockpit.stations.get(next_idx) else {
        return;
    };

    // Load the new texture
    let texture_handle = asset_server.load(&next_station.texture);

    // Insert a new overlay component with the new texture
    commands.entity(entity).insert(CockpitOverlay {
        texture: texture_handle,
    });

    // Update the active station resource
    active_station.station_id.clone_from(&next_station.id);

    tracing::debug!(
        "cockpit: switched to station '{}' ({})",
        next_station.id,
        next_station.texture
    );
}

/// Cycles to the previous cockpit station when the player presses the key.
///
/// Runs in `Update` during `AppState::InGame`.
/// Reads `ActiveActions` for `CockpitCyclePrev`.
/// Cycles backward through stations in JSON order.
#[allow(clippy::needless_pass_by_value)]
pub fn cockpit_station_cycle_prev_system(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    cockpit: Res<'_, CockpitOverlayResource>,
    mut active_station: ResMut<'_, ActiveCockpitStation>,
    query: Query<'_, '_, Entity, With<CockpitOverlay>>,
    active_actions: Res<'_, ActiveActions>,
) {
    // Check if the cycle prev action is pressed
    if !active_actions.0.contains(&LogicalAction::CockpitCyclePrev) {
        return;
    }

    let Ok(entity) = query.single() else {
        return;
    };

    // Find current station index
    let current_idx = cockpit
        .stations
        .iter()
        .position(|s| s.id == active_station.station_id)
        .unwrap_or(0);

    // Calculate previous index (wrap around)
    let prev_idx = if current_idx == 0 {
        cockpit.stations.len() - 1
    } else {
        current_idx - 1
    };
    let Some(prev_station) = cockpit.stations.get(prev_idx) else {
        return;
    };

    // Load the new texture
    let texture_handle = asset_server.load(&prev_station.texture);

    // Insert a new overlay component with the new texture
    commands.entity(entity).insert(CockpitOverlay {
        texture: texture_handle,
    });

    // Update the active station resource
    active_station.station_id.clone_from(&prev_station.id);

    tracing::debug!(
        "cockpit: switched to station '{}' ({})",
        prev_station.id,
        prev_station.texture
    );
}
