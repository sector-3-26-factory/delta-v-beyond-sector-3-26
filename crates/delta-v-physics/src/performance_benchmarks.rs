// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Performance benchmarks for physics system scaling.
//!
//! Measures CPU time per frame for different object counts to identify bottlenecks.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp)]

use bevy::prelude::*;
use bevy::time::TimePlugin;
use std::time::Instant;

use crate::celestial::{Moon, OrbitalBody, Planet, Sun};
use crate::constants::GRAVITATIONAL_CONSTANT;
use crate::rigid_body::{MassSource, RigidBody};
use crate::systems::{
    PhysicsSet, clear_accumulators_system, gravity_system, integrate_angular_velocity_system,
    integrate_position_system, integrate_velocity_system,
};

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

fn run_fixed_update(app: &mut App) {
    let fixed_duration = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(fixed_duration);
    app.update();
}

/// Spawns a planet (celestial body with SOI)
fn spawn_planet(app: &mut App, position: Vec3, mass: f32, orbital_distance: f32) -> Entity {
    app.world_mut()
        .spawn((
            RigidBody::new(mass, 1.0),
            Transform::from_translation(position),
            MassSource,
            Planet {
                rotation_period: Some(24.0 * 3600.0),
                axial_tilt: 0.0,
                animations_enabled: false,
            },
            OrbitalBody {
                orbital_parent: Entity::PLACEHOLDER,
                orbital_distance,
                orbital_period: 365.25 * 24.0 * 3600.0,
                orbital_eccentricity: 0.0167,
                orbital_inclination: 0.0,
                initial_orbital_angle: 0.0,
            },
        ))
        .id()
}

/// Spawns a dynamic body (ship/asteroid) that feels gravity
fn spawn_dynamic_body(app: &mut App, position: Vec3, mass: f32) -> Entity {
    app.world_mut()
        .spawn((
            RigidBody::new(mass, 1.0),
            Transform::from_translation(position),
        ))
        .id()
}

/// Benchmark result for a single configuration
#[derive(Debug, Clone)]
struct BenchmarkResult {
    num_celestial_bodies: usize,
    num_dynamic_bodies: usize,
    total_objects: usize,
    avg_frame_time_ms: f64,
    min_frame_time_ms: f64,
    max_frame_time_ms: f64,
    frames_measured: usize,
}

/// Runs a benchmark with the given number of celestial and dynamic bodies
fn run_benchmark(num_celestial: usize, num_dynamic: usize, num_frames: usize) -> BenchmarkResult {
    let mut app = build_physics_app();

    // Spawn celestial bodies (Sun + planets)
    let mut celestial_entities = Vec::new();
    for i in 0..num_celestial {
        let distance = 1.496e11 * (i as f32 + 1.0); // Spread out along X axis
        let mass = 5.972e24 * (i as f32 + 1.0); // Earth mass scaled
        let entity = spawn_planet(&mut app, Vec3::new(distance, 0.0, 0.0), mass, distance);
        celestial_entities.push(entity);
    }

    // Spawn dynamic bodies (ships/asteroids) distributed around
    let mut dynamic_entities = Vec::new();
    for i in 0..num_dynamic {
        let angle = (i as f32) * std::f32::consts::TAU / (num_dynamic as f32);
        let radius = 6.371e6 + 400_000.0 + (i as f32 * 100_000.0); // Different altitudes
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let entity = spawn_dynamic_body(&mut app, Vec3::new(x, y, 0.0), 1000.0);
        dynamic_entities.push(entity);
    }

    // Warm-up frames
    for _ in 0..10 {
        run_fixed_update(&mut app);
    }

    // Measure frames
    let mut frame_times = Vec::with_capacity(num_frames);
    for _ in 0..num_frames {
        let start = Instant::now();
        run_fixed_update(&mut app);
        let elapsed = start.elapsed();
        frame_times.push(elapsed.as_secs_f64() * 1000.0); // Convert to ms
    }

    let avg = frame_times.iter().sum::<f64>() / frame_times.len() as f64;
    let min = frame_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = frame_times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    BenchmarkResult {
        num_celestial_bodies: num_celestial,
        num_dynamic_bodies: num_dynamic,
        total_objects: num_celestial + num_dynamic,
        avg_frame_time_ms: avg,
        min_frame_time_ms: min,
        max_frame_time_ms: max,
        frames_measured: num_frames,
    }
}

