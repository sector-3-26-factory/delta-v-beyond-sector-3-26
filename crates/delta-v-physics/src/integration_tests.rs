// AGENTS: before modifying this file, read AGENTS.md at the repository root.

#![cfg(test)]

//! Multi-tick physics integration tests and floating origin tests.
//! Updated for SOI gravity model per ADR-0055.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp)]

use bevy::prelude::*;
use bevy::time::TimePlugin;

use crate::celestial::{OrbitalBody, Planet};
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
/// Uses SOI gravity model per ADR-0055.
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
fn test_soi_velocity_accumulates_over_multiple_ticks() {
    let mut app = build_physics_app();

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

    run_fixed_updates(&mut app, 60);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    assert!(
        body.velocity.x < 0.0,
        "body should have negative X velocity (pulled toward planet), got {}",
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
fn test_soi_clear_accumulators_resets_forces_between_ticks() {
    let mut app = build_physics_app();

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

    run_fixed_update(&mut app);

    let body = app.world().get::<RigidBody>(body_id).unwrap();
    assert_eq!(
        body.force_accumulator,
        Vec3::ZERO,
        "force accumulator should be cleared after tick"
    );
}

#[test]
fn test_soi_celestial_body_no_self_gravity() {
    let mut app = build_physics_app();

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

    run_fixed_update(&mut app);

    let planet = app.world().get::<RigidBody>(planet_id).unwrap();
    assert_eq!(
        planet.force_accumulator,
        Vec3::ZERO,
        "Celestial body should not exert gravity on itself"
    );
}

// ---------------------------------------------------------------------------
// P2: Gravity at multiple distances
// ---------------------------------------------------------------------------

// Old N-body gravity tests removed - superseded by SOI gravity tests per ADR-0055.
// The old tests used bare MassSource without celestial body components (Sun/Planet/Moon + OrbitalBody).
// New SOI gravity tests (test_soi_gravity_*) cover the same functionality with proper celestial hierarchy.

#[test]
fn test_soi_gravity_inside_soi() {
    let mut app = build_gravity_only_app();

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

    run_fixed_update(&mut app);

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    assert!(
        ship.force_accumulator.length() > f32::EPSILON,
        "gravity should be applied inside SOI"
    );
    // Force should be toward planet (negative X)
    assert!(ship.force_accumulator.x < 0.0);
}

// ---------------------------------------------------------------------------
// P3: Floating origin recentering tests
// ---------------------------------------------------------------------------

#[test]
fn test_floating_origin_no_recenter_below_threshold() {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

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

    // Spawn player ship at position below threshold
    let player_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(1_000.0, 0.0, 0.0)))
        .id();
    app.insert_resource(PlayerShipEntity(player_id));

    // Spawn another entity to verify it doesn't get recentered
    let other_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)))
        .id();

    let original_player_pos = app.world().get::<Transform>(player_id).unwrap().translation;
    let original_other_pos = app.world().get::<Transform>(other_id).unwrap().translation;

    run_fixed_update(&mut app);

    let player_transform = app.world().get::<Transform>(player_id).unwrap();
    let other_transform = app.world().get::<Transform>(other_id).unwrap();

    // Player should still be at the same position (no recentering below threshold)
    assert_eq!(
        player_transform.translation, original_player_pos,
        "player below threshold should not be recentered"
    );
    // Other entity should also not be recentered
    assert_eq!(
        other_transform.translation, original_other_pos,
        "other entity should not be recentered when player is below threshold"
    );
}

#[test]
fn test_floating_origin_recenter_above_threshold() {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

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

    // Spawn player ship at position above threshold (6km away)
    let player_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(6_000.0, 0.0, 0.0)))
        .id();
    app.insert_resource(PlayerShipEntity(player_id));

    // Spawn another entity at 1.6km from player (in front)
    let other_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(7_600.0, 0.0, 0.0))) // 6000 + 1600
        .id();

    run_fixed_update(&mut app);

    // Check that the origin was recentered to the player's position
    let origin = app.world().get_resource::<FloatingOrigin>().unwrap();
    assert_eq!(
        origin.offset,
        Vec3::new(6_000.0, 0.0, 0.0),
        "origin should be recentered to player position"
    );

    // Check that the player is now at the origin (local position 0,0,0)
    let player_transform = app.world().get::<Transform>(player_id).unwrap();
    assert_eq!(
        player_transform.translation,
        Vec3::ZERO,
        "player should be at origin after recentering"
    );

    // Check that the other entity is now at 1600 (relative to player)
    let other_transform = app.world().get::<Transform>(other_id).unwrap();
    assert_eq!(
        other_transform.translation,
        Vec3::new(1_600.0, 0.0, 0.0),
        "other entity should be at relative position after recentering"
    );
}

