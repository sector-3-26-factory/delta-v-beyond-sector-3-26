// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Ships input → forces pipeline tests (converted from `delta-v-ships` unit tests).
//!
//! Each test receives a fully-built and render-settled `App` from `main.rs`
//! (which calls `settle_render(app, 60)` before dispatching). Tests use plain
//! `assert!` — a panic exits the process with code 1, which the Z2 driver
//! reads as failure.
//!
//! The original unit tests in `crates/delta-v-ships/src/systems_tests.rs`
//! (which built a minimal `App::new()` + `TimePlugin`) are now redundant
//! with the real-app versions here and have been removed.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    clippy::doc_markdown,
    unsafe_code,
    dead_code
)]

use bevy::prelude::*;
use delta_v_types::LogicalAction;

const MAX_FORWARD_THRUST: f32 = 100_000.0;
const MAX_BACKWARD_THRUST: f32 = 40_000.0;
const MAX_TORQUE: f32 = 50_000.0;
const MAX_STRAFE_THRUST: f32 = 50_000.0;

// ---------------------------------------------------------------------------
// input_reader_system
// ---------------------------------------------------------------------------

fn test_input_reader_thrust_forward(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;

    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::ThrustForward);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    // ThrustCommand is cleared by clear_commands_system at end of tick
    // Assert on velocity instead (integrated from force)
    assert!(
        body.velocity.z < 0.0,
        "ship should have negative Z velocity from forward thrust, got {}",
        body.velocity.z
    );
}

fn test_input_reader_thrust_backward(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;

    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::ThrustBackward);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.velocity.z > 0.0,
        "ship should have positive Z velocity from backward thrust, got {}",
        body.velocity.z
    );
}

fn test_input_reader_strafe_left(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;

    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::StrafeLeft);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.velocity.x < 0.0,
        "ship should have negative X velocity from strafe left, got {}",
        body.velocity.x
    );
}

fn test_input_reader_strafe_right(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;

    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::StrafeRight);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.velocity.x > 0.0,
        "ship should have positive X velocity from strafe right, got {}",
        body.velocity.x
    );
}

fn test_input_reader_pitch_up(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;

    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::PitchUp);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    // TorqueCommand is cleared by clear_commands_system at end of tick
    // Assert on angular_velocity instead (integrated from torque)
    assert!(
        body.angular_velocity.x > 0.0,
        "ship should have positive X angular velocity from pitch up, got {}",
        body.angular_velocity.x
    );
}

fn test_input_reader_yaw_left(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;

    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::YawLeft);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.angular_velocity.y > 0.0,
        "ship should have positive Y angular velocity from yaw left, got {}",
        body.angular_velocity.y
    );
}

fn test_input_reader_roll_left(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;

    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::RollLeft);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.angular_velocity.z > 0.0,
        "ship should have positive Z angular velocity from roll left, got {}",
        body.angular_velocity.z
    );
}

fn test_input_reader_multiple_actions(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;

    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::ThrustForward);
    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::YawLeft);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.velocity.z < 0.0,
        "forward thrust should be applied, got velocity.z={}",
        body.velocity.z
    );
    assert!(
        body.angular_velocity.y > 0.0,
        "yaw torque should be applied, got angular_velocity.y={}",
        body.angular_velocity.y
    );
}

fn test_input_reader_no_actions(app: &mut App) {
    delta_v_test_framework::run_fixed_update(app);
    let thrust = app.world().resource::<delta_v_ships::ThrustCommand>();
    let torque = app.world().resource::<delta_v_ships::TorqueCommand>();
    assert_eq!(thrust.force, Vec3::ZERO, "no thrust with no actions");
    assert_eq!(torque.torque, Vec3::ZERO, "no torque with no actions");
}

// ---------------------------------------------------------------------------
// flight_assist_toggle_system
// ---------------------------------------------------------------------------

fn test_flight_assist_toggle_on_press(app: &mut App) {
    {
        let mut state = app
            .world_mut()
            .resource_mut::<delta_v_core::FlightAssistState>();
        state.enabled = false;
    }
    delta_v_test_framework::press_and_update(app, LogicalAction::ToggleFlightAssist);
    let state = app.world().resource::<delta_v_core::FlightAssistState>();
    assert!(state.enabled, "flight assist should be toggled on");
}

