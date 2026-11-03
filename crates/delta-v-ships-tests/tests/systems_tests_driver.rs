// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Strategy Z2 test driver: spawns the real `delta-v` binary with
//! `--test-function <name>` and checks the exit code.
//!
//! Run with: cargo test --test `systems_tests_driver` -p delta-v-ships-tests

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
        .arg("crates/delta-v-ships-tests/tests/fixtures")
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
fn test_input_reader_thrust_forward() {
    run_test("ships::input_reader_thrust_forward").unwrap();
}

#[test]
fn test_input_reader_thrust_backward() {
    run_test("ships::input_reader_thrust_backward").unwrap();
}

#[test]
fn test_input_reader_strafe_left() {
    run_test("ships::input_reader_strafe_left").unwrap();
}

#[test]
fn test_input_reader_strafe_right() {
    run_test("ships::input_reader_strafe_right").unwrap();
}

#[test]
fn test_input_reader_pitch_up() {
    run_test("ships::input_reader_pitch_up").unwrap();
}

#[test]
fn test_input_reader_yaw_left() {
    run_test("ships::input_reader_yaw_left").unwrap();
}

#[test]
fn test_input_reader_roll_left() {
    run_test("ships::input_reader_roll_left").unwrap();
}

#[test]
fn test_input_reader_multiple_actions() {
    run_test("ships::input_reader_multiple_actions").unwrap();
}

#[test]
fn test_input_reader_no_actions() {
    run_test("ships::input_reader_no_actions").unwrap();
}

#[test]
fn test_flight_assist_toggle_on_press() {
    run_test("ships::flight_assist_toggle_on_press").unwrap();
}

#[test]
fn test_flight_assist_no_toggle_on_hold() {
    run_test("ships::flight_assist_no_toggle_on_hold").unwrap();
}

#[test]
fn test_flight_assist_toggle_twice_returns_to_original() {
    run_test("ships::flight_assist_toggle_twice_returns_to_original").unwrap();
}

#[test]
fn test_flight_assist_damping_reduces_velocity() {
    run_test("ships::flight_assist_damping_reduces_velocity").unwrap();
}

#[test]
fn test_flight_assist_damping_disabled() {
    run_test("ships::flight_assist_damping_disabled").unwrap();
}

#[test]
fn test_clear_commands_resets_thrust_and_torque() {
    run_test("ships::clear_commands_resets_thrust_and_torque").unwrap();
}

#[test]
fn test_thrust_system_applies_force_to_rigid_body() {
    run_test("ships::thrust_system_applies_force_to_rigid_body").unwrap();
}

#[test]
fn test_thrust_system_zero_force_noop() {
    run_test("ships::thrust_system_zero_force_noop").unwrap();
}

#[test]
fn test_torque_system_applies_torque_to_rigid_body() {
    run_test("ships::torque_system_applies_torque_to_rigid_body").unwrap();
}

#[test]
fn test_ramp_first_tick() {
    run_test("ships::ramp_first_tick").unwrap();
}

#[test]
fn test_ramp_factor_with_different_ticks() {
    run_test("ships::ramp_factor_with_different_ticks").unwrap();
}

#[test]
fn test_ramp_ticks_one_is_full() {
    run_test("ships::ramp_ticks_one_is_full").unwrap();
}

#[test]
fn test_ramp_no_actions_zero() {
    run_test("ships::ramp_no_actions_zero").unwrap();
}

#[test]
fn test_ramp_active_then_inactive_same_tick() {
    run_test("ships::ramp_active_then_inactive_same_tick").unwrap();
}

#[test]
fn test_ramp_opposite_direction() {
    run_test("ships::ramp_opposite_direction").unwrap();
}

#[test]
fn test_ramp_zero_means_instant_full() {
    run_test("ships::ramp_zero_means_instant_full").unwrap();
}

#[test]
fn test_ramp_does_not_affect_thrust() {
    run_test("ships::ramp_does_not_affect_thrust").unwrap();
}

#[test]
fn test_ramp_axes_independent() {
    run_test("ships::ramp_axes_independent").unwrap();
}