#[test]
fn test_floating_origin_accumulates_offset_with_nonzero_initial_offset() {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    // Start with a non-zero offset to test accumulation
    app.insert_resource(FloatingOrigin::new(Vec3::new(10_000.0, 0.0, 0.0)));
    app.insert_resource(FloatingOriginConfig {
        recenter_threshold_m: 5_000.0,
    });

    app.add_systems(
        FixedUpdate,
        crate::floating_origin_systems::check_and_recenter_origin_system,
    );

    // Spawn player ship at local position (-6000, 0, 0) which has distance 6000 from origin
    // This will trigger recentering since 6000 > 5000 threshold
    // Absolute position = (-6000, 0, 0) + (10000, 0, 0) = (4000, 0, 0)
    let player_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(-6_000.0, 0.0, 0.0)))
        .id();
    app.insert_resource(PlayerShipEntity(player_id));

    // Spawn another entity at 1.6km from player (in +X direction in local space)
    // Other entity at local position (-6000 + 1600, 0, 0) = (-4400, 0, 0)
    // Absolute position = (-4400, 0, 0) + (10000, 0, 0) = (5600, 0, 0)
    let other_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(-4_400.0, 0.0, 0.0)))
        .id();

    run_fixed_update(&mut app);

    // After recentering:
    // Player was at local position (-6000, 0, 0) relative to origin at (10000, 0, 0)
    // New offset should be (10000, 0, 0) + (-6000, 0, 0) = (4000, 0, 0)
    let origin = app.world().get_resource::<FloatingOrigin>().unwrap();
    assert_eq!(
        origin.offset,
        Vec3::new(4_000.0, 0.0, 0.0),
        "origin should be accumulated to player's local position"
    );

    // Player should be at origin (0, 0, 0)
    let player_transform = app.world().get::<Transform>(player_id).unwrap();
    assert_eq!(
        player_transform.translation,
        Vec3::ZERO,
        "player should be at origin after recentering"
    );

    // Other entity should be at 1600 (relative to player)
    let other_transform = app.world().get::<Transform>(other_id).unwrap();
    assert_eq!(
        other_transform.translation,
        Vec3::new(1_600.0, 0.0, 0.0),
        "other entity should be at relative position after recentering"
    );
}

#[test]
fn test_floating_origin_preserves_rotation_on_recenter() {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

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

    // Spawn player ship at position above threshold (6km away)
    let player_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(6_000.0, 0.0, 0.0)))
        .id();
    app.insert_resource(PlayerShipEntity(player_id));

    // Spawn another entity with a specific rotation
    let rotation = Quat::from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_4); // 45 degrees
    let other_id = app
        .world_mut()
        .spawn(Transform {
            translation: Vec3::new(7_600.0, 0.0, 0.0),
            rotation,
            ..default()
        })
        .id();

    run_fixed_update(&mut app);

    // Check that the other entity's rotation is preserved
    let other_transform = app.world().get::<Transform>(other_id).unwrap();
    assert_eq!(
        other_transform.rotation, rotation,
        "other entity's rotation should be preserved after recentering"
    );
}

#[test]
fn test_floating_origin_preserves_scale_on_recenter() {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

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

    // Spawn player ship at position above threshold (6km away)
    let player_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(6_000.0, 0.0, 0.0)))
        .id();
    app.insert_resource(PlayerShipEntity(player_id));

    // Spawn another entity with a non-default scale (e.g., 400.0 for a scaled-down sun)
    let scale = Vec3::new(400.0, 400.0, 400.0);
    let other_id = app
        .world_mut()
        .spawn(Transform {
            translation: Vec3::new(7_600.0, 0.0, 0.0),
            scale,
            ..default()
        })
        .id();

    run_fixed_update(&mut app);

    // Check that the other entity's scale is preserved after recentering
    let other_transform = app.world().get::<Transform>(other_id).unwrap();
    assert_eq!(
        other_transform.scale, scale,
        "other entity's scale should be preserved after recentering"
    );
}

// ---------------------------------------------------------------------------
// P4: RigidBody panic tests
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
