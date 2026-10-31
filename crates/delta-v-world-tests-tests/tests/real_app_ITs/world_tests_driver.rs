// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Strategy Z2 test driver: spawns the real `delta-v` binary with
//! `--test-function <name>` and checks the exit code.
//!
//! Run with: cargo test --test `world_tests_driver` -p delta-v-world-tests-tests

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::process::Command;
use std::time::{Duration, Instant};

fn run_test(name: &str, world: &str) -> Result<(), String> {
    let binary = delta_v_test_framework::ensure_binary_built();
    let mut child = Command::new(&binary)
        .arg("--test-function")
        .arg(name)
        .arg("--world")
        .arg(world)
        .arg("--asset-root")
        .arg("crates/delta-v-world/tests/fixtures/assets")
        .spawn()
        .map_err(|e| format!("failed to spawn: {e}"))?;

    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            if status.success() {
                return Ok(());
            }
            return Err(format!("exit code: {:?}", status.code()));
        }
        if Instant::now() > deadline {
            child.kill().ok();
            return Err("timeout".to_string());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn test_world_loading_pipeline() {
    run_test("world::world_loading_pipeline", "test").unwrap();
}

#[test]
fn test_player_controlled_validation_one() {
    run_test("world::player_controlled_validation_one", "test").unwrap();
}

#[test]
fn test_entity_validation() {
    run_test("world::entity_validation", "belt_field_test").unwrap();
}

#[test]
fn test_belt_streaming_happy_path() {
    run_test("world::belt_streaming_happy_path", "belt_field_test").unwrap();
}

#[test]
fn test_belt_streaming_player_far() {
    run_test("world::belt_streaming_player_far", "belt_field_test").unwrap();
}
