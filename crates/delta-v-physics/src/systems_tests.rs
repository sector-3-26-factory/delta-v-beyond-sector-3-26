// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for physics integration systems.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use bevy::prelude::*;
use bevy::time::TimePlugin;

use crate::constants::{GRAVITATIONAL_CONSTANT, GRAVITY_CUTOFF_RADIUS_M};
use crate::rigid_body::{MassSource, RigidBody};
use crate::systems::{gravity_system, PhysicsSet};

/// Builds a minimal Bevy app with the fixed timestep and physics systems.
fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.configure_sets(
        FixedUpdate,
        (
            PhysicsSet::AccumulateForces,
            PhysicsSet::IntegrateVelocity,
            PhysicsSet::IntegratePosition,
            PhysicsSet::ClearAccumulators,
        )
            .chain(),
    );
    app.add_systems(
        FixedUpdate,
        gravity_system.in_set(PhysicsSet::AccumulateForces),
    );
    app
}

/// Runs one fixed update tick.
///
/// In Bevy, `FixedUpdate` runs when `Time<Real>` has advanced past the
/// fixed timestep threshold. We advance `Time<Real>` by the fixed timestep
/// duration and then call `app.update()`, which triggers `FixedUpdate`.
fn run_fixed_update(app: &mut App) {
    let fixed_duration = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(fixed_duration);
    app.update();
}

#[test]
fn test_gravity_force_direction() {
    let mut app = build_test_app();

    app.world_mut().spawn((
        RigidBody::new(1000.0, 1.0),
        Transform::from_translation(Vec3::ZERO),
        MassSource,
    ));

    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        ))
        .id();

    run_fixed_update(&mut app);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    assert!(
        body.force_accumulator.x < 0.0,
        "gravity should pull toward source (negative X), got {}",
        body.force_accumulator.x
    );
    assert!(
        body.force_accumulator.y.abs() < f32::EPSILON,
        "no Y force expected, got {}",
        body.force_accumulator.y
    );
    assert!(
        body.force_accumulator.z.abs() < f32::EPSILON,
        "no Z force expected, got {}",
        body.force_accumulator.z
    );
}

#[test]
fn test_gravity_force_magnitude() {
    let mut app = build_test_app();

    let source_mass = 1000.0_f32;
    let body_mass = 10.0_f32;
    let distance = 10.0_f32;

    app.world_mut().spawn((
        RigidBody::new(source_mass, 1.0),
        Transform::from_translation(Vec3::ZERO),
        MassSource,
    ));

    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(body_mass, 1.0),
            Transform::from_translation(Vec3::new(distance, 0.0, 0.0)),
        ))
        .id();

    run_fixed_update(&mut app);

    let expected_force = GRAVITATIONAL_CONSTANT * source_mass * body_mass / (distance * distance);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    let actual_force = body.force_accumulator.x.abs();

    let relative_error = (actual_force - expected_force).abs() / expected_force;
    assert!(
        relative_error < 0.001,
        "force magnitude mismatch: expected {expected_force}, got {actual_force} (relative error {relative_error})"
    );
}

#[test]
fn test_gravity_beyond_cutoff() {
    let mut app = build_test_app();

    let cutoff = GRAVITY_CUTOFF_RADIUS_M;

    app.world_mut().spawn((
        RigidBody::new(1e15, 1.0),
        Transform::from_translation(Vec3::ZERO),
        MassSource,
    ));

    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(cutoff + 1.0, 0.0, 0.0)),
        ))
        .id();

    run_fixed_update(&mut app);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    assert!(
        body.force_accumulator.length() < f32::EPSILON,
        "no gravity expected beyond cutoff, got {:?}",
        body.force_accumulator
    );
}

#[test]
fn test_gravity_no_sources() {
    let mut app = build_test_app();

    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        ))
        .id();

    run_fixed_update(&mut app);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    assert_eq!(
        body.force_accumulator,
        Vec3::ZERO,
        "no gravity without mass sources"
    );
}

#[test]
fn test_gravity_multiple_sources_deterministic() {
    let run_scenario = || -> f32 {
        let mut app = build_test_app();

        app.world_mut().spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::new(-5.0, 0.0, 0.0)),
            MassSource,
        ));
        app.world_mut().spawn((
            RigidBody::new(2000.0, 1.0),
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
            MassSource,
        ));

        let body_id = app
            .world_mut()
            .spawn((
                RigidBody::new(10.0, 1.0),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        run_fixed_update(&mut app);

        let body = app.world().get::<RigidBody>(body_id).unwrap();
        body.force_accumulator.x
    };

    let result1 = run_scenario();
    let result2 = run_scenario();

    assert!(
        (result1 - result2).abs() < f32::EPSILON,
        "gravity must be deterministic: run 1 = {result1}, run 2 = {result2}"
    );
}
