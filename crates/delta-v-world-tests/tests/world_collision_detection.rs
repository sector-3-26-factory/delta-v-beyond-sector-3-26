// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Integration test for world loading collision detection.
//!
//! Registered as a `[[test]]` target with `harness = false` in Cargo.toml so
//! that `cargo test --test world_collision_detection` runs `main()` on the
//! **main thread**, which is required by Bevy's `WinitPlugin`.
//!
//! This test expects the world loading to PANIC because the fixture has
//! two entities at the same position. The panic is expected and indicates
//! the collision detection in world loading works correctly.
//!
//!   cargo test --test `world_collision_detection` -p delta-v-world
//!
//! See ADR-0019 (Asset pipeline), ADR-0020 (Save and load format), and
//! ADR-0060 (Test framework single source of truth).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_app::{AppConfig, build_app};
use delta_v_test_framework::make_app;
use std::panic;

/// Build the test `App` via `build_app` using the `test_collision` fixture.
fn build_test_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-world/tests/fixtures/assets".to_string(),
        world_name: Some("test_collision".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
}

/// Entry point — runs on the main thread so `WinitPlugin` can initialize.
/// This test EXPECTS a panic during world load.
fn main() {
    // Use catch_unwind to catch the expected panic
    let result = panic::catch_unwind(|| {
        let _app = make_app(60, build_test_app);
    });

    match result {
        Ok(()) => {
            // No panic - this is a test failure
            eprintln!(
                "FAIL: Expected panic for overlapping entities, but world loaded successfully"
            );
            std::process::exit(1);
        }
        Err(payload) => {
            // Check if it's the expected panic
            let msg = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("unknown panic");

            if msg.contains("overlapping collision shapes") {
                eprintln!("PASS: Expected panic caught for overlapping entities");
                std::process::exit(0);
            } else {
                eprintln!("FAIL: Unexpected panic: {msg}");
                std::process::exit(1);
            }
        }
    }
}
