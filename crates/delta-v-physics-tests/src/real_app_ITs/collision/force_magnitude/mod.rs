// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Force magnitude test — verifies gravitational force matches F = G·M·m/r².
//
//! Each test function is registered in the global registry via
//! [`init_registry`] and called by the binary's `run_test_function()`
//! when `--test-function <name>` is passed.
//
//! Approach: Option A (Programmatic Setup) — entities are spawned in code;
//! no world fixture needed because this test does not exercise the world
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
use delta_v_physics::constants::GRAVITATIONAL_CONSTANT;
use delta_v_physics::rigid_body::{MassSource, RigidBody};

/// Register all force-magnitude test functions in the global registry.
pub fn init_registry() {
    do_register(
        "physics::soi_gravity_force_magnitude",
        test_soi_gravity_force_magnitude,
    );
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}

fn test_soi_gravity_force_magnitude(app: &mut App) {
    let planet_mass = 5.972e24_f32;
    let ship_mass = 1000.0_f32;
    let distance = 6.371e6 + 400_000.0;

    let _planet_id = app
        .world_mut()
        .spawn((
            RigidBody::new(planet_mass, 1.0),
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
        .id();

    let ship_id = app
        .world_mut()
        .spawn((
            RigidBody::new(ship_mass, 1.0),
            Transform::from_translation(Vec3::new(distance, 0.0, 0.0)),
        ))
        .id();

    delta_v_test_framework::run_fixed_update(app);

    let distance_sq = distance * distance;
    let expected_velocity = GRAVITATIONAL_CONSTANT * planet_mass / distance_sq / 60.0;

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    let actual_velocity = ship.velocity.x.abs();

    let relative_error = (actual_velocity - expected_velocity).abs() / expected_velocity;
    assert!(
        relative_error < 0.001,
        "velocity magnitude mismatch: expected {expected_velocity}, got {actual_velocity} (relative error {relative_error})"
    );
}