fn test_flight_assist_no_toggle_on_hold(app: &mut App) {
    {
        let mut state = app
            .world_mut()
            .resource_mut::<delta_v_core::FlightAssistState>();
        state.enabled = false;
    }
    delta_v_test_framework::press_and_update(app, LogicalAction::ToggleFlightAssist);
    assert!(
        app.world()
            .resource::<delta_v_core::FlightAssistState>()
            .enabled,
        "should toggle on first press"
    );
    delta_v_test_framework::run_fixed_update(app);
    assert!(
        app.world()
            .resource::<delta_v_core::FlightAssistState>()
            .enabled,
        "should remain enabled while holding (no re-toggle)"
    );
}

fn test_flight_assist_toggle_twice_returns_to_original(app: &mut App) {
    // Start enabled, toggle twice → back to enabled.
    {
        let mut state = app
            .world_mut()
            .resource_mut::<delta_v_core::FlightAssistState>();
        state.enabled = true;
    }
    // First press: toggle off. press_and_update releases then presses, so the
    // key is released between the two presses (required for just_pressed edge
    // detection on the second press).
    delta_v_test_framework::press_and_update(app, LogicalAction::ToggleFlightAssist);
    assert!(
        !app.world()
            .resource::<delta_v_core::FlightAssistState>()
            .enabled,
        "flight assist should be toggled off on first press"
    );
    // Second press: toggle back on.
    delta_v_test_framework::press_and_update(app, LogicalAction::ToggleFlightAssist);
    assert!(
        app.world()
            .resource::<delta_v_core::FlightAssistState>()
            .enabled,
        "flight assist should be toggled on on second press"
    );
}

// ---------------------------------------------------------------------------
// flight_assist_damping_system
// ---------------------------------------------------------------------------

fn test_flight_assist_damping_reduces_velocity(app: &mut App) {
    use delta_v_core::{FlightAssistConfig, PlayerShipEntity};
    use delta_v_physics::RigidBody;

    app.insert_resource(delta_v_core::FlightAssistState { enabled: true });
    app.insert_resource(FlightAssistConfig {
        enabled_by_default: true,
        damping_coefficient: 0.1,
    });

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;

    {
        let mut body = app.world_mut().get_mut::<RigidBody>(ship_entity).unwrap();
        body.velocity = Vec3::new(10.0, 5.0, 3.0);
    }

    delta_v_test_framework::reset_fixed_time(app);
    delta_v_test_framework::run_fixed_update(app);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        (body.velocity.x - 9.0).abs() < 0.01,
        "velocity should be damped, got x={}",
        body.velocity.x
    );
    assert!(
        (body.velocity.y - 4.5).abs() < 0.01,
        "velocity should be damped, got y={}",
        body.velocity.y
    );
}

fn test_flight_assist_damping_disabled(app: &mut App) {
    use delta_v_core::{FlightAssistConfig, PlayerShipEntity};
    use delta_v_physics::RigidBody;

    app.insert_resource(delta_v_core::FlightAssistState { enabled: false });
    app.insert_resource(FlightAssistConfig {
        enabled_by_default: false,
        damping_coefficient: 0.1,
    });

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;

    {
        let mut body = app.world_mut().get_mut::<RigidBody>(ship_entity).unwrap();
        body.velocity = Vec3::new(10.0, 5.0, 3.0);
    }

    delta_v_test_framework::reset_fixed_time(app);
    delta_v_test_framework::run_fixed_update(app);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        (body.velocity.x - 10.0).abs() < 0.01,
        "velocity should not be damped when disabled, got x={}",
        body.velocity.x
    );
}

// ---------------------------------------------------------------------------
// clear_commands_system
// ---------------------------------------------------------------------------

fn test_clear_commands_resets_thrust_and_torque(app: &mut App) {
    {
        let mut thrust = app
            .world_mut()
            .resource_mut::<delta_v_ships::ThrustCommand>();
        thrust.force = Vec3::new(100.0, 200.0, 300.0);
    }
    {
        let mut torque = app
            .world_mut()
            .resource_mut::<delta_v_ships::TorqueCommand>();
        torque.torque = Vec3::new(50.0, 60.0, 70.0);
    }
    delta_v_test_framework::run_fixed_update(app);
    let thrust = app.world().resource::<delta_v_ships::ThrustCommand>();
    let torque = app.world().resource::<delta_v_ships::TorqueCommand>();
    assert_eq!(thrust.force, Vec3::ZERO, "thrust should be cleared");
    assert_eq!(torque.torque, Vec3::ZERO, "torque should be cleared");
}

