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

//! Entry point for Delta-V beyond Sector 3.26.
//!
//! This binary is responsible for application composition only: it
//! registers every plugin in the correct order and starts the Bevy
//! event loop. No game logic lives here.
//!
//! See ADR-0005 (plugin architecture).
#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

use std::path::PathBuf;

use bevy::prelude::*;
use clap::Parser;
use delta_v_ai::AiPlugin;
use delta_v_assets::AssetsPlugin;
use delta_v_config::ConfigPlugin;
use delta_v_core::CorePlugin;
use delta_v_items::ItemsPlugin;
use delta_v_net::NetPlugin;
use delta_v_physics::PhysicsPlugin;
use delta_v_propulsion::PropulsionPlugin;
use delta_v_ships::ShipsPlugin;
use delta_v_stations::StationsPlugin;
use delta_v_ui::UiPlugin;
use delta_v_weapons::WeaponsPlugin;
use delta_v_world::{WorldPath, WorldPlugin};

/// Command-line arguments for Delta-V.
#[derive(Parser, Debug)]
#[command(name = "delta-v")]
#[command(
    about = "Delta-V beyond Sector 3.26 - a space simulation game",
    version
)]
struct Args {
    /// World name or path to load.
    ///
    /// If the value ends with `.world.json`, it's treated as a full path.
    /// Otherwise, it's treated as a short name and looked up in `assets/worlds/`.
    ///
    /// Examples:
    ///   - `inspect` → `assets/worlds/inspect.world.json`
    ///   - `default` → `assets/worlds/default.world.json`
    ///   - `custom` → `assets/worlds/custom.world.json`
    ///   - `path/to/my.world.json` → `path/to/my.world.json`
    #[arg(short, long, default_value = "assets/worlds/default.world.json")]
    world: String,
}

fn main() {
    // Parse command-line arguments.
    let args = Args::parse();

    // Resolve the world path.
    // If the argument doesn't end with ".world.json", treat it as a short name
    // and look for it in the worlds directory.
    let world_path = if args.world.ends_with(".world.json") {
        WorldPath(PathBuf::from(args.world))
    } else {
        WorldPath(PathBuf::from(format!(
            "assets/worlds/{}.world.json",
            args.world
        )))
    };

    // Print a display-forwarding hint before Bevy initialises the window.
    // If X11 is not reachable, Bevy will panic with XOpenDisplayFailed
    // immediately after; the hint tells the developer what to check.
    #[cfg(target_os = "linux")]
    print_x11_hint();

    let mut app = App::new();

    app.insert_resource(world_path).add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Delta-V beyond Sector 3.26".to_string(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            })
            .set(bevy::asset::AssetPlugin {
                // SAFETY: This is a simple path transformation. If the env var is not set,
                // we fall back to "assets" which is the expected relative path.
                #[allow(clippy::map_unwrap_or)]
                file_path: std::env::var("CARGO_MANIFEST_DIR")
                    .ok()
                    .map(|dir| {
                        std::path::PathBuf::from(dir)
                            .join("../..")
                            .join("assets")
                            .to_string_lossy()
                            .to_string()
                    })
                    .unwrap_or_else(|| "assets".to_string()),
                ..default()
            })
            .set(bevy::log::LogPlugin {
                #[cfg(not(feature = "dev"))]
                filter: "warn,delta_v=info,delta_v_core=info,delta_v_config=info,\
                             delta_v_physics=info,delta_v_assets=info,delta_v_ships=info,\
                             delta_v_propulsion=info,delta_v_weapons=info,delta_v_stations=info,\
                             delta_v_items=info,delta_v_world=info,delta_v_ai=info"
                    .to_string(),
                #[cfg(feature = "dev")]
                filter: "warn,delta_v=debug,delta_v_core=debug,delta_v_config=debug,\
                             delta_v_physics=debug,delta_v_assets=debug,delta_v_ships=debug,\
                             delta_v_propulsion=debug,delta_v_weapons=debug,delta_v_stations=debug,\
                             delta_v_items=debug,delta_v_world=debug,delta_v_ai=debug"
                    .to_string(),
                level: bevy::log::Level::TRACE,
                ..default()
            }),
    );

    // Technical plugins — must be added before domain plugins.
    // CorePlugin owns AppState and must come first.
    app.add_plugins(CorePlugin)
        .add_plugins(ConfigPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(AssetsPlugin)
        .add_plugins(NetPlugin);

    // Domain plugins.
    app.add_plugins(ShipsPlugin)
        .add_plugins(PropulsionPlugin)
        .add_plugins(WeaponsPlugin)
        .add_plugins(StationsPlugin)
        .add_plugins(ItemsPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(AiPlugin);

    app.add_plugins(UiPlugin);

    app.run();
}

/// Prints a display-forwarding hint on Linux so the developer sees
/// actionable steps if Bevy subsequently panics with `XOpenDisplayFailed`.
#[cfg(target_os = "linux")]
fn print_x11_hint() {
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| "(not set)".to_string());
    eprintln!(
        "\
[delta-v] Starting on Linux. DISPLAY={display}
[delta-v] If the next line panics with XOpenDisplayFailed, check:
[delta-v]   1. On the HOST run once per session:
[delta-v]        xhost +local:
[delta-v]   2. Compare DISPLAY on host vs inside the devcontainer:
[delta-v]        host:      echo $DISPLAY   (e.g. :0)
[delta-v]        container: echo $DISPLAY   (must match)
[delta-v]   3. If they differ, inside the devcontainer run:
[delta-v]        export DISPLAY=<host value>   (e.g. export DISPLAY=:0)
[delta-v]      then retry: cargo run --bin delta-v
[delta-v]   See .devcontainer/README.md for full troubleshooting."
    );
}
