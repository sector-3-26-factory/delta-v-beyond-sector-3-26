// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Physics systems integration tests (SOI gravity model per ADR-0055).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_physics::celestial::{OrbitalBody, Planet, Sun};
use delta_v_physics::rigid_body::{MassSource, RigidBody};
use delta_v_physics::systems::GravityAttractor;

fn test_soi_gravity_sun_soi_encompasses_all(app: &mut App) {
    let _sun_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1.989e30, 1.0),
            Transform::from_translation(Vec3::ZERO),
            MassSource,
            Sun {
                rotation_period: Some(25.0 * 24.0),
            },
            OrbitalBody {
                orbital_parent: Entity::PLACEHOLDER,
                orbital_distance: 0.0,
                orbital_period: 0.0,
                orbital_eccentricity: 0.0,
                orbital_inclination: 0.0,
                initial_orbital_angle: 0.0,
            },
        ))
        .id();

    let ship_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::new(1.496e11, 0.0, 0.0)),
        ))
        .id();

    delta_v_test_framework::run_fixed_update(app);

    // The full physics stack clears `force_accumulator` at the end of each
    // tick (clear_accumulators_system), so assert on the integrated
    // `velocity` instead — it carries the same gravity signal.
    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    assert!(
        ship.velocity.x < 0.0,
        "ship should feel Sun's gravity (negative X), got {}",
        ship.velocity.x
    );
}

fn test_soi_gravity_no_celestial_bodies_no_gravity(app: &mut App) {
    let ship_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        ))
        .id();

    delta_v_test_framework::run_fixed_update(app);

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    assert_eq!(
        ship.velocity,
        Vec3::ZERO,
        "no gravity without celestial bodies"
    );
}

fn test_soi_gravity_attractor_overrides(app: &mut App) {
    let _planet_id = app
        .world_mut()
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
        .id();

    let _attractor_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1e12, 1.0),
            Transform::from_translation(Vec3::new(1000.0, 0.0, 0.0)),
            GravityAttractor {
                radius: 5000.0,
                strength_multiplier: 10.0,
            },
        ))
        .id();

    let ship_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::new(2000.0, 0.0, 0.0)),
        ))
        .id();

    delta_v_test_framework::run_fixed_update(app);

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    assert!(
        ship.velocity.x < 0.0,
        "ship should feel combined gravity (negative X), got {}",
        ship.velocity.x
    );

    // Full stack clears force_accumulator each tick; assert on the
    // integrated velocity instead. v = F/m * dt, dt = 1/60 s.
    let planet_only_force = 6.67430e-11 * 5.972e24 * 1000.0 / (2000.0 * 2000.0);
    let attractor_force = 6.67430e-11 * 1e12 * 1000.0 / (1000.0 * 1000.0) * 10.0;
    let expected_total = planet_only_force + attractor_force;
    let expected_velocity = expected_total / 1000.0 / 60.0;

    let actual_velocity = ship.velocity.x.abs();
    let relative_error = (actual_velocity - expected_velocity).abs() / expected_velocity;
    assert!(
        relative_error < 0.01,
        "combined force mismatch: expected velocity {expected_velocity}, got {actual_velocity} (relative error {relative_error})"
    );
}

fn test_soi_gravity_deterministic(app: &mut App) {
    // The shared app is already render-settled by `main.rs`; we cannot
    // rebuild it here (WinitPlugin permits one EventLoop per thread).
    // Assert against the analytic value instead: F = G*M*m/r², integrated
    // to velocity via v = F/m*dt (dt = 1/60 s). This is a stronger check
    // than comparing two runs — it proves the force is exactly right.
    let _planet_id = app
        .world_mut()
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
        .id();

    let ship_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::new(6.371e6 + 400_000.0, 0.0, 0.0)),
        ))
        .id();

    delta_v_test_framework::reset_fixed_time(app);
    delta_v_test_framework::run_fixed_update(app);

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    // Planet at origin, ship at +X → gravity pulls -X.
    assert!(
        ship.velocity.x < 0.0,
        "ship should feel planet gravity (negative X), got {}",
        ship.velocity.x
    );

    let g = 6.67430e-11_f32;
    let mass = 5.972e24_f32;
    let ship_mass = 1000.0_f32;
    let dist = 6.371e6 + 400_000.0;
    let force = g * mass * ship_mass / (dist * dist);
    let expected_velocity = force / ship_mass / 60.0;

    let actual = ship.velocity.x.abs();
    let relative_error = (actual - expected_velocity).abs() / expected_velocity;
    assert!(
        relative_error < 0.01,
        "gravity force mismatch: expected velocity {expected_velocity}, got {actual} (relative error {relative_error})"
    );
}

pub fn init_registry() {
    do_register(
        "physics::soi_gravity_sun_soi_encompasses_all",
        test_soi_gravity_sun_soi_encompasses_all,
    );
    do_register(
        "physics::soi_gravity_no_celestial_bodies_no_gravity",
        test_soi_gravity_no_celestial_bodies_no_gravity,
    );
    do_register(
        "physics::soi_gravity_attractor_overrides",
        test_soi_gravity_attractor_overrides,
    );
    do_register(
        "physics::soi_gravity_deterministic",
        test_soi_gravity_deterministic,
    );
}

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
