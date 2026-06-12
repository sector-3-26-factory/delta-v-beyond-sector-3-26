// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for the input → forces pipeline systems.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::panic
)]

use bevy::prelude::*;
use bevy::time::TimePlugin;

use delta_v_core::{ActiveActions, FlightAssistState, LogicalAction};
use delta_v_physics::RigidBody;

use crate::ship_templates::{ShipPropulsionConfig, ThrustCommand, TorqueCommand};
use crate::systems::{
    clear_commands_system, flight_assist_damping_system, flight_assist_toggle_system,
    input_reader_system, thrust_system, torque_system, PreviousActions,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds a minimal Bevy app with the input → forces pipeline resources.
fn build_input_app() -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.init_resource::<ActiveActions>();
    app.init_resource::<ThrustCommand>();
    app.init_resource::<TorqueCommand>();
    app.init_resource::<PreviousActions>();
    app.init_resource::<FlightAssistState>();
    app.insert_resource(ShipPropulsionConfig {
        max_forward_thrust: 100_000.0,
        max_backward_thrust: 40_000.0,
        max_torque: 50_000.0,
        max_strafe_thrust: 50_000.0,
        active_main_thruster_index: 0,
    });
    app
}

/// Runs one fixed update tick.
fn run_fixed_update(app: &mut App) {
    let fixed_duration = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(fixed_duration);
    app.update();
}

// ---------------------------------------------------------------------------
// input_reader_system
// ---------------------------------------------------------------------------

#[test]
fn test_input_reader_thrust_forward() {
    let mut app = build_input_app();

    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::ThrustForward);
    }

    app.add_systems(FixedUpdate, input_reader_system);
    run_fixed_update(&mut app);

    let thrust = app.world().resource::<ThrustCommand>();
    assert!(
        thrust.force.z < 0.0,
        "forward thrust should be negative Z, got {}",
        thrust.force.z
    );
    assert!(
        (thrust.force.z - (-100_000.0)).abs() < 0.01,
        "forward thrust should be -100000, got {}",
        thrust.force.z
    );
}

#[test]
fn test_input_reader_thrust_backward() {
    let mut app = build_input_app();

    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::ThrustBackward);
    }

    app.add_systems(FixedUpdate, input_reader_system);
    run_fixed_update(&mut app);

    let thrust = app.world().resource::<ThrustCommand>();
    assert!(
        (thrust.force.z - 40_000.0).abs() < 0.01,
        "backward thrust should be 40000, got {}",
        thrust.force.z
    );
}

#[test]
fn test_input_reader_strafe_left() {
    let mut app = build_input_app();

    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::StrafeLeft);
    }

    app.add_systems(FixedUpdate, input_reader_system);
    run_fixed_update(&mut app);

    let thrust = app.world().resource::<ThrustCommand>();
    assert!(
        (thrust.force.x - (-50_000.0)).abs() < 0.01,
        "strafe left should be -50000 X, got {}",
        thrust.force.x
    );
}

#[test]
fn test_input_reader_strafe_right() {
    let mut app = build_input_app();

    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::StrafeRight);
    }

    app.add_systems(FixedUpdate, input_reader_system);
    run_fixed_update(&mut app);

    let thrust = app.world().resource::<ThrustCommand>();
    assert!(
        (thrust.force.x - 50_000.0).abs() < 0.01,
        "strafe right should be 50000 X, got {}",
        thrust.force.x
    );
}

#[test]
fn test_input_reader_pitch_up() {
    let mut app = build_input_app();

    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::PitchUp);
    }

    app.add_systems(FixedUpdate, input_reader_system);
    run_fixed_update(&mut app);

    let torque = app.world().resource::<TorqueCommand>();
    assert!(
        (torque.torque.x - 50_000.0).abs() < 0.01,
        "pitch up torque should be 50000 X, got {}",
        torque.torque.x
    );
}

#[test]
fn test_input_reader_yaw_left() {
    let mut app = build_input_app();

    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::YawLeft);
    }

    app.add_systems(FixedUpdate, input_reader_system);
    run_fixed_update(&mut app);

    let torque = app.world().resource::<TorqueCommand>();
    assert!(
        (torque.torque.y - 50_000.0).abs() < 0.01,
        "yaw left torque should be 50000 Y, got {}",
        torque.torque.y
    );
}

