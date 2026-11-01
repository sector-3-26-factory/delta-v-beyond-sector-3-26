// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Strategy Z2 test driver: spawns the real `delta-v` binary with
//! `--test-function <name>` and checks the exit code.
//!
//! Run with: cargo test --test `benchmarks_tests_driver` --features bench -p delta-v-physics-tests

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::process::Command;
use std::time::{Duration, Instant};

fn run_test(name: &str) -> Result<(), String> {
    let binary = delta_v_test_framework::ensure_binary_built();
    let mut child = Command::new(&binary)
        .arg("--test-function")
        .arg(name)
        .arg("--world")
        .arg("floating_origin")
        .arg("--asset-root")
        .arg("crates/delta-v-physics-tests/tests/fixtures")
        .spawn()
        .map_err(|e| format!("failed to spawn: {e}"))?;

    let deadline = Instant::now() + Duration::from_secs(300);
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
#[ignore = "performance benchmark; run explicitly with --ignored"]
fn test_benchmark_scaling() {
    run_test("physics::benchmark_scaling").unwrap();
}

#[test]
#[ignore = "performance benchmark; run explicitly with --ignored"]
fn test_benchmark_solar_system_simulation() {
    run_test("physics::benchmark_solar_system_simulation").unwrap();
}

#[test]
#[ignore = "performance benchmark; run explicitly with --ignored"]
fn test_benchmark_gravity_system_only() {
    run_test("physics::benchmark_gravity_system_only").unwrap();
}

#[test]
#[ignore = "performance benchmark; run explicitly with --ignored"]
fn test_benchmark_collision() {
    run_test("physics::benchmark_collision").unwrap();
}

#[test]
#[ignore = "performance benchmark; run explicitly with --ignored"]
fn test_benchmark_collision_relevance() {
    run_test("physics::benchmark_collision_relevance").unwrap();
}
