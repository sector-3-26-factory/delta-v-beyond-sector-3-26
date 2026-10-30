// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Strategy Z2 test driver: spawns the real `delta-v` binary with
// `--test-function <name>` and checks the exit code.
//
// Run with: cargo test --test sphere_sphere_tests_driver -p delta-v-physics

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
        .arg("crates/delta-v-physics/tests/fixtures/assets")
        .spawn()
        .map_err(|e| format!("failed to spawn: {e}"))?;

    let deadline = Instant::now() + Duration::from_secs(30);
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
fn test_sphere_sphere_overlap() {
    run_test("physics::sphere_sphere_overlap").unwrap();
}

#[test]
fn test_sphere_sphere_no_overlap() {
    run_test("physics::sphere_sphere_no_overlap").unwrap();
}

#[test]
fn test_sphere_sphere_touching_exactly() {
    run_test("physics::sphere_sphere_touching_exactly").unwrap();
}

#[test]
fn test_sphere_sphere_coincident_centers() {
    run_test("physics::sphere_sphere_coincident_centers").unwrap();
}
