// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Integration tests for ships plugin — real gameplay pipeline tests.
//!
//! Registered as a `[[test]]` target with `harness = false` in Cargo.toml so
//! that `cargo test --test ships_integration_tests` runs `main()` on the **main
//! thread**, which is required by Bevy's `WinitPlugin` (see
//! plans/testframework.md step 8).
//!
//!   cargo test --test `ships_integration_tests` -p delta-v-ships
//!
//! See ADR-0021 (Testing strategy) and ADR-0060 (Test framework single
//! source of truth).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_app::{AppConfig, build_app};
use delta_v_test_support::{check, press_and_fixed_update, press_and_update, run_harness};
use delta_v_types::LogicalAction;

/// Test: targeting pipeline — `CycleTargetNext` advances selection through navigation list.
///
/// Uses `press_and_update` to send `CycleTargetNext` through the real input pipeline.
/// Verifies that `cycle_target_system` reads `just_pressed`, advances the index
/// in `NavigationListData`, and updates `SelectedTarget`.
fn test_targeting_pipeline_cycle_next(app: &mut App) {
    // Verify navigation list is populated with 3 targetable ships
    let list_data = app.world().resource::<delta_v_core::NavigationListData>();
    check(
        list_data.entries.len() == 3,
        "navigation list should have 3 targetable entries",
    );

    // Capture entity IDs before any mutable borrows
    let first_entity = list_data.entries[0].entity;
    let second_entity = list_data.entries[1].entity;
    let third_entity = list_data.entries[2].entity;

    // Initial selection should be first entry (set by update_selection_system)
    let selected = app
        .world()
        .resource::<delta_v_ships::cockpit::components::SelectedTarget>();
    check(
        selected.0 == Some(first_entity),
        "SelectedTarget should start at first entry",
    );

    // Press CycleTargetNext — should advance to second entry
    press_and_update(app, LogicalAction::CycleTargetNext);

    let selected = app
        .world()
        .resource::<delta_v_ships::cockpit::components::SelectedTarget>();
    check(
        selected.0 == Some(second_entity),
        "CycleTargetNext should advance to second entry",
    );

    // Press again — should advance to third entry
    press_and_update(app, LogicalAction::CycleTargetNext);

    let selected = app
        .world()
        .resource::<delta_v_ships::cockpit::components::SelectedTarget>();
    check(
        selected.0 == Some(third_entity),
        "second CycleTargetNext should advance to third entry",
    );

    // Press again — should wrap to first entry
    press_and_update(app, LogicalAction::CycleTargetNext);

    let selected = app
        .world()
        .resource::<delta_v_ships::cockpit::components::SelectedTarget>();
    check(
        selected.0 == Some(first_entity),
        "third CycleTargetNext should wrap to first entry",
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

    check(
        active_station.is_some(),
        "ActiveCockpitStation should be set after spawning",
    );

    // The debug-ship-cube template defines a station with ID "default"
    check(
        active_station.unwrap().station_id == "default",
        "station_id should match the template's station ID",
    );
}

/// Test: thrust/torque pipeline — input reader produces ThrustCommand/TorqueCommand,
/// and the thrust/torque systems apply forces to the player ship's [`RigidBody`].
///
/// Presses `ThrustForward` and verifies the ship's velocity moves forward (-Z).
/// Presses `PitchUp` and verifies the ship's angular velocity gains +X torque.
/// Verifies flight assist toggle works.
fn test_thrust_torque_pipeline(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    // Initial state — ship is stationary
    let ship_entity = app.world().resource::<PlayerShipEntity>().0;
    let ship = app.world().get::<RigidBody>(ship_entity).unwrap();
    check(
        ship.velocity == Vec3::ZERO,
        "ship velocity should start at zero",
    );
    check(
        ship.angular_velocity == Vec3::ZERO,
        "ship angular velocity should start at zero",
    );

    // Press ThrustForward — should populate ThrustCommand.force and apply a
    // forward (-Z) force to the ship's RigidBody.
    press_and_fixed_update(app, LogicalAction::ThrustForward);

    // Query action state first (mutable borrow)
    let action_state = app
        .world_mut()
        .query::<&leafwing_input_manager::prelude::ActionState<LogicalAction>>()
        .single(app.world_mut())
        .unwrap();
    eprintln!(
        "DEBUG action_state: ThrustForward pressed={} just_pressed={}",
        action_state.pressed(&LogicalAction::ThrustForward),
        action_state.just_pressed(&LogicalAction::ThrustForward)
    );
    let thrust_cmd = app.world().resource::<delta_v_ships::ThrustCommand>();
    eprintln!("DEBUG thrust_cmd: force={:?}", thrust_cmd.force);
    let fixed_time = app
        .world()
        .resource::<bevy::time::Time<bevy::time::Fixed>>();
    let real_time = app.world().resource::<bevy::time::Time<bevy::time::Real>>();
    let virtual_time = app
        .world()
        .resource::<bevy::time::Time<bevy::time::Virtual>>();
    eprintln!(
        "DEBUG time: fixed_timestep={:?} real_delta={:?} virtual_delta={:?}",
        fixed_time.timestep(),
        real_time.delta(),
        virtual_time.delta()
    );
    // Now get ship (immutable borrow)
    let ship = app.world().get::<RigidBody>(ship_entity).unwrap();
    eprintln!(
        "DEBUG thrust: velocity={:?} angular={:?}",
        ship.velocity, ship.angular_velocity
    );
    check(
        ship.velocity.z < 0.0,
        "ship velocity.z should be negative (forward) after ThrustForward",
    );

    // Press PitchUp — should populate TorqueCommand.torque and apply +X
    // angular velocity to the ship's RigidBody.
    press_and_fixed_update(app, LogicalAction::PitchUp);

    // Query action state first (mutable borrow)
    let action_state = app
        .world_mut()
        .query::<&leafwing_input_manager::prelude::ActionState<LogicalAction>>()
        .single(app.world_mut())
        .unwrap();
    eprintln!(
        "DEBUG action_state: PitchUp pressed={} just_pressed={}",
        action_state.pressed(&LogicalAction::PitchUp),
        action_state.just_pressed(&LogicalAction::PitchUp)
    );
    let torque_cmd = app.world().resource::<delta_v_ships::TorqueCommand>();
    eprintln!("DEBUG torque_cmd: torque={:?}", torque_cmd.torque);
    let config = app
        .world()
        .resource::<delta_v_ships::ShipPropulsionConfig>();
    eprintln!(
        "DEBUG config: max_torque={} rotation_ramp_ticks={}",
        config.max_torque, config.rotation_ramp_ticks
    );
    let ramp = app
        .world()
        .resource::<delta_v_ships::systems::RotationRampState>();
    eprintln!("DEBUG ramp: ramp_ticks={:?}", ramp.ramp_ticks);
    // Now get ship (immutable borrow)
    let ship = app.world().get::<RigidBody>(ship_entity).unwrap();
    eprintln!(
        "DEBUG pitch: velocity={:?} angular={:?}",
        ship.velocity, ship.angular_velocity
    );
    check(
        ship.angular_velocity.x > 0.0,
        "ship angular_velocity.x should be positive (pitch up) after PitchUp",
    );

    // Toggle flight assist — should flip the state
    let flight_assist_before = app
        .world()
        .resource::<delta_v_core::FlightAssistState>()
        .enabled;
    press_and_update(app, LogicalAction::ToggleFlightAssist);

    let flight_assist_after = app
        .world()
        .resource::<delta_v_core::FlightAssistState>()
        .enabled;
    check(
        flight_assist_after != flight_assist_before,
        "ToggleFlightAssist should flip flight assist state",
    );
}

/// Entry point — runs on the main thread so `WinitPlugin` can initialize.
fn main() {
    let mut app = delta_v_test_support::make_app(60, || {
        build_app(AppConfig {
            asset_root_suffix: "crates/delta-v-ships/tests/fixtures/assets".to_string(),
            world_name: Some("test_targetable".to_string()),
            log_filter: Some("warn,delta_v=debug".to_string()),
        })
    });
    run_harness(
        &mut app,
        &[
            (
                "targeting_pipeline_cycle_next",
                test_targeting_pipeline_cycle_next,
            ),
            ("cockpit_station_loading", test_cockpit_station_loading),
            ("thrust_torque_pipeline", test_thrust_torque_pipeline),
        ],
    );
}
