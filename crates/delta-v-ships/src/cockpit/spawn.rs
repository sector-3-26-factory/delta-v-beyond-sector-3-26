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
use bevy::ui::widget::NodeImageMode;

use crate::ship_templates::CockpitStation;
use delta_v_core::RenderLayer;

use super::ActiveCockpitStation;

/// Spawns the cockpit overlay for the player ship.
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

    // The template_path is the asset directory relative to the assets/ root,
    // e.g., "templates/ships/space-fighter-comrade1280".
    // The station texture is relative to that directory, e.g., "cockpit/default.png".
    // The asset server loads from "assets/" + template_path + "/" + texture.
    let texture_path = format!("{}/{}", cockpit.template_path, station.texture);
    let texture_handle = asset_server.load::<Image>(texture_path);

    // Spawn a full-screen UI node with an ImageNode for the cockpit overlay.
    // ImageNode with Stretch mode fills the entire viewport regardless of image size.
    // The parent node uses PositionType::Absolute and ZIndex(100) to render on top.
    commands
        .spawn((
            Node {
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            RenderLayer::Ui.render_layers(),
            Visibility::Visible,
            super::CockpitOverlay {
                texture: texture_handle.clone(),
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode {
                    image: texture_handle,
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                Visibility::Visible,
            ));
        });

    commands.insert_resource(ActiveCockpitStation {
        station_id: station.id.clone(),
    });
}

/// Resource holding the cockpit overlay definition for the player ship.
#[derive(Resource)]
pub struct CockpitOverlayResource {
    /// Template asset directory path relative to the assets/ root
    /// (e.g., `templates/ships/space-fighter-comrade1280`).
    pub template_path: String,
    /// List of cockpit stations.
    pub stations: Vec<CockpitStation>,
}
