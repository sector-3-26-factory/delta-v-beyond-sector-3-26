// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Multi-tick physics integration tests and floating origin tests.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp)]

use bevy::prelude::*;
use bevy::time::TimePlugin;

use crate::constants::{GRAVITATIONAL_CONSTANT, GRAVITY_CUTOFF_RADIUS_M};
use crate::rigid_body::{MassSource, RigidBody};
use crate::systems::{
    PhysicsSet, clear_accumulators_system, gravity_system, integrate_angular_velocity_system,
    integrate_position_system, integrate_velocity_system,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds a minimal Bevy app with the full physics pipeline registered.
fn build_physics_app() -> App {
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
        (
            gravity_system.in_set(PhysicsSet::AccumulateForces),
            integrate_velocity_system.in_set(PhysicsSet::IntegrateVelocity),
            integrate_angular_velocity_system.in_set(PhysicsSet::IntegrateVelocity),
            integrate_position_system.in_set(PhysicsSet::IntegratePosition),
            clear_accumulators_system.in_set(PhysicsSet::ClearAccumulators),
        ),
    );
    app
}

/// Builds a minimal Bevy app with only gravity (no `clear_accumulators`),
/// useful for checking force accumulator values after a tick.
fn build_gravity_only_app() -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.configure_sets(FixedUpdate, (PhysicsSet::AccumulateForces,));
    app.add_systems(
        FixedUpdate,
        gravity_system.in_set(PhysicsSet::AccumulateForces),
    );
    app
}

fn run_fixed_update(app: &mut App) {
    let fixed_duration = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(fixed_duration);
    app.update();
}

fn run_fixed_updates(app: &mut App, n: usize) {
    for _ in 0..n {
        run_fixed_update(app);
    }
}

// ---------------------------------------------------------------------------
// P1: Physics integration multi-tick tests
// ---------------------------------------------------------------------------

#[test]
fn test_velocity_accumulates_over_multiple_ticks() {
    let mut app = build_physics_app();

    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        ))
        .id();

    app.world_mut().spawn((
        RigidBody::new(1000.0, 1.0),
        Transform::from_translation(Vec3::ZERO),
        MassSource,
    ));

    run_fixed_updates(&mut app, 60);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    assert!(
        body.velocity.x < 0.0,
        "body should have negative X velocity (pulled toward source), got {}",
        body.velocity.x
    );
}

#[test]
fn test_position_changes_with_velocity() {
    let mut app = build_physics_app();

    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
        ))
        .id();

    {
        let mut body = app.world_mut().get_mut::<RigidBody>(body_id).unwrap();
        body.velocity = Vec3::new(1.0, 0.0, 0.0);
    }

    run_fixed_update(&mut app);

    let transform = app.world().get::<Transform>(body_id).unwrap();
    let expected_x = 1.0 / 60.0;
    assert!(
        (transform.translation.x - expected_x).abs() < 0.001,
        "position should integrate velocity, got x={}",
        transform.translation.x
    );
}

#[test]
fn test_clear_accumulators_resets_forces_between_ticks() {
    let mut app = build_physics_app();

    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        ))
        .id();

    app.world_mut().spawn((
        RigidBody::new(1000.0, 1.0),
        Transform::from_translation(Vec3::ZERO),
        MassSource,
    ));

    run_fixed_update(&mut app);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    assert_eq!(
        body.force_accumulator,
        Vec3::ZERO,
        "force accumulator should be cleared after tick"
    );
}

#[test]
fn test_gravity_self_interaction_skip() {
    let mut app = build_physics_app();

    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            MassSource,
        ))
        .id();

    run_fixed_update(&mut app);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    assert_eq!(
        body.force_accumulator,
        Vec3::ZERO,
        "MassSource should not exert gravity on itself"
    );
}

// ---------------------------------------------------------------------------
// P2: Gravity at multiple distances
// ---------------------------------------------------------------------------

