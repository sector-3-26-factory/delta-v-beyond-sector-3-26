// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Strategy Z2 test driver: spawns the real `delta-v` binary with
//! `--test-function <name>` and checks the exit code.
//!
//! Run with: cargo test --test `ships_tests_driver` -p delta-v-ships-tests

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::process::Command;
use std::time::{Duration, Instant};

fn run_test(name: &str) -> Result<(), String> {
    let binary = delta_v_test_framework::ensure_binary_built();
    let mut child = Command::new(&binary)
        .arg("--test-function")
        .arg(name)
        .arg("--world")
        .arg("test_targetable")
        .arg("--asset-root")
        .arg("crates/delta-v-ships/tests/fixtures/assets")
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
fn test_targeting_pipeline_cycle_next() {
    run_test("ships::targeting_pipeline_cycle_next").unwrap();
}

#[test]
fn test_cockpit_station_loading() {
    run_test("ships::cockpit_station_loading").unwrap();
}

#[test]
fn test_thrust_torque_pipeline() {
    run_test("ships::thrust_torque_pipeline").unwrap();
}
