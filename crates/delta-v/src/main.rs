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
//! See ADR-0005 (plugin architecture) and ADR-0060 (test framework single source of truth).
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

use clap::Parser;
use delta_v_app::{AppConfig, build_app};

/// Command-line arguments for Delta-V.
#[derive(Parser, Debug)]
#[command(name = "delta-v")]
#[command(
    about = "Delta-V beyond Sector 3.26 - a space simulation game",
    version
)]
struct Args {
    /// World name to load (directory under `assets/worlds/`).
    ///
    /// A world is a directory containing `world.json` plus optional assets
    /// such as lore (Markdown), intro video, background sound, etc.
    ///
    /// Examples:
    ///   - `inspect` → `assets/worlds/inspect/`
    ///   - `default` → `assets/worlds/default/`
    ///   - `custom` → `assets/worlds/custom/`
    #[arg(short, long, default_value = "default")]
    world: String,
}

fn main() {
    // Parse command-line arguments.
    let args = Args::parse();

    // World name is a short name (directory under assets/worlds/)
    // A world is a directory containing world.json plus optional assets
    // such as lore (Markdown), intro video, background sound, etc.
    let world_name = args.world;

    #[cfg(target_os = "linux")]
    print_x11_hint();

    let mut app = build_app(AppConfig {
        is_headless: false,
        asset_root_suffix: "assets".to_string(), // Production suffix
        world_name: Some(world_name),
        log_filter: None, // Default from build_app (dev/non-dev via feature)
    });
    app.run();
}

/// Prints a display-forwarding hint on Linux.
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
