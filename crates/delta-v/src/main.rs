// See AGENTS.md
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

//! Entry point for Delta-V beyond Sector 3.26.
//!
//! This binary orchestrates all plugins and is responsible for
//! application composition only. See ADR-0005 for the plugin architecture.

use bevy::prelude::*;
use delta_v_core::CorePlugin;
use delta_v_config::ConfigPlugin;
use delta_v_physics::PhysicsPlugin;
use delta_v_assets::AssetsPlugin;
use delta_v_net::NetPlugin;
use delta_v_ships::ShipsPlugin;
use delta_v_propulsion::PropulsionPlugin;
use delta_v_weapons::WeaponsPlugin;
use delta_v_stations::StationsPlugin;
use delta_v_items::ItemsPlugin;
use delta_v_world::WorldPlugin;

fn main() {
    env_logger::init();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Delta-V beyond Sector 3.26".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Core and technical plugins
        .add_plugins(CorePlugin)
        .add_plugins(ConfigPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(AssetsPlugin)
        .add_plugins(NetPlugin)
        // Domain plugins
        .add_plugins(ShipsPlugin)
        .add_plugins(PropulsionPlugin)
        .add_plugins(WeaponsPlugin)
        .add_plugins(StationsPlugin)
        .add_plugins(ItemsPlugin)
        .add_plugins(WorldPlugin)
        // Startup systems
        .add_systems(Startup, setup)
        .run();
}

/// Minimal startup system: spawns a camera so the window has something
/// well-defined to render. Real scene setup will come in later milestones.
fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
