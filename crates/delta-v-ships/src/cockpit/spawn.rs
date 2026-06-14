// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Cockpit overlay spawning systems.

use bevy::prelude::*;

use delta_v_core::PlayerShipEntity;

use super::{ActiveCockpitStation, CockpitOverlay, CockpitOverlayEntity};

/// Spawns the cockpit overlay for the player ship.
///
/// Runs during `OnEnter(AppState::InGame)`.
/// Reads the `CockpitOverlayResource` to get the cockpit definition.
/// Validates that `stations` is non-empty (hard error if empty).
/// Uses the first station as the default.
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_cockpit_overlay(
    mut commands: Commands<'_, '_>,
    asset_server: &Res<'_, AssetServer>,
    _ship_entity: &Res<'_, PlayerShipEntity>,
    cockpit: &Res<'_, CockpitOverlayResource>,
) {
    let Some(station) = cockpit.stations.first() else {
        log::error!("cockpit.stations must contain at least one station");
        return;
    };

    let texture_handle = asset_server.load(&station.texture);

    commands.spawn((
        CockpitOverlay {
            texture: texture_handle,
        },
        CockpitOverlayEntity,
    ));

    commands.insert_resource(ActiveCockpitStation {
        station_id: station.id.clone(),
    });

    log::debug!("spawned cockpit overlay for station '{}'", station.id);
}

/// Resource holding the cockpit overlay definition for the player ship.
#[derive(Resource)]
pub struct CockpitOverlayResource {
    /// List of cockpit stations.
    pub stations: Vec<crate::ship_templates::CockpitStation>,
}
