// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Multi-tick physics integration tests and floating origin tests.
//!
//! Updated for SOI gravity model per ADR-0055.
//
//! Each test function is registered in the global registry via
//! [`init_registry`] and called by the binary's `run_test_function()`
//! when `--test-function <name>` is passed.
//
//! Approach: Option A (programmatic setup) — entities are spawned in code;
//! no world fixture needed because these tests do not exercise the world
//! loading pipeline (templates, spawning, UI).
//
//! # Panics
//!
//! Each test runs in its own process (one `--test-function` call per
//! process), so a failed assertion panics and the binary exits 1. This
//! is the intended failure mode: the test driver checks the exit code.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_physics::celestial::{OrbitalBody, Planet};
use delta_v_physics::rigid_body::{MassSource, RigidBody};

/// Register all physics integration test functions in the global registry.
pub fn init_registry() {
    do_register(
        "physics::soi_velocity_accumulates_over_multiple_ticks",
        test_soi_velocity_accumulates_over_multiple_ticks,
    );
    do_register(
        "physics::position_changes_with_velocity",
        test_position_changes_with_velocity,
    );
    do_register(
        "physics::soi_clear_accumulators_resets_forces_between_ticks",
        test_soi_clear_accumulators_resets_forces_between_ticks,
    );
    do_register(
        "physics::soi_celestial_body_no_self_gravity",
        test_soi_celestial_body_no_self_gravity,
    );
    do_register(
        "physics::soi_gravity_inside_soi",
        test_soi_gravity_inside_soi,
    );
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}

fn spawn_earth(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            RigidBody::new(5.972e24, 1.0),
            Transform::from_translation(Vec3::ZERO),
            MassSource,
            Planet {
                rotation_period: Some(24.0 * 3600.0),
                axial_tilt: 0.0,
                animations_enabled: false,
            },
            OrbitalBody {
                orbital_parent: Entity::PLACEHOLDER,
                orbital_distance: 1.496e11,
                orbital_period: 365.25 * 24.0 * 3600.0,
                orbital_eccentricity: 0.0167,
                orbital_inclination: 0.0,
                initial_orbital_angle: 0.0,
            },
        ))
        .id()
}

fn test_soi_velocity_accumulates_over_multiple_ticks(app: &mut App) {
    let _planet_id = spawn_earth(app);

    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(6.371e6 + 400_000.0, 0.0, 0.0)),
        ))
        .id();

    delta_v_test_framework::run_fixed_updates(app, 60);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    assert!(
        body.velocity.x < 0.0,
        "body should have negative X velocity (pulled toward planet)"
    );
}

fn test_position_changes_with_velocity(app: &mut App) {
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

    delta_v_test_framework::run_fixed_update(app);

    let transform = app.world().get::<Transform>(body_id).unwrap();
    let expected_x = 1.0 / 60.0;
    assert!(
        (transform.translation.x - expected_x).abs() < 0.001,
        "position should integrate velocity"
    );
}

fn test_soi_clear_accumulators_resets_forces_between_ticks(app: &mut App) {
    let _planet_id = spawn_earth(app);

    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(6.371e6 + 400_000.0, 0.0, 0.0)),
        ))
        .id();

    delta_v_test_framework::run_fixed_update(app);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    assert!(
        body.force_accumulator == Vec3::ZERO,
        "force accumulator should be cleared after tick"
    );
}

fn test_soi_celestial_body_no_self_gravity(app: &mut App) {
    let planet_id = spawn_earth(app);

    delta_v_test_framework::run_fixed_update(app);

    let planet = app.world().get::<RigidBody>(planet_id).unwrap();
    assert!(
        planet.force_accumulator == Vec3::ZERO,
        "Celestial body should not exert gravity on itself"
    );
}

fn test_soi_gravity_inside_soi(app: &mut App) {
    let _planet_id = spawn_earth(app);

    let ship_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::new(6.371e6 + 400_000.0, 0.0, 0.0)),
        ))
        .id();

    delta_v_test_framework::run_fixed_update(app);

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    assert!(
        ship.velocity.x < 0.0,
        "ship should gain negative X velocity (gravity applied inside SOI)"
    );
    assert!(
        ship.velocity.y.abs() < f32::EPSILON,
        "no Y velocity expected"
    );
    assert!(
        ship.velocity.z.abs() < f32::EPSILON,
        "no Z velocity expected"
    );
}
