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
#![allow(
    clippy::multiple_crate_versions,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]

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

    /// Asset root suffix (path fragment under workspace root).
    /// Production: "assets". Tests: "crates/delta-v-physics/tests/fixtures/assets".
    #[arg(long, default_value = "assets")]
    asset_root: String,

    /// Run a named test function from the registry instead of the event loop.
    ///
    /// The function must be registered via `init_registry()` in a per-domain
    /// test crate compiled into this binary behind the `test` feature flag.
    #[arg(long)]
    test_function: Option<String>,

    /// Run a named test function that drives the full event loop
    /// (`app.run()`) and reads the exit code. Unlike `--test-function`,
    /// the binary does NOT call `settle_render` before dispatching — the
    /// registered function is responsible for calling `app.run()` itself.
    ///
    /// Used by screenshot-based visual regression tests that need the
    /// `Update` schedule (e.g. `ui::fade_zones_visible`).
    #[arg(long)]
    test_run: Option<String>,

    /// Run a named test function that is EXPECTED to panic during
    /// `build_app` / `settle_render` in `main.rs`. The panic message
    /// must contain the substring registered via `TestEntry::panic_message`.
    ///
    /// Used by world-loading validation tests that expect a hard error
    /// (e.g. zero player-controlled entities, overlapping collision
    /// shapes). The binary wraps app construction in `catch_unwind`,
    /// so the panic never propagates to the driver.
    #[arg(long)]
    test_panic: Option<String>,
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

    // `--test-panic` tests expect `build_app` itself to panic (e.g.
    // world validation failure). They need `catch_unwind` around the
    // construction, so the app is built inside `run_test_panic`, not here.
    if let Some(test_name) = args.test_panic {
        run_test_panic(
            &AppConfig {
                asset_root_suffix: args.asset_root,
                world_name: Some(world_name),
                log_filter: None,
            },
            &test_name,
        );
        return;
    }

    let mut app = build_app(AppConfig {
        asset_root_suffix: args.asset_root,
        world_name: Some(world_name),
        log_filter: None, // Default from build_app (dev/non-dev via feature)
    });

    if let Some(test_name) = args.test_function {
        run_test_function(&mut app, &test_name);
    } else if let Some(test_name) = args.test_run {
        run_test_run(&mut app, &test_name);
    } else {
        app.run();
    }
}

/// Run a named test function from the registry.
///
/// The function must be registered via `init_registry()` in a per-domain
/// test crate compiled into this binary behind the `test` feature flag.
///
/// Exits 0 on success, 1 on failure.
#[cfg(feature = "test")]
fn run_test_function(app: &mut bevy::prelude::App, test_name: &str) {
    use delta_v_physics_tests::real_app_ITs::init_registry as init_physics;
    use delta_v_ships_tests::real_app_ITs::init_registry as init_ships;
    use delta_v_test_framework::all_tests;
    use delta_v_ui_tests::real_app_ITs::init_registry as init_ui;
    use delta_v_world_tests_tests::real_app_ITs::init_registry as init_world;

    // Settle the render graph so render-dependent resources
    // (RenderDevice, etc.) are available before the test function runs.
    delta_v_test_framework::settle_render(app, 60);

    // Register all test functions from every per-domain test crate
    // before looking up the requested one.
    init_physics();
    init_ships();
    init_ui();
    init_world();

    for entry in all_tests() {
        if entry.name == test_name {
            (entry.func)(app);
            std::process::exit(0);
        }
    }

    eprintln!("test function not found: {test_name}");
    std::process::exit(1);
}

/// Fallback when the `test` feature is not enabled.
#[cfg(not(feature = "test"))]
fn run_test_function(_app: &mut bevy::prelude::App, _test_name: &str) {
    eprintln!("test feature not enabled; rebuild with --features test");
    std::process::exit(1);
}