#[test]
fn benchmark_scaling() {
    // Test configurations: (celestial_bodies, dynamic_bodies)
    let configs = vec![
        (1, 10),   // 11 total
        (1, 20),   // 21 total
        (1, 50),   // 51 total
        (1, 100),  // 101 total
        (1, 200),  // 201 total
        (1, 300),  // 301 total
        (5, 10),   // 15 total
        (5, 50),   // 55 total
        (5, 100),  // 105 total
        (5, 200),  // 205 total
        (10, 10),  // 20 total
        (10, 50),  // 60 total
        (10, 100), // 110 total
        (10, 200), // 210 total
    ];

    println!("\n=== PHYSICS PERFORMANCE BENCHMARK ===");
    println!(
        "{:>8} {:>8} {:>8} {:>12} {:>12} {:>12} {:>10}",
        "Celest", "Dynamic", "Total", "Avg(ms)", "Min(ms)", "Max(ms)", "Frames"
    );
    println!("{}", "-".repeat(80));

    let mut results = Vec::new();

    for (celestial, dynamic) in configs {
        let result = run_benchmark(celestial, dynamic, 100);
        println!(
            "{:>8} {:>8} {:>8} {:>12.3} {:>12.3} {:>12.3} {:>10}",
            result.num_celestial_bodies,
            result.num_dynamic_bodies,
            result.total_objects,
            result.avg_frame_time_ms,
            result.min_frame_time_ms,
            result.max_frame_time_ms,
            result.frames_measured
        );
        results.push(result);
    }

    println!("\n=== SCALING ANALYSIS ===");
    println!("Fixed 1 celestial, varying dynamic:");
    for r in &results {
        if r.num_celestial_bodies == 1 {
            println!(
                "  {} dynamic: {:.3} ms/frame",
                r.num_dynamic_bodies, r.avg_frame_time_ms
            );
        }
    }

    println!("\nFixed 5 celestial, varying dynamic:");
    for r in &results {
        if r.num_celestial_bodies == 5 {
            println!(
                "  {} dynamic: {:.3} ms/frame",
                r.num_dynamic_bodies, r.avg_frame_time_ms
            );
        }
    }

    println!("\nFixed 10 celestial, varying dynamic:");
    for r in &results {
        if r.num_celestial_bodies == 10 {
            println!(
                "  {} dynamic: {:.3} ms/frame",
                r.num_dynamic_bodies, r.avg_frame_time_ms
            );
        }
    }

    // Check for concerning performance
    for r in &results {
        if r.avg_frame_time_ms > 16.67 {
            // 60 FPS budget
            println!(
                "\n⚠️  WARNING: {} objects exceeds 60 FPS budget ({:.2} ms > 16.67 ms)",
                r.total_objects, r.avg_frame_time_ms
            );
        }
        if r.avg_frame_time_ms > 33.33 {
            // 30 FPS budget
            println!(
                "🚨 CRITICAL: {} objects exceeds 30 FPS budget ({:.2} ms > 33.33 ms)",
                r.total_objects, r.avg_frame_time_ms
            );
        }
    }
}

/// Benchmark that simulates the actual solar system world loading
/// This tests the full pipeline including orbital motion for all bodies
#[test]
fn benchmark_solar_system_simulation() {
    let configs = vec![
        10, 20, 50, 100, 200, 289, // 289 = full solar system moons
    ];

    println!("\n=== SOLAR SYSTEM SIMULATION BENCHMARK (with orbital motion) ===");
    println!(
        "{:>8} {:>12} {:>12} {:>12} {:>10}",
        "Objects", "Avg(ms)", "Min(ms)", "Max(ms)", "Frames"
    );

    for num_moons in configs {
        let mut app = build_physics_app();

        // Spawn Sun
        let sun_id = app
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

        // Spawn planets (8 planets)
        let planet_data = vec![
            ("mercury", 3.301e23, 5.79e10, 88.0 * 24.0 * 3600.0),
            ("venus", 4.867e24, 1.082e11, 225.0 * 24.0 * 3600.0),
            ("earth", 5.972e24, 1.496e11, 365.25 * 24.0 * 3600.0),
            ("mars", 6.417e23, 2.279e11, 687.0 * 24.0 * 3600.0),
            (
                "jupiter",
                1.898e27,
                7.785e11,
                11.86 * 365.25 * 24.0 * 3600.0,
            ),
            ("saturn", 5.683e26, 1.433e12, 29.46 * 365.25 * 24.0 * 3600.0),
            ("uranus", 8.681e25, 2.872e12, 84.01 * 365.25 * 24.0 * 3600.0),
            (
                "neptune",
                1.024e26,
                4.495e12,
                164.8 * 365.25 * 24.0 * 3600.0,
            ),
        ];

        for (i, (name, mass, distance, period)) in planet_data.iter().enumerate() {
            app.world_mut().spawn((
                RigidBody::new(*mass, 1.0),
                Transform::from_translation(Vec3::new(*distance, 0.0, 0.0)),
                MassSource,
                Planet {
                    rotation_period: Some(24.0 * 3600.0),
                    axial_tilt: 0.0,
                    animations_enabled: false,
                },
                OrbitalBody {
                    orbital_parent: sun_id,
                    orbital_distance: *distance,
                    orbital_period: *period,
                    orbital_eccentricity: 0.01,
                    orbital_inclination: 0.0,
                    initial_orbital_angle: (i as f32) * 0.5,
                },
            ));
        }

        // Spawn moons (up to num_moons)
        // Distribute them around the planets
        let moons_per_planet = (num_moons as f32 / 8.0).ceil() as usize;
        let mut moon_count = 0;

        for (planet_idx, (_, _, planet_distance, _)) in planet_data.iter().enumerate() {
            for moon_idx in 0..moons_per_planet {
                if moon_count >= num_moons {
                    break;
                }
                let moon_distance = planet_distance + 1e8 + (moon_idx as f32 * 5e7); // Different orbital radii
                let moon_mass = 1e20 + (moon_idx as f32 * 1e19);
                let moon_period = 24.0 * 3600.0 * (moon_idx as f32 + 1.0) * 0.1;

                app.world_mut().spawn((
                    RigidBody::new(moon_mass, 1.0),
                    Transform::from_translation(Vec3::new(moon_distance, 0.0, 0.0)),
                    MassSource,
                    Moon {
                        rotation_period: Some(moon_period),
                        axial_tilt: 0.0,
                        animations_enabled: false,
                    },
                    OrbitalBody {
                        orbital_parent: Entity::PLACEHOLDER, // Would be resolved to planet in real spawn
                        orbital_distance: moon_distance,
                        orbital_period: moon_period,
                        orbital_eccentricity: 0.001,
                        orbital_inclination: 0.01,
                        initial_orbital_angle: (moon_idx as f32) * 0.1,
                    },
                ));
                moon_count += 1;
            }
            if moon_count >= num_moons {
                break;
            }
        }

        // Warm-up
        for _ in 0..10 {
            run_fixed_update(&mut app);
        }

        // Measure
        let mut times = Vec::new();
        for _ in 0..100 {
            let start = Instant::now();
            run_fixed_update(&mut app);
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }

        let avg = times.iter().sum::<f64>() / times.len() as f64;
        let min = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        println!(
            "{:>8} {:>12.3} {:>12.3} {:>12.3} {:>10}",
            num_moons + 9, // 1 sun + 8 planets + moons
            avg,
            min,
            max,
            100
        );
    }
}