#[test]
fn test_gravity_at_near_distance() {
    let mut app = build_gravity_only_app();

    let source_mass = 1e10_f32;
    let body_mass = 10.0_f32;
    let distance = 100.0_f32;

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
        "near-distance force mismatch: expected {expected_force}, got {actual_force}"
    );
}

#[test]
fn test_gravity_at_mid_distance() {
    let mut app = build_gravity_only_app();

    let source_mass = 1e10_f32;
    let body_mass = 10.0_f32;
    let distance = 10_000.0_f32;

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
        "mid-distance force mismatch: expected {expected_force}, got {actual_force}"
    );
}

#[test]
fn test_gravity_at_cutoff_boundary() {
    let mut app = build_gravity_only_app();

    let cutoff = GRAVITY_CUTOFF_RADIUS_M;
    let source_mass = 1e15_f32;
    let body_mass = 10.0_f32;

    app.world_mut().spawn((
        RigidBody::new(source_mass, 1.0),
        Transform::from_translation(Vec3::ZERO),
        MassSource,
    ));

    let body_inside = app
        .world_mut()
        .spawn((
            RigidBody::new(body_mass, 1.0),
            Transform::from_translation(Vec3::new(cutoff - 1.0, 0.0, 0.0)),
        ))
        .id();

    run_fixed_update(&mut app);

    let body = app.world().get::<RigidBody>(body_inside).unwrap();
    assert!(
        body.force_accumulator.length() > f32::EPSILON,
        "gravity should be applied just inside cutoff"
    );
}

// ---------------------------------------------------------------------------
// P1: Floating origin recentering tests
// ---------------------------------------------------------------------------

#[test]
fn test_floating_origin_no_recenter_below_threshold() {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, FloatingOriginEligible};

    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.insert_resource(FloatingOrigin::new(Vec3::ZERO));
    app.insert_resource(FloatingOriginConfig {
        recenter_threshold_m: 5_000.0,
    });

    app.add_systems(
        FixedUpdate,
        crate::floating_origin_systems::check_and_recenter_origin_system,
    );

    let entity_id = app
        .world_mut()
        .spawn((
            Transform::from_translation(Vec3::new(1_000.0, 0.0, 0.0)),
            FloatingOriginEligible,
        ))
        .id();

    let original_pos = app.world().get::<Transform>(entity_id).unwrap().translation;

    run_fixed_update(&mut app);

    let transform = app.world().get::<Transform>(entity_id).unwrap();
    assert_eq!(
        transform.translation, original_pos,
        "entity below threshold should not be recentered"
    );
}

#[test]
fn test_mark_new_entities_system() {
    use delta_v_core::FloatingOriginEligible;

    let mut app = App::new();
    app.add_plugins(TimePlugin);

    app.add_systems(
        Update,
        crate::floating_origin_systems::mark_new_entities_system,
    );

    let entity_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)))
        .id();

    app.update();

    let eligible = app.world().get::<FloatingOriginEligible>(entity_id);
    assert!(
        eligible.is_some(),
        "new entity with Transform should be marked as FloatingOriginEligible"
    );
}

// ---------------------------------------------------------------------------
// P2: RigidBody panic tests
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "RigidBodyData mass must be positive and finite")]
fn test_rigid_body_zero_mass_panics() {
    RigidBody::new(0.0, 1.0);
}

#[test]
#[should_panic(expected = "RigidBodyData mass must be positive and finite")]
fn test_rigid_body_negative_mass_panics() {
    RigidBody::new(-10.0, 1.0);
}

#[test]
#[should_panic(expected = "RigidBodyData mass must be positive and finite")]
fn test_rigid_body_nan_mass_panics() {
    RigidBody::new(f32::NAN, 1.0);
}

#[test]
#[should_panic(expected = "RigidBodyData mass must be positive and finite")]
fn test_rigid_body_infinite_mass_panics() {
    RigidBody::new(f32::INFINITY, 1.0);
}

#[test]
#[should_panic(expected = "RigidBodyData mass must be positive and finite")]
fn test_rigid_body_neg_infinity_mass_panics() {
    RigidBody::new(f32::NEG_INFINITY, 1.0);
}
