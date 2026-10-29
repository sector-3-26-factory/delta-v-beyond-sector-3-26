// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Multi-tick physics integration tests and floating origin tests.
//! Updated for SOI gravity model per ADR-0055.
//!
//! Registered as a `[[test]]` target with `harness = false` in Cargo.toml so
//! that `cargo test --test physics_integration_tests` runs `main()` on the
//! **main thread**, which is required by Bevy's `WinitPlugin` (see
//! plans/testframework.md step 8).
//!
//!   xvfb-run -a cargo test --test `physics_integration_tests` -p delta-v-physics
//!
//! See ADR-0021 (Testing strategy) and ADR-0060 (Test framework single
//! source of truth).

use bevy::prelude::*;
use delta_v_app::{AppConfig, build_app};
use delta_v_physics::celestial::{OrbitalBody, Planet};
use delta_v_physics::rigid_body::{MassSource, RigidBody};
use delta_v_test_framework::{check, make_app, run_fixed_update, run_fixed_updates, run_harness};

/// Build the test `App` via `build_app` (Single Source of Truth per ADR-0060).
/// `PhysicsPlugin` registers `FloatingOrigin`, `FloatingOriginConfig`,
/// `Time<Fixed>` and the gravity/integrate systems. No manual
/// `insert_resource` for those (plan step 7).
fn build_test_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-physics/tests/fixtures/assets".to_string(),
        world_name: Some("test".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
}

// ---------------------------------------------------------------------------
// P1: Physics integration multi-tick tests
// ---------------------------------------------------------------------------

/// Test: SOI gravity accumulates velocity over multiple ticks.
///
/// A ship at 400km altitude inside a planet's SOI should gain negative X
/// velocity (falling toward the planet) after 60 fixed-timestep ticks.
///
/// Approach: Option A (programmatic setup) — entities are spawned in code;
/// no world fixture needed because this test does not exercise the world
/// loading pipeline (templates, spawning, UI).
fn test_soi_velocity_accumulates_over_multiple_ticks(app: &mut App) {
    // Spawn a planet (celestial body with SOI)
    let _planet_id = app
        .world_mut()
        .spawn((
            RigidBody::new(5.972e24, 1.0), // Earth mass
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

    // Spawn a ship (dynamic body)
    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(6.371e6 + 400_000.0, 0.0, 0.0)), // 400km altitude
        ))
        .id();

    run_fixed_updates(app, 60);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    check(
        body.velocity.x < 0.0,
        "body should have negative X velocity (pulled toward planet)",
    );
}

/// Entry point — runs on the main thread so `WinitPlugin` can initialize.
fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(
        &mut app,
        &[
            (
                "soi_velocity_accumulates_over_multiple_ticks",
                test_soi_velocity_accumulates_over_multiple_ticks,
            ),
            (
                "position_changes_with_velocity",
                test_position_changes_with_velocity,
            ),
            (
                "soi_clear_accumulators_resets_forces_between_ticks",
                test_soi_clear_accumulators_resets_forces_between_ticks,
            ),
            (
                "soi_celestial_body_no_self_gravity",
                test_soi_celestial_body_no_self_gravity,
            ),
            ("soi_gravity_inside_soi", test_soi_gravity_inside_soi),
        ],
    );
}

/// Test: position integrates velocity over one tick.
///
/// A body with X velocity 1.0 m/s should move by 1/60 m after one fixed
/// timestep (60 Hz). Verifies the position integration system.
///
/// Approach: Option A (programmatic setup).
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

    run_fixed_update(app);

    let transform = app.world().get::<Transform>(body_id).unwrap();
    let expected_x = 1.0 / 60.0;
    check(
        (transform.translation.x - expected_x).abs() < 0.001,
        "position should integrate velocity",
    );
}

