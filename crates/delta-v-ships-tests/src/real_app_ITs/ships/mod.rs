// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Ships real-app integration tests.
//!
//! Each test function receives a fully-built and render-settled `App`
//! from `main.rs` (which calls `settle_render(app, 60)` before
//! dispatching). Tests use plain `assert!` — a panic exits the process
//! with code 1, which the Z2 driver reads as failure.

#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::cargo
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
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use bevy::prelude::*;
use delta_v_types::LogicalAction;

/// Test: targeting pipeline — `CycleTargetNext` advances selection through navigation list.
///
/// Uses `press_and_update` to send `CycleTargetNext` through the real input pipeline.
/// Verifies that `cycle_target_system` reads `just_pressed`, advances the index
/// in `NavigationListData`, and updates `SelectedTarget`.
fn test_targeting_pipeline_cycle_next(app: &mut App) {
    // Verify navigation list is populated with 3 targetable ships
    let list_data = app.world().resource::<delta_v_core::NavigationListData>();
    assert!(
        list_data.entries.len() == 3,
        "navigation list should have 3 targetable entries"
    );

    // Capture entity IDs before any mutable borrows
    let first_entity = list_data.entries[0].entity;
    let second_entity = list_data.entries[1].entity;
    let third_entity = list_data.entries[2].entity;

    // Initial selection should be first entry (set by update_selection_system)
    let selected = app
        .world()
        .resource::<delta_v_ships::cockpit::components::SelectedTarget>();
    assert!(
        selected.0 == Some(first_entity),
        "SelectedTarget should start at first entry"
    );

    // Press CycleTargetNext — should advance to second entry
    delta_v_test_framework::press_and_update(app, LogicalAction::CycleTargetNext);

    let selected = app
        .world()
        .resource::<delta_v_ships::cockpit::components::SelectedTarget>();
    assert!(
        selected.0 == Some(second_entity),
        "CycleTargetNext should advance to second entry"
    );

    // Press again — should advance to third entry
    delta_v_test_framework::press_and_update(app, LogicalAction::CycleTargetNext);

    let selected = app
        .world()
        .resource::<delta_v_ships::cockpit::components::SelectedTarget>();
    assert!(
        selected.0 == Some(third_entity),
        "second CycleTargetNext should advance to third entry"
    );

    // Press again — should wrap to first entry
    delta_v_test_framework::press_and_update(app, LogicalAction::CycleTargetNext);

    let selected = app
        .world()
        .resource::<delta_v_ships::cockpit::components::SelectedTarget>();
    assert!(
        selected.0 == Some(first_entity),
        "third CycleTargetNext should wrap to first entry"
    );
}

/// Test: cockpit station loading — station from ship.json is loaded and `ActiveCockpitStation` is set.
///
/// The debug-ship-cube template includes a cockpit station definition.
/// After spawning, `ActiveCockpitStation` should be set to that station's ID.
fn test_cockpit_station_loading(app: &mut App) {
    // ActiveCockpitStation should be set by spawn_cockpit_overlay during OnEnter(InGame)
    let active_station = app
        .world()
        .get_resource::<delta_v_ships::cockpit::components::ActiveCockpitStation>();

    assert!(
        active_station.is_some(),
        "ActiveCockpitStation should be set after spawning"
    );

    // The debug-ship-cube template defines a station with ID "default"
    assert!(
        active_station.unwrap().station_id == "default",
        "station_id should match the template's station ID"
    );
}

/// Test: thrust/torque pipeline — input reader produces ThrustCommand/TorqueCommand,
/// and the thrust/torque systems apply forces to the player ship's [`RigidBody`].
///
/// Presses `ThrustForward` and verifies the ship's velocity moves forward (-Z).
/// Presses `PitchUp` and verifies the ship's angular velocity gains +X torque.
/// Verifies flight assist toggle works.
#[allow(clippy::float_cmp, clippy::doc_markdown)]
fn test_thrust_torque_pipeline(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    // Initial state — ship is stationary
    let ship_entity = app.world().resource::<PlayerShipEntity>().0;
    let ship = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        ship.velocity == Vec3::ZERO,
        "ship velocity should start at zero"
    );
    assert!(
        ship.angular_velocity == Vec3::ZERO,
        "ship angular velocity should start at zero"
    );

    // Press ThrustForward — should populate ThrustCommand.force and apply a
    // forward (-Z) force to the ship's RigidBody.
    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::ThrustForward);

    // Now get ship (immutable borrow)
    let ship = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        ship.velocity.z < 0.0,
        "ship velocity.z should be negative (forward) after ThrustForward"
    );

    // Press PitchUp — should populate TorqueCommand.torque and apply +X
    // angular velocity to the ship's RigidBody.
    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::PitchUp);

    let ship = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        ship.angular_velocity.x > 0.0,
        "ship angular_velocity.x should be positive (pitch up) after PitchUp"
    );

    // Toggle flight assist — should flip the state
    let flight_assist_before = app
        .world()
        .resource::<delta_v_core::FlightAssistState>()
        .enabled;
    delta_v_test_framework::press_and_update(app, LogicalAction::ToggleFlightAssist);

    let flight_assist_after = app
        .world()
        .resource::<delta_v_core::FlightAssistState>()
        .enabled;
    assert!(
        flight_assist_after != flight_assist_before,
        "ToggleFlightAssist should flip flight assist state"
    );
}

/// Register all ships test functions in the global registry.
///
/// Called from [`super::init_registry`] (the crate-level
/// `real_app_ITs::init_registry`).
#[cfg(feature = "test")]
pub fn init_registry() {
    do_register(
        "ships::targeting_pipeline_cycle_next",
        test_targeting_pipeline_cycle_next,
    );
    do_register(
        "ships::cockpit_station_loading",
        test_cockpit_station_loading,
    );
    do_register("ships::thrust_torque_pipeline", test_thrust_torque_pipeline);
}

#[cfg(feature = "test")]
fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
