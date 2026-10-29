// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Force magnitude test — verifies gravitational force matches F = G·M·m/r².
//!
//! Registered as a `[[test]]` target with `harness = false` in Cargo.toml so
//! that `cargo test --test force_magnitude_tests` runs `main()` on the
//! **main thread**, which is required by Bevy's `WinitPlugin`.
//!
//!   xvfb-run -a cargo test --test `force_magnitude_tests` -p delta-v-physics
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
use delta_v_physics::celestial::{OrbitalBody, Planet};
use delta_v_physics::constants::GRAVITATIONAL_CONSTANT;
use delta_v_physics::rigid_body::{MassSource, RigidBody};
use delta_v_test_framework::{check, make_app, run_fixed_update, run_harness};

/// Build the test `App` via `build_app` (Single Source of Truth per ADR-0060).
/// Uses the empty `test` world fixture so no celestial bodies interfere.
fn build_test_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-physics/tests/fixtures/assets".to_string(),
        world_name: Some("test".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
}

/// Test: gravitational force magnitude matches F = G·M·m/r².
///
/// Spawns an Earth-mass planet at the origin and a 1000kg ship at 400km
/// altitude. After one fixed tick, the ship's velocity should match the
/// integrated force within 0.1% relative error.
///
/// Approach: Option A (Programmatic Setup) — entities are spawned in code;
/// no world fixture needed because this test does not exercise the world
/// loading pipeline (templates, spawning, UI).
fn test_soi_gravity_force_magnitude(app: &mut App) {
    let planet_mass = 5.972e24_f32; // Earth mass
    let ship_mass = 1000.0_f32;
    let distance = 6.371e6 + 400_000.0; // Earth radius + 400km altitude

    // Spawn a planet (celestial body with MassSource and OrbitalBody)
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

    // Spawn a ship (dynamic body - no MassSource, no celestial components)
    let ship_id = app
        .world_mut()
        .spawn((
            RigidBody::new(ship_mass, 1.0),
            Transform::from_translation(Vec3::new(distance, 0.0, 0.0)),
        ))
        .id();

    run_fixed_update(app);

    // Expected force: F = G·M·m/r²
    // Expected acceleration: a = F/m = G·M/r²
    // Expected velocity after one tick: v = a * dt = (G·M/r²) * dt
    let distance_sq = distance * distance;
    let expected_velocity = GRAVITATIONAL_CONSTANT * planet_mass / distance_sq / 60.0;

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    let actual_velocity = ship.velocity.x.abs();

    let relative_error = (actual_velocity - expected_velocity).abs() / expected_velocity;
    check(
        relative_error < 0.001,
        &format!(
            "velocity magnitude mismatch: expected {expected_velocity}, got {actual_velocity} (relative error {relative_error})"
        ),
    );
}

/// Entry point — runs on the main thread so `WinitPlugin` can initialize.
fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(
        &mut app,
        &[(
            "soi_gravity_force_magnitude",
            test_soi_gravity_force_magnitude,
        )],
    );
}