#[test]
fn benchmark_gravity_system_only() {
    // Isolate just the gravity system to measure its cost
    let configs = vec![
        (1, 10),
        (1, 50),
        (1, 100),
        (1, 200),
        (1, 300),
        (5, 10),
        (5, 50),
        (5, 100),
        (5, 200),
        (10, 10),
        (10, 50),
        (10, 100),
        (10, 200),
    ];

    println!("\n=== GRAVITY SYSTEM ONLY BENCHMARK ===");
    println!(
        "{:>8} {:>8} {:>8} {:>12} {:>12} {:>12}",
        "Celest", "Dynamic", "Total", "Avg(ms)", "Min(ms)", "Max(ms)"
    );

    for (celestial, dynamic) in configs {
        // Build fresh app for each configuration (App doesn't implement Clone)
        let mut app = App::new();
        app.add_plugins(TimePlugin);
        app.insert_resource(Time::<Fixed>::from_hz(60.0));
        app.configure_sets(FixedUpdate, (PhysicsSet::AccumulateForces,));
        app.add_systems(
            FixedUpdate,
            gravity_system.in_set(PhysicsSet::AccumulateForces),
        );

        // Spawn celestial bodies
        for i in 0..celestial {
            let distance = 1.496e11 * (i as f32 + 1.0);
            let mass = 5.972e24 * (i as f32 + 1.0);
            app.world_mut().spawn((
                RigidBody::new(mass, 1.0),
                Transform::from_translation(Vec3::new(distance, 0.0, 0.0)),
                MassSource,
                Planet {
                    rotation_period: Some(24.0 * 3600.0),
                    axial_tilt: 0.0,
                    animations_enabled: false,
                },
                OrbitalBody {
                    orbital_parent: Entity::PLACEHOLDER,
                    orbital_distance: distance,
                    orbital_period: 365.25 * 24.0 * 3600.0,
                    orbital_eccentricity: 0.0167,
                    orbital_inclination: 0.0,
                    initial_orbital_angle: 0.0,
                },
            ));
        }

        // Spawn dynamic bodies
        for i in 0..dynamic {
            let angle = (i as f32) * std::f32::consts::TAU / (dynamic as f32);
            let radius = 6.371e6 + 400_000.0 + (i as f32 * 100_000.0);
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            app.world_mut().spawn((
                RigidBody::new(1000.0, 1.0),
                Transform::from_translation(Vec3::new(x, y, 0.0)),
            ));
        }

        // Warm-up
        for _ in 0..10 {
            let fixed_duration = app.world().resource::<Time<Fixed>>().timestep();
            app.world_mut()
                .resource_mut::<Time<Real>>()
                .advance_by(fixed_duration);
            app.update();
        }

        // Measure
        let mut times = Vec::new();
        for _ in 0..100 {
            let start = Instant::now();
            let fixed_duration = app.world().resource::<Time<Fixed>>().timestep();
            app.world_mut()
                .resource_mut::<Time<Real>>()
                .advance_by(fixed_duration);
            app.update();
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }

        let avg = times.iter().sum::<f64>() / times.len() as f64;
        let min = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        println!(
            "{:>8} {:>8} {:>8} {:>12.3} {:>12.3} {:>12.3}",
            celestial,
            dynamic,
            celestial + dynamic,
            avg,
            min,
            max
        );
    }
}