// ---------------------------------------------------------------------------
// thrust_system / torque_system
// ---------------------------------------------------------------------------

fn test_thrust_system_applies_force_to_rigid_body(app: &mut App) {
    use delta_v_core::{EntityType, PlayerShipEntity, Targetable, WorldEntityId};
    use delta_v_physics::{CollisionShape, RigidBody};

    let ship_entity = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            Targetable,
            EntityType("ship".to_string()),
            WorldEntityId("test_ship".to_string()),
            CollisionShape::sphere(1.0, Vec3::ZERO),
        ))
        .id();
    app.insert_resource(PlayerShipEntity(ship_entity));

    {
        let mut thrust = app
            .world_mut()
            .resource_mut::<delta_v_ships::ThrustCommand>();
        thrust.force = Vec3::new(0.0, 0.0, -100_000.0);
    }

    delta_v_test_framework::run_fixed_update(app);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    // ThrustCommand is cleared by clear_commands_system at end of tick
    // Assert on velocity instead (integrated from force)
    assert!(
        body.velocity.z < 0.0,
        "ship should have negative Z velocity from forward thrust, got {}",
        body.velocity.z
    );
}

fn test_thrust_system_zero_force_noop(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;

    delta_v_test_framework::run_fixed_update(app);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert_eq!(
        body.velocity,
        Vec3::ZERO,
        "zero thrust should not apply force"
    );
}

fn test_torque_system_applies_torque_to_rigid_body(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    let ship_entity = app.world().resource::<PlayerShipEntity>().0;
    {
        let mut torque = app
            .world_mut()
            .resource_mut::<delta_v_ships::TorqueCommand>();
        torque.torque = Vec3::new(50_000.0, 0.0, 0.0);
    }

    delta_v_test_framework::run_fixed_update(app);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    // TorqueCommand is cleared by clear_commands_system at end of tick
    // Assert on angular_velocity instead (integrated from torque)
    assert!(
        body.angular_velocity.x > 0.0,
        "ship should have positive X angular velocity from torque, got {}",
        body.angular_velocity.x
    );
}

// ---------------------------------------------------------------------------
// rotation force ramp
// ---------------------------------------------------------------------------
//
// The debug-ship-cube template uses `rcs-standard`, whose
// `rotation_ramp_ticks` is 60. These tests override `ShipPropulsionConfig`
// to drive the ramp at a known value.

fn override_propulsion(app: &mut App, rotation_ramp_ticks: u32) {
    app.insert_resource(delta_v_ships::ShipPropulsionConfig {
        max_forward_thrust: MAX_FORWARD_THRUST,
        max_backward_thrust: MAX_BACKWARD_THRUST,
        max_torque: MAX_TORQUE,
        max_strafe_thrust: MAX_STRAFE_THRUST,
        active_main_thruster_index: 0,
        rotation_ramp_ticks,
        thrust_sound: None,
    });
}

/// Reset the `RotationRampState` resource to its default (zero) state.
///
/// `RotationRampState` is a Bevy `Resource` that persists across tests in the
/// shared `App`. Without resetting it, the ramp counter from a previous test
/// leaks into the next test's assertion on `ramp_ticks`.
fn reset_ramp_state(app: &mut App) {
    app.world_mut()
        .resource_mut::<delta_v_ships::systems::RotationRampState>()
        .ramp_ticks = Vec3::ZERO;
}

fn test_ramp_first_tick(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    override_propulsion(app, 10);
    reset_ramp_state(app);
    let ship_entity = app.world().resource::<PlayerShipEntity>().0;
    delta_v_test_framework::reset_fixed_time(app);
    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::YawLeft);
    // TorqueCommand is cleared by clear_commands_system at end of tick
    // Assert on angular_velocity instead (integrated from torque)
    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.angular_velocity.y > 0.0,
        "first tick should produce positive Y angular velocity (ramp 10%), got {}",
        body.angular_velocity.y
    );
    let ramp = app
        .world()
        .resource::<delta_v_ships::systems::RotationRampState>();
    assert!(
        (ramp.ramp_ticks.y - 1.0).abs() < 0.01,
        "ramp counter should be 1 after first tick, got {}",
        ramp.ramp_ticks.y
    );
}