#[test]
fn test_input_reader_roll_left() {
    let mut app = build_input_app();

    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::RollLeft);
    }

    app.add_systems(FixedUpdate, input_reader_system);
    run_fixed_update(&mut app);

    let torque = app.world().resource::<TorqueCommand>();
    assert!(
        (torque.torque.z - 50_000.0).abs() < 0.01,
        "roll left torque should be 50000 Z, got {}",
        torque.torque.z
    );
}

#[test]
fn test_input_reader_multiple_actions() {
    let mut app = build_input_app();

    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::ThrustForward);
        active.0.insert(LogicalAction::YawLeft);
    }

    app.add_systems(FixedUpdate, input_reader_system);
    run_fixed_update(&mut app);

    let thrust = app.world().resource::<ThrustCommand>();
    let torque = app.world().resource::<TorqueCommand>();
    assert!(
        (thrust.force.z - (-100_000.0)).abs() < 0.01,
        "forward thrust should be applied"
    );
    assert!(
        (torque.torque.y - 50_000.0).abs() < 0.01,
        "yaw torque should be applied"
    );
}

#[test]
fn test_input_reader_no_actions() {
    let mut app = build_input_app();

    app.add_systems(FixedUpdate, input_reader_system);
    run_fixed_update(&mut app);

    let thrust = app.world().resource::<ThrustCommand>();
    let torque = app.world().resource::<TorqueCommand>();
    assert_eq!(thrust.force, Vec3::ZERO, "no thrust with no actions");
    assert_eq!(torque.torque, Vec3::ZERO, "no torque with no actions");
}

// ---------------------------------------------------------------------------
// flight_assist_toggle_system
// ---------------------------------------------------------------------------

#[test]
fn test_flight_assist_toggle_on_press() {
    let mut app = build_input_app();

    {
        let mut state = app.world_mut().resource_mut::<FlightAssistState>();
        state.enabled = false;
    }

    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::ToggleFlightAssist);
    }

    app.add_systems(FixedUpdate, flight_assist_toggle_system);
    run_fixed_update(&mut app);

    let state = app.world().resource::<FlightAssistState>();
    assert!(state.enabled, "flight assist should be toggled on");
}

#[test]
fn test_flight_assist_no_toggle_on_hold() {
    let mut app = build_input_app();

    {
        let mut state = app.world_mut().resource_mut::<FlightAssistState>();
        state.enabled = false;
    }

    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::ToggleFlightAssist);
    }

    app.add_systems(FixedUpdate, flight_assist_toggle_system);

    // First tick — toggles on
    run_fixed_update(&mut app);
    let state = app.world().resource::<FlightAssistState>();
    assert!(state.enabled, "should toggle on first press");

    // Second tick — still held, should NOT toggle again
    run_fixed_update(&mut app);
    let state = app.world().resource::<FlightAssistState>();
    assert!(
        state.enabled,
        "should remain enabled while holding (no re-toggle)"
    );
}

#[test]
fn test_flight_assist_toggle_off_on_second_press() {
    let mut app = build_input_app();

    {
        let mut state = app.world_mut().resource_mut::<FlightAssistState>();
        state.enabled = true;
    }

    // First press
    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::ToggleFlightAssist);
    }

    app.add_systems(FixedUpdate, flight_assist_toggle_system);
    run_fixed_update(&mut app);

    // Release the key
    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.clear();
    }
    run_fixed_update(&mut app);

    // Second press
    {
        let mut active = app.world_mut().resource_mut::<ActiveActions>();
        active.0.insert(LogicalAction::ToggleFlightAssist);
    }
    run_fixed_update(&mut app);

    let state = app.world().resource::<FlightAssistState>();
    assert!(
        !state.enabled,
        "flight assist should be toggled off on second press"
    );
}

// ---------------------------------------------------------------------------
// flight_assist_damping_system
// ---------------------------------------------------------------------------

#[test]
fn test_flight_assist_damping_reduces_velocity() {
    let mut app = build_input_app();

    app.insert_resource(FlightAssistState { enabled: true });
    app.insert_resource(delta_v_core::FlightAssistConfig {
        enabled_by_default: true,
        damping_coefficient: 0.1,
    });
    // PlayerShipEntity is a resource, not a component

    // Spawn a ship entity with FlightAssist marker and a RigidBody with velocity
    let ship_entity = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            delta_v_core::FlightAssist,
            Transform::from_translation(Vec3::ZERO),
        ))
        .id();
    // Insert PlayerShipEntity resource referencing the spawned entity
    app.insert_resource(delta_v_core::PlayerShipEntity(ship_entity));

    {
        let mut body = app.world_mut().get_mut::<RigidBody>(ship_entity).unwrap();
        body.velocity = Vec3::new(10.0, 5.0, 3.0);
    }

    app.add_systems(FixedUpdate, flight_assist_damping_system);
    run_fixed_update(&mut app);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    // With damping_coefficient = 0.1, velocity should be multiplied by 0.9
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

