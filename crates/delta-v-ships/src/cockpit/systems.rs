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
use delta_v_types::LogicalAction;

use super::ActiveCockpitStation;
use super::components::CockpitOverlay;
use super::spawn::CockpitOverlayResource;

/// Tracks which actions were already consumed to prevent repeated firing.
// allow-default: Bevy requires Default on resources for init_resource. This
// resource tracks key press state for edge detection; it starts false.
#[derive(Resource, Default)]
pub struct CockpitCycleState {
    /// Whether `CockpitCycleNext` was active last tick.
    next_active: bool,
    /// Whether `CockpitCyclePrev` was active last tick.
    prev_active: bool,
}

/// Cycles to the next or previous cockpit station when the player presses the key.
///
/// Runs in `Update` during `AppState::InGame`.
/// Reads `ActiveActions` for `CockpitCycleNext` or `CockpitCyclePrev`.
/// Cycles through stations in JSON order (wrapping around), loading the new station's PNG texture.
///
/// Uses edge detection to fire only once per key press, not every frame while held.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn cockpit_station_cycle_system(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    cockpit: Res<'_, CockpitOverlayResource>,
    mut active_station: ResMut<'_, ActiveCockpitStation>,
    query: Query<'_, '_, (Entity, &'static Children), With<CockpitOverlay>>,
    mut image_node_query: Query<'_, '_, &'static mut ImageNode>,
    active_actions: Res<'_, ActiveActions>,
    mut cycle_state: ResMut<'_, CockpitCycleState>,
) {
    let next_active = active_actions.0.contains(&LogicalAction::CockpitCycleNext);
    let prev_active = active_actions.0.contains(&LogicalAction::CockpitCyclePrev);

    // Edge detection: only fire on the frame the key is first pressed
    let direction: i32 = if next_active && !cycle_state.next_active {
        1
    } else if prev_active && !cycle_state.prev_active {
        -1
    } else {
        cycle_state.next_active = next_active;
        cycle_state.prev_active = prev_active;
        return;
    };

    cycle_state.next_active = next_active;
    cycle_state.prev_active = prev_active;

    let Ok((entity, children)) = query.single() else {
        tracing::warn!("[cockpit] cycle: no CockpitOverlay entity found");
        return;
    };

    let current_idx = cockpit
        .stations
        .iter()
        .position(|s| s.id == active_station.station_id)
        .unwrap_or(0);

    let station_count = cockpit.stations.len();
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let new_idx = ((current_idx as i32 + direction).rem_euclid(station_count as i32)) as usize;
    let Some(new_station) = cockpit.stations.get(new_idx) else {
        return;
    };

    let texture_path = format!("{}/{}", cockpit.template_path, new_station.texture);
    let texture_handle = asset_server.load(&texture_path);

    for child in children {
        if let Ok(mut image_node) = image_node_query.get_mut(*child) {
            image_node.image = texture_handle.clone();
        }
    }

    commands.entity(entity).insert(CockpitOverlay {
        texture: texture_handle,
    });

    active_station.station_id.clone_from(&new_station.id);

    tracing::debug!(
        "[cockpit] switched to station '{}' ({})",
        new_station.id,
        new_station.texture
    );
}
