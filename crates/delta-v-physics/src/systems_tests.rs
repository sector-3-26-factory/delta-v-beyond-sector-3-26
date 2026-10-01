// AGENTS: before modifying this file, read AGENTS.md at the repository root.

#![cfg(test)]

//! Tests for physics integration systems (SOI gravity model per ADR-0055).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use bevy::prelude::*;
use bevy::time::TimePlugin;

use crate::celestial::{OrbitalBody, Planet, Sun};
use crate::constants::GRAVITATIONAL_CONSTANT;
use crate::rigid_body::{MassSource, RigidBody};
use crate::systems::{GravityAttractor, PhysicsSet, gravity_system};

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
fn test_soi_gravity_ship_feels_planet_gravity() {
    let mut app = build_test_app();

    // Spawn a planet (celestial body with MassSource and OrbitalBody)
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
                orbital_parent: Entity::PLACEHOLDER, // Sun would be parent
                orbital_distance: 1.496e11,          // 1 AU
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
            RigidBody::new(1000.0, 1.0), // 1 ton ship
            Transform::from_translation(Vec3::new(6.371e6 + 400_000.0, 0.0, 0.0)), // 400km above surface
        ))
        .id();

    run_fixed_update(&mut app);

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    // Ship should feel gravity toward planet (negative X direction)
    assert!(
        ship.force_accumulator.x < 0.0,
        "ship should feel gravity toward planet (negative X), got {}",
        ship.force_accumulator.x
    );
    assert!(
        ship.force_accumulator.y.abs() < f32::EPSILON,
        "no Y force expected, got {}",
        ship.force_accumulator.y
    );
    assert!(
        ship.force_accumulator.z.abs() < f32::EPSILON,
        "no Z force expected, got {}",
        ship.force_accumulator.z
    );
}

#[test]
fn test_soi_gravity_force_magnitude() {
    let mut app = build_test_app();

    let planet_mass = 5.972e24_f32; // Earth mass
    let ship_mass = 1000.0_f32;
    let distance = 6.371e6 + 400_000.0; // Earth radius + 400km altitude

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

    run_fixed_update(&mut app);

    let expected_force = GRAVITATIONAL_CONSTANT * planet_mass * ship_mass / (distance * distance);

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    let actual_force = ship.force_accumulator.x.abs();

    let relative_error = (actual_force - expected_force).abs() / expected_force;
    assert!(
        relative_error < 0.001,
        "force magnitude mismatch: expected {expected_force}, got {actual_force} (relative error {relative_error})"
    );
}

#[test]
fn test_soi_gravity_sun_soi_encompasses_all() {
    let mut app = build_test_app();

    // Spawn the Sun (no orbital parent = infinite SOI)
    let _sun_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1.989e30, 1.0), // Solar mass
            Transform::from_translation(Vec3::ZERO),
            MassSource,
            Sun {
                rotation_period: Some(25.0 * 24.0), // ~25 days in hours
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

    // Spawn a ship far from the Sun (at Earth orbit distance)
    let ship_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::new(1.496e11, 0.0, 0.0)), // 1 AU
        ))
        .id();

    run_fixed_update(&mut app);

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    // Ship should feel Sun's gravity (negative X direction)
    assert!(
        ship.force_accumulator.x < 0.0,
        "ship should feel Sun's gravity (negative X), got {}",
        ship.force_accumulator.x
    );
}

#[test]
fn test_soi_gravity_no_celestial_bodies_no_gravity() {
    let mut app = build_test_app();

    // Only a ship, no celestial bodies
    let ship_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        ))
        .id();

    run_fixed_update(&mut app);

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    assert_eq!(
        ship.force_accumulator,
        Vec3::ZERO,
        "no gravity without celestial bodies"
    );
}

#[test]
fn test_soi_gravity_attractor_overrides() {
    let mut app = build_test_app();

    // Spawn a planet
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

    // Spawn a gravity attractor (black hole) near the ship
    let _attractor_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1e12, 1.0), // Small black hole
            Transform::from_translation(Vec3::new(1000.0, 0.0, 0.0)),
            GravityAttractor {
                radius: 5000.0,            // 5km radius
                strength_multiplier: 10.0, // 10x gravity
            },
        ))
        .id();

    // Spawn a ship near the attractor
    let ship_id = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::new(2000.0, 0.0, 0.0)), // 2km from attractor
        ))
        .id();

    run_fixed_update(&mut app);

    let ship = app.world().get::<RigidBody>(ship_id).unwrap();
    // Ship should feel BOTH planet gravity AND attractor gravity
    // Attractor is at (1000,0,0), ship at (2000,0,0) -> attractor pulls negative X
    // Planet is at (0,0,0), ship at (2000,0,0) -> planet pulls negative X
    // Both pull in same direction, so force should be larger than planet alone
    assert!(
        ship.force_accumulator.x < 0.0,
        "ship should feel combined gravity (negative X), got {}",
        ship.force_accumulator.x
    );

    // Verify attractor contributes (force should be stronger than planet alone at this distance)
    let planet_only_force = GRAVITATIONAL_CONSTANT * 5.972e24 * 1000.0 / (2000.0 * 2000.0);
    let attractor_force = GRAVITATIONAL_CONSTANT * 1e12 * 1000.0 / (1000.0 * 1000.0) * 10.0;
    let expected_total = planet_only_force + attractor_force;

    let actual_force = ship.force_accumulator.x.abs();
    let relative_error = (actual_force - expected_total).abs() / expected_total;
    assert!(
        relative_error < 0.01,
        "combined force mismatch: expected {expected_total}, got {actual_force} (relative error {relative_error})"
    );
}

#[test]
fn test_soi_gravity_deterministic() {
    let run_scenario = || -> f32 {
        let mut app = build_test_app();

        // Spawn a planet
        app.world_mut().spawn((
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
        ));

        let ship_id = app
            .world_mut()
            .spawn((
                RigidBody::new(1000.0, 1.0),
                Transform::from_translation(Vec3::new(6.371e6 + 400_000.0, 0.0, 0.0)),
            ))
            .id();

        run_fixed_update(&mut app);

        let ship = app.world().get::<RigidBody>(ship_id).unwrap();
        ship.force_accumulator.x
    };

    let result1 = run_scenario();
    let result2 = run_scenario();

    assert!(
        (result1 - result2).abs() < f32::EPSILON,
        "SOI gravity must be deterministic: run 1 = {result1}, run 2 = {result2}"
    );
}
