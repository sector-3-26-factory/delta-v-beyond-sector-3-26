// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Cockpit-related systems.

use bevy::prelude::*;

use super::{ActiveCockpitStation, CockpitOverlay};

/// Cycles to the next cockpit station when the player presses the key.
///
/// Runs in `Update` during `AppState::InGame`.
#[allow(clippy::needless_pass_by_value)]
pub fn cockpit_station_cycle_next_system(
    _active: ResMut<'_, ActiveCockpitStation>,
    _overlay: Query<'_, '_, &mut CockpitOverlay>,
    keyboard: &Res<'_, ButtonInput<KeyCode>>,
) {
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
pub fn cockpit_station_cycle_prev_system(
    _active: ResMut<'_, ActiveCockpitStation>,
    _overlay: Query<'_, '_, &mut CockpitOverlay>,
    keyboard: &Res<'_, ButtonInput<KeyCode>>,
) {
    if !keyboard.pressed(KeyCode::AltLeft) || !keyboard.pressed(KeyCode::F2) {
        return;
    }

    // TODO: Implement station switching logic
    // This will cycle backward through stations in JSON order
    log::debug!("cockpit: cycle prev station");
}
