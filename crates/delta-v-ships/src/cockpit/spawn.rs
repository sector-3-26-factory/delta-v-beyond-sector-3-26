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

//! Cockpit overlay spawning systems.

use bevy::prelude::*;

use crate::ship_templates::CockpitStation;

use super::ActiveCockpitStation;

/// Spawns the cockpit overlay for the player ship.
///
/// Runs during `OnEnter(AppState::InGame)`.
/// Reads the `CockpitOverlayResource` to get the cockpit definition.
/// Validates that `stations` is non-empty (hard error if empty).
/// Uses the first station as the default.
///
/// The overlay is rendered as a full-screen 2D UI sprite with alpha transparency,
/// allowing the 3D scene to show through transparent areas.
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_cockpit_overlay(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    cockpit: Res<'_, CockpitOverlayResource>,
) {
    let Some(station) = cockpit.stations.first() else {
        tracing::error!("cockpit.stations must contain at least one station");
        return;
    };

    let texture_handle = asset_server.load(&station.texture);

    // Spawn a full-screen UI node for the cockpit overlay container.
    // The sprite is spawned as a child of this node.
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    image: texture_handle.clone(),
                    ..default()
                },
                Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)),
                Visibility::default(),
                super::CockpitOverlay {
                    texture: texture_handle,
                },
            ));
        });

    // Insert the active station resource.
    commands.insert_resource(ActiveCockpitStation {
        station_id: station.id.clone(),
    });

    tracing::debug!("spawned cockpit overlay for station '{}'", station.id);
}

/// Resource holding the cockpit overlay definition for the player ship.
///
/// This resource is populated from the player ship template's cockpit definition
/// when the player ship is spawned.
#[derive(Resource)]
pub struct CockpitOverlayResource {
    /// List of cockpit stations.
    pub stations: Vec<CockpitStation>,
}