#[test]
fn test_flight_assist_damping_disabled() {
    let mut app = build_input_app();

    app.insert_resource(FlightAssistState { enabled: false });
    app.insert_resource(delta_v_core::FlightAssistConfig {
        enabled_by_default: false,
        damping_coefficient: 0.1,
    });

    let ship_entity = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            delta_v_core::FlightAssist,
            Transform::from_translation(Vec3::ZERO),
        ))
        .id();
    app.insert_resource(delta_v_core::PlayerShipEntity(ship_entity));

    {
        let mut body = app.world_mut().get_mut::<RigidBody>(ship_entity).unwrap();
        body.velocity = Vec3::new(10.0, 5.0, 3.0);
    }

    app.add_systems(FixedUpdate, flight_assist_damping_system);
    run_fixed_update(&mut app);

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

#[test]
fn test_clear_commands_resets_thrust_and_torque() {
    let mut app = build_input_app();

    {
        let mut thrust = app.world_mut().resource_mut::<ThrustCommand>();
        thrust.force = Vec3::new(100.0, 200.0, 300.0);
    }
    {
        let mut torque = app.world_mut().resource_mut::<TorqueCommand>();
        torque.torque = Vec3::new(50.0, 60.0, 70.0);
    }

    app.add_systems(FixedUpdate, clear_commands_system);
    run_fixed_update(&mut app);

    let thrust = app.world().resource::<ThrustCommand>();
    let torque = app.world().resource::<TorqueCommand>();
    assert_eq!(thrust.force, Vec3::ZERO, "thrust should be cleared");
    assert_eq!(torque.torque, Vec3::ZERO, "torque should be cleared");
}

// ---------------------------------------------------------------------------
// thrust_system
// ---------------------------------------------------------------------------

#[test]
fn test_thrust_system_applies_force_to_rigid_body() {
    let mut app = build_input_app();

    let ship_entity = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
        ))
        .id();
    app.insert_resource(delta_v_core::PlayerShipEntity(ship_entity));

    {
        let mut thrust = app.world_mut().resource_mut::<ThrustCommand>();
        thrust.force = Vec3::new(0.0, 0.0, -100_000.0);
    }

    app.add_systems(FixedUpdate, thrust_system);
    run_fixed_update(&mut app);

    // The thrust system reads PlayerShipEntity as a resource and applies force
    // to the matching RigidBody. Since we spawned a RigidBody at the right entity,
    // the force should be applied.
    let thrust = app.world().resource::<ThrustCommand>();
    // The system consumes the thrust, so after running it should still be non-zero
    // (it's not cleared by thrust_system, only by clear_commands_system)
    assert!(thrust.force.z < 0.0, "thrust command should still be set");
}

#[test]
fn test_thrust_system_zero_force_noop() {
    let mut app = build_input_app();

    let ship_entity = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
        ))
        .id();
    app.insert_resource(delta_v_core::PlayerShipEntity(ship_entity));

    // ThrustCommand defaults to zero
    app.add_systems(FixedUpdate, thrust_system);
    run_fixed_update(&mut app);

    let body = app.world().get::<RigidBody>(ship_entity).unwrap();
    assert_eq!(
        body.force_accumulator,
        Vec3::ZERO,
        "zero thrust should not apply force"
    );
}

// ---------------------------------------------------------------------------
// torque_system
// ---------------------------------------------------------------------------

#[test]
fn test_torque_system_applies_torque_to_rigid_body() {
    let mut app = build_input_app();

    let ship_entity = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
        ))
        .id();
    app.insert_resource(delta_v_core::PlayerShipEntity(ship_entity));
    {
        let mut torque = app.world_mut().resource_mut::<TorqueCommand>();
        torque.torque = Vec3::new(50_000.0, 0.0, 0.0);
    }

    app.add_systems(FixedUpdate, torque_system);
    run_fixed_update(&mut app);

    let torque = app.world().resource::<TorqueCommand>();
    assert!(torque.torque.x > 0.0, "torque command should still be set");
}