/// Run a named test function that drives the full event loop.
///
/// Unlike [`run_test_function`], this does NOT call `settle_render` —
/// the registered function is responsible for calling `app.run()` itself
/// (and reading the exit code). Used by screenshot-based visual regression
/// tests that need the `Update` schedule.
///
/// Exits 0 on success, 1 on failure.
#[cfg(feature = "test")]
fn run_test_run(app: &mut bevy::prelude::App, test_name: &str) {
    use delta_v_physics_tests::real_app_ITs::init_registry as init_physics;
    use delta_v_ships_tests::real_app_ITs::init_registry as init_ships;
    use delta_v_test_framework::all_tests;
    use delta_v_ui_tests::real_app_ITs::init_registry as init_ui;
    use delta_v_world_tests_tests::real_app_ITs::init_registry as init_world;

    // Register all test functions from every per-domain test crate
    // before looking up the requested one. Do NOT call settle_render —
    // the test function will call app.run() itself, which drives the
    // full event loop including finish()/cleanup().
    init_physics();
    init_ships();
    init_ui();
    init_world();

    for entry in all_tests() {
        if entry.name == test_name {
            (entry.func)(app);
            // If the test function returns without calling app.run()
            // or process::exit, treat it as success.
            std::process::exit(0);
        }
    }

    eprintln!("test run not found: {test_name}");
    std::process::exit(1);
}

/// Fallback when the `test` feature is not enabled.
#[cfg(not(feature = "test"))]
fn run_test_run(_app: &mut bevy::prelude::App, _test_name: &str) {
    eprintln!("test feature not enabled; rebuild with --features test");
    std::process::exit(1);
}

/// Run a named test function that is EXPECTED to panic during app
/// construction.
///
/// The binary wraps `build_app` + `settle_render` in `catch_unwind`.
/// If the panic message contains the registered substring, the test
/// passes (exit 0). If no panic occurs, or the message doesn't match,
/// the test fails (exit 1).
///
/// Used by world-loading validation tests that expect a hard error
/// (e.g. zero player-controlled entities, overlapping collision
/// shapes).
///
/// Exits 0 on expected panic, 1 on failure.
#[cfg(feature = "test")]
fn run_test_panic(config: &delta_v_app::AppConfig, test_name: &str) {
    use delta_v_physics_tests::real_app_ITs::init_registry as init_physics;
    use delta_v_ships_tests::real_app_ITs::init_registry as init_ships;
    use delta_v_test_framework::all_tests;
    use delta_v_ui_tests::real_app_ITs::init_registry as init_ui;
    use delta_v_world_tests_tests::real_app_ITs::init_registry as init_world;

    // Register all test functions before looking up the requested one.
    init_physics();
    init_ships();
    init_ui();
    init_world();

    // Find the entry to get its expected panic message.
    let expected_msg = all_tests()
        .iter()
        .find(|e| e.name == test_name)
        .map(|e| e.panic_message);

    let Some(expected_msg) = expected_msg else {
        eprintln!("test panic not found: {test_name}");
        std::process::exit(1);
    };

    // Build the app inside catch_unwind — the panic happens during
    // `build_app` / `settle_render`, before any test function runs.
    let result = std::panic::catch_unwind(|| {
        let mut app = build_app(config.clone());
        delta_v_test_framework::settle_render(&mut app, 60);
    });

    match result {
        Ok(()) => {
            eprintln!(
                "FAIL: expected panic for `{test_name}` (message: \"{expected_msg}\"), \
                 but app construction succeeded"
            );
            std::process::exit(1);
        }
        Err(payload) => {
            let msg = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("unknown panic");

            if msg.contains(expected_msg) {
                eprintln!("PASS: expected panic caught for `{test_name}`");
                std::process::exit(0);
            }

            eprintln!(
                "FAIL: unexpected panic for `{test_name}`: {msg} \
                 (expected substring: \"{expected_msg}\")"
            );
            std::process::exit(1);
        }
    }
}

/// Fallback when the `test` feature is not enabled.
#[cfg(not(feature = "test"))]
fn run_test_panic(_config: &delta_v_app::AppConfig, _test_name: &str) {
    eprintln!("test feature not enabled; rebuild with --features test");
    std::process::exit(1);
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