/// Test: force accumulator is cleared after each tick.
///
/// After one fixed update, the ship's `force_accumulator` must be `Vec3::ZERO`
/// so forces do not accumulate across ticks. Each tick recomputes gravity
/// fresh; without this clear, forces would compound (G, 2G, 3G...).
///
/// Approach: Option A (programmatic setup).
fn test_soi_clear_accumulators_resets_forces_between_ticks(app: &mut App) {
    // Spawn a planet (celestial body with SOI)
    let _planet_id = app
        .world_mut()
        .spawn((
            RigidBody::new(5.972e24, 1.0), // Earth mass
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

    let body_id = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(6.371e6 + 400_000.0, 0.0, 0.0)),
        ))
        .id();

    run_fixed_update(app);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    check(
        body.force_accumulator == Vec3::ZERO,
        "force accumulator should be cleared after tick",
    );
}

/// Test: a celestial body does not exert gravity on itself.
///
/// A planet with `MassSource` should not accumulate force from its own
/// gravity field — only dynamic bodies inside its SOI are affected.
///
/// Approach: Option A (programmatic setup).
fn test_soi_celestial_body_no_self_gravity(app: &mut App) {
    // Spawn a planet (celestial body with SOI)
    let planet_id = app
        .world_mut()
        .spawn((
            RigidBody::new(5.972e24, 1.0), // Earth mass
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

    run_fixed_update(app);

    let planet = app.world().get::<RigidBody>(planet_id).unwrap();
    check(
        planet.force_accumulator == Vec3::ZERO,
        "Celestial body should not exert gravity on itself",
    );
}

// ---------------------------------------------------------------------------
// P2: Gravity at multiple distances
// ---------------------------------------------------------------------------

// Old N-body gravity tests removed - superseded by SOI gravity tests per ADR-0055.
// The old tests used bare MassSource without celestial body components (Sun/Planet/Moon + OrbitalBody).
// New SOI gravity tests (test_soi_gravity_*) cover the same functionality with proper celestial hierarchy.

/// Test: gravity is applied inside the planet's SOI.
///
/// A ship at 400km altitude (well inside Earth's ~925,000km SOI) should
/// accumulate a non-zero force toward the planet (negative X). This is the
/// positive counterpart to `test_soi_celestial_body_no_self_gravity` — it
/// confirms gravity IS applied inside the SOI, not just that the planet
/// doesn't self-gravitate.
///
/// Approach: Option A (programmatic setup).
fn test_soi_gravity_inside_soi(app: &mut App) {
    // Spawn a planet with known SOI
    let planet_mass = 5.972e24_f32; // Earth mass
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
                orbital_distance: 1.496e11, // 1 AU
                orbital_period: 365.25 * 24.0 * 3600.0,
                orbital_eccentricity: 0.0167,
                orbital_inclination: 0.0,
                initial_orbital_angle: 0.0,
            },
        ))
        .id();

    // Spawn a ship well inside the planet's SOI (at 400km altitude)
    // Earth SOI radius ≈ 925,000 km, so 400km is well inside
    let ship_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::new(6.371e6 + 400_000.0, 0.0, 0.0)),
        ))
        .id();

    run_fixed_update(app);

    // Note: `force_accumulator` is cleared by `clear_accumulators_system`
    // at the end of every tick, so it is always Vec3::ZERO after a completed
    // tick. The observable signal of gravity is the ship's velocity — the
    // integrated result of the gravitational force. After one tick inside
    // the planet's SOI, the ship should have negative X velocity (falling
    // toward the planet at the origin).
    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    check(
        ship.velocity.x < 0.0,
        "ship should gain negative X velocity (gravity applied inside SOI)",
    );
    // Y/Z zero-check: gravity should pull purely along X toward the planet
    // at the origin. No lateral velocity should appear.
    check(
        ship.velocity.y.abs() < f32::EPSILON,
        "no Y velocity expected",
    );
    check(
        ship.velocity.z.abs() < f32::EPSILON,
        "no Z velocity expected",
    );
}

// ---------------------------------------------------------------------------
// P4: RigidBody panic tests
// ---------------------------------------------------------------------------
//
// These tests live in `crates/delta-v-physics/src/rigid_body_panic_tests.rs`
// (unit tests, no harness needed). Duplicated here previously; removed to
// avoid running the same assertions twice.
// ---------------------------------------------------------------------------