fn test_ramp_factor_with_different_ticks(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    override_propulsion(app, 5);
    reset_ramp_state(app);
    let ship_entity = app.world().resource::<PlayerShipEntity>().0;
    delta_v_test_framework::reset_fixed_time(app);
    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::YawLeft);
    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.angular_velocity.y > 0.0,
        "first tick with ramp=5 should produce positive Y angular velocity (20%), got {}",
        body.angular_velocity.y
    );
    let ramp = app
        .world()
        .resource::<delta_v_ships::systems::RotationRampState>();
    assert!(
        (ramp.ramp_ticks.y - 1.0).abs() < 0.01,
        "ramp counter should be 1, got {}",
        ramp.ramp_ticks.y
    );
}

fn test_ramp_ticks_one_is_full(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    override_propulsion(app, 1);
    let ship_entity = app.world().resource::<PlayerShipEntity>().0;
    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::YawLeft);
    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.angular_velocity.y > 0.0,
        "ramp_ticks=1 should give full torque (positive Y angular velocity), got {}",
        body.angular_velocity.y
    );
}

fn test_ramp_no_actions_zero(app: &mut App) {
    override_propulsion(app, 10);
    delta_v_test_framework::run_fixed_update(app);
    let torque = app.world().resource::<delta_v_ships::TorqueCommand>();
    let ramp = app
        .world()
        .resource::<delta_v_ships::systems::RotationRampState>();
    assert_eq!(
        torque.torque.y, 0.0,
        "no actions should produce zero torque"
    );
    assert_eq!(
        ramp.ramp_ticks.y, 0.0,
        "no actions should produce zero ramp counter"
    );
}

fn test_ramp_active_then_inactive_same_tick(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    override_propulsion(app, 10);
    reset_ramp_state(app);
    let ship_entity = app.world().resource::<PlayerShipEntity>().0;
    delta_v_test_framework::reset_fixed_time(app);
    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::YawLeft);
    let ramp = app
        .world()
        .resource::<delta_v_ships::systems::RotationRampState>();
    assert!(
        (ramp.ramp_ticks.y - 1.0).abs() < 0.01,
        "ramp should be at 1 after one tick, got {}",
        ramp.ramp_ticks.y
    );
    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.angular_velocity.y > 0.0,
        "torque should produce positive Y angular velocity (10%), got {}",
        body.angular_velocity.y
    );
}

fn test_ramp_opposite_direction(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    override_propulsion(app, 10);
    let ship_entity = app.world().resource::<PlayerShipEntity>().0;
    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::YawRight);
    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.angular_velocity.y < 0.0,
        "yaw right should produce negative Y angular velocity, got {}",
        body.angular_velocity.y
    );
}

fn test_ramp_zero_means_instant_full(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    override_propulsion(app, 0);
    let ship_entity = app.world().resource::<PlayerShipEntity>().0;
    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::YawLeft);
    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.angular_velocity.y > 0.0,
        "zero ramp ticks should give full torque instantly (positive Y angular velocity), got {}",
        body.angular_velocity.y
    );
}

fn test_ramp_does_not_affect_thrust(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    override_propulsion(app, 10);
    let ship_entity = app.world().resource::<PlayerShipEntity>().0;
    delta_v_test_framework::press_and_fixed_update(app, LogicalAction::ThrustForward);
    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.velocity.z < 0.0,
        "thrust should be full force (negative Z velocity), got {}",
        body.velocity.z
    );
}

fn test_ramp_axes_independent(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::RigidBody;

    override_propulsion(app, 10);
    reset_ramp_state(app);
    let ship_entity = app.world().resource::<PlayerShipEntity>().0;
    // Press both PitchUp and YawLeft in the same tick. Use `press_key`
    // directly (NOT `press_action`, which calls `app.update()` and would run
    // a FixedMain pass between the two presses, incrementing the ramp counter
    // twice). Write both KeyboardInput messages, then advance one fixed tick
    // so the input reader sees both actions held simultaneously.
    delta_v_test_framework::reset_fixed_time(app);
    delta_v_test_framework::press_key(app, LogicalAction::PitchUp);
    delta_v_test_framework::press_key(app, LogicalAction::YawLeft);
    delta_v_test_framework::advance_one_fixed_tick(app);
    app.update();
    let ramp = app
        .world()
        .resource::<delta_v_ships::systems::RotationRampState>();
    assert!(
        (ramp.ramp_ticks.x - 1.0).abs() < 0.01,
        "pitch ramp should be at 1, got {}",
        ramp.ramp_ticks.x
    );
    assert!(
        (ramp.ramp_ticks.y - 1.0).abs() < 0.01,
        "yaw ramp should be at 1, got {}",
        ramp.ramp_ticks.y
    );
    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert!(
        body.angular_velocity.x > 0.0,
        "pitch torque should produce positive X angular velocity (10%), got {}",
        body.angular_velocity.x
    );
    assert!(
        body.angular_velocity.y > 0.0,
        "yaw torque should produce positive Y angular velocity (10%), got {}",
        body.angular_velocity.y
    );
    // Release both actions so they don't leak into the next test.
    {
        let mut query = app
            .world_mut()
            .query::<&mut leafwing_input_manager::prelude::ActionState<LogicalAction>>();
        if let Ok(mut action_state) = query.single_mut(app.world_mut()) {
            *action_state =
                leafwing_input_manager::prelude::ActionState::<LogicalAction>::default();
        }
    }
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Register all ships systems test functions in the global registry.
#[cfg(feature = "test")]
pub fn init_registry() {
    do_register(
        "ships::input_reader_thrust_forward",
        test_input_reader_thrust_forward,
    );
    do_register(
        "ships::input_reader_thrust_backward",
        test_input_reader_thrust_backward,
    );
    do_register(
        "ships::input_reader_strafe_left",
        test_input_reader_strafe_left,
    );
    do_register(
        "ships::input_reader_strafe_right",
        test_input_reader_strafe_right,
    );
    do_register("ships::input_reader_pitch_up", test_input_reader_pitch_up);
    do_register("ships::input_reader_yaw_left", test_input_reader_yaw_left);
    do_register("ships::input_reader_roll_left", test_input_reader_roll_left);
    do_register(
        "ships::input_reader_multiple_actions",
        test_input_reader_multiple_actions,
    );
    do_register(
        "ships::input_reader_no_actions",
        test_input_reader_no_actions,
    );
    do_register(
        "ships::flight_assist_toggle_on_press",
        test_flight_assist_toggle_on_press,
    );
    do_register(
        "ships::flight_assist_no_toggle_on_hold",
        test_flight_assist_no_toggle_on_hold,
    );
    do_register(
        "ships::flight_assist_toggle_twice_returns_to_original",
        test_flight_assist_toggle_twice_returns_to_original,
    );
    do_register(
        "ships::flight_assist_damping_reduces_velocity",
        test_flight_assist_damping_reduces_velocity,
    );
    do_register(
        "ships::flight_assist_damping_disabled",
        test_flight_assist_damping_disabled,
    );
    do_register(
        "ships::clear_commands_resets_thrust_and_torque",
        test_clear_commands_resets_thrust_and_torque,
    );
    do_register(
        "ships::thrust_system_applies_force_to_rigid_body",
        test_thrust_system_applies_force_to_rigid_body,
    );
    do_register(
        "ships::thrust_system_zero_force_noop",
        test_thrust_system_zero_force_noop,
    );
    do_register(
        "ships::torque_system_applies_torque_to_rigid_body",
        test_torque_system_applies_torque_to_rigid_body,
    );
    do_register("ships::ramp_first_tick", test_ramp_first_tick);
    do_register(
        "ships::ramp_factor_with_different_ticks",
        test_ramp_factor_with_different_ticks,
    );
    do_register("ships::ramp_ticks_one_is_full", test_ramp_ticks_one_is_full);
    do_register("ships::ramp_no_actions_zero", test_ramp_no_actions_zero);
    do_register(
        "ships::ramp_active_then_inactive_same_tick",
        test_ramp_active_then_inactive_same_tick,
    );
    do_register(
        "ships::ramp_opposite_direction",
        test_ramp_opposite_direction,
    );
    do_register(
        "ships::ramp_zero_means_instant_full",
        test_ramp_zero_means_instant_full,
    );
    do_register(
        "ships::ramp_does_not_affect_thrust",
        test_ramp_does_not_affect_thrust,
    );
    do_register("ships::ramp_axes_independent", test_ramp_axes_independent);
}

/// Register a ships systems test function in the global registry.
///
/// The entry is leaked so it lives for the lifetime of the process.
#[cfg(feature = "test")]
fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry {
        name,
        func,
        panic_expected: false,
        panic_message: "",
    };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
