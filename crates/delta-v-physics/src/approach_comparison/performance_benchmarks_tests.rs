// AGENTS: before modifying this file, read AGENTS.md at the repository root.

#![cfg(test)]

//! Performance benchmarks for physics system scaling.
//!
//! Measures CPU time per frame for different object counts to identify bottlenecks.
//!
//! BENCHMARK-PURPOSE: approach-comparison
//! BENCHMARK-BASELINE: sibling-variants-same-run
//! BENCHMARK-SCENARIO: physics-capacity-curve
//! BENCHMARK-RUN: cargo test -p delta-v-physics --features bench -- --ignored --nocapture
//!
//! Per ADR-0056 this file is an `approach-comparison` benchmark: every run
//! measures sibling entity counts in one process, so the numbers are
//! relative to each other and the hardware is irrelevant. `criterion` is
//! deliberately not used here — it compares a benchmark against its own
//! previous run, which is a `regression-tracking` baseline, not this one.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp)]

use bevy::prelude::*;
use bevy::time::TimePlugin;
use std::time::Instant;

use crate::CollisionDetected;
use crate::celestial::{Moon, OrbitalBody, Planet, Sun};
use crate::collision::{CollisionLayersComponent, CollisionShape, CollisionShapeType};
use crate::rigid_body::{MassSource, RigidBody};
use crate::systems::{
    PhysicsSet, clear_accumulators_system, gravity_system, integrate_angular_velocity_system,
    integrate_position_system, integrate_velocity_system,
};
use delta_v_types::CollisionShapeData;

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

/// Converts a benchmark count to `f32`.
///
/// `f32` implements `From` only for the integer types it can represent
/// exactly, so the conversion goes through `u16` and the narrowing is a
/// checked conversion rather than an `as` cast. Every count in this file is
/// orders of magnitude below `u16::MAX`; exceeding it panics instead of
/// silently producing a wrong benchmark layout.
fn count_as_f32(value: usize) -> f32 {
    f32::from(u16::try_from(value).expect("benchmark count must fit in u16"))
}

/// Converts a sample count to `f64` for averaging frame times.
///
/// `f64` represents every `u32` exactly, so this conversion is lossless.
fn len_as_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("sample count must fit in u32"))
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
    for i in 0..num_celestial {
        let index = count_as_f32(i);
        let distance = 1.496e11 * (index + 1.0); // Spread out along X axis
        let mass = 5.972e24 * (index + 1.0); // Earth mass scaled
        spawn_planet(&mut app, Vec3::new(distance, 0.0, 0.0), mass, distance);
    }

    // Spawn dynamic bodies (ships/asteroids) distributed around
    for i in 0..num_dynamic {
        let index = count_as_f32(i);
        let angle = index * std::f32::consts::TAU / count_as_f32(num_dynamic);
        let radius = index.mul_add(100_000.0, 6.371e6 + 400_000.0); // Different altitudes
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        spawn_dynamic_body(&mut app, Vec3::new(x, y, 0.0), 1000.0);
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

    let avg = frame_times.iter().sum::<f64>() / len_as_f64(frame_times.len());
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
    let configs = [
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

/// Name, mass in kg, orbital distance in m and orbital period in s for the
/// eight planets, in order.
const SOLAR_SYSTEM_PLANETS: [(&str, f32, f32, f32); 8] = [
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

/// Spawns the benchmark Sun and returns its entity.
fn spawn_benchmark_sun(app: &mut App) -> Entity {
    app.world_mut()
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
        .id()
}

/// Spawns the eight benchmark planets orbiting `sun_id`.
fn spawn_benchmark_planets(app: &mut App, sun_id: Entity) {
    for (i, (_name, mass, distance, period)) in SOLAR_SYSTEM_PLANETS.iter().enumerate() {
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
                initial_orbital_angle: count_as_f32(i) * 0.5,
            },
        ));
    }
}

/// Spawns up to `num_moons` benchmark moons, spread across the planets.
fn spawn_benchmark_moons(app: &mut App, num_moons: usize) {
    // Spawn moons (up to num_moons)
    // Distribute them around the planets
    let moons_per_planet = num_moons.div_ceil(SOLAR_SYSTEM_PLANETS.len());
    let mut moon_count = 0;

    for &(_name, _mass, planet_distance, _period) in &SOLAR_SYSTEM_PLANETS {
        for moon_idx in 0..moons_per_planet {
            if moon_count >= num_moons {
                break;
            }
            let index = count_as_f32(moon_idx);
            // Different orbital radii
            let moon_distance = index.mul_add(5e7, planet_distance + 1e8);
            let moon_mass = index.mul_add(1e19, 1e20);
            let moon_period = 24.0 * 3600.0 * (index + 1.0) * 0.1;

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
                    initial_orbital_angle: index * 0.1,
                },
            ));
            moon_count += 1;
        }
        if moon_count >= num_moons {
            break;
        }
    }
}

/// Benchmark that simulates the actual solar system world loading
/// This tests the full pipeline including orbital motion for all bodies
#[test]
fn benchmark_solar_system_simulation() {
    let configs = [
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
        let sun_id = spawn_benchmark_sun(&mut app);

        // Spawn planets (8 planets)
        spawn_benchmark_planets(&mut app, sun_id);

        // Spawn moons (up to num_moons), distributed around the planets
        spawn_benchmark_moons(&mut app, num_moons);

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

        let avg = times.iter().sum::<f64>() / len_as_f64(times.len());
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
            let index = count_as_f32(i);
            let distance = 1.496e11 * (index + 1.0);
            let mass = 5.972e24 * (index + 1.0);
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
            let index = count_as_f32(i);
            let angle = index * std::f32::consts::TAU / count_as_f32(dynamic);
            let radius = index.mul_add(100_000.0, 6.371e6 + 400_000.0);
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

        let avg = times.iter().sum::<f64>() / len_as_f64(times.len());
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

// ---------------------------------------------------------------------------
// Collision detection benchmark
//
// `collision_detection_system` is O(n^2) over every body carrying
// RigidBody + Transform + CollisionShape + CollisionLayersComponent. The other
// benchmarks in this file deliberately exclude it, so they cannot show the cost
// that a large asteroid belt would actually incur.
// ---------------------------------------------------------------------------

/// Builds an app with collision detection and response registered.
fn build_collision_app() -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.add_message::<CollisionDetected>();
    app.configure_sets(FixedUpdate, (PhysicsSet::AccumulateForces,));
    app.add_systems(
        FixedUpdate,
        (
            crate::collision_detection_system,
            crate::collision_response_system,
        )
            .in_set(PhysicsSet::AccumulateForces),
    );
    app
}

/// Deterministic 32-bit LCG so belt layouts are reproducible across runs.
struct Lcg(u64);

impl Lcg {
    /// Returns a pseudo-random value in `[0, 1)`.
    fn next_f32(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // `>> 40` leaves exactly 24 significant bits, which is the full f32
        // mantissa, so the cast below is exact and the result is in [0, 1).
        // `self.0` is `u64`, so narrowing it to `u32` is a truncation cast,
        // not a precision-loss cast; that is why this allow names
        // `cast_possible_truncation` while the `u32 -> f32` conversion below
        // names `cast_precision_loss`.
        #[allow(clippy::cast_possible_truncation)]
        let bits = (self.0 >> 40) as u32;
        #[allow(clippy::cast_precision_loss)]
        let value = bits as f32;
        value / 16_777_216.0
    }
}

/// One astronomical unit in metres.
const AU: f32 = 1.496e11;

/// Spawns `count` asteroids spread across a main-belt-like volume.
///
/// Semi-major axis is sampled across 2.1-3.3 AU with a golden-angle
/// distribution in longitude, and a small vertical thickness, so bodies are
/// mostly far enough apart that few actual collisions occur. The O(n^2) pair
/// loop runs regardless of whether pairs collide, so this measures the
/// detection cost rather than a degenerate all-overlapping case.
fn spawn_belt(app: &mut App, count: usize) {
    let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let mut rng = Lcg(0x5DEE_CE66_D1CE_B00D);

    for i in 0..count {
        let a = 1.2f32.mul_add(rng.next_f32(), 2.1) * AU;
        // `count` is at most a few thousand, far inside the exactly
        // representable f32 range, so this cast is lossless.
        #[allow(clippy::cast_precision_loss)]
        let theta = i as f32 * golden_angle;
        let y = (rng.next_f32() - 0.5) * 4.0e10;

        app.world_mut().spawn((
            RigidBody::new(1.0e15, 1.0),
            Transform::from_translation(Vec3::new(a * theta.cos(), y, a * theta.sin())),
            CollisionShape(CollisionShapeData {
                shape_type: CollisionShapeType::Sphere { radius: 250.0 },
                offset: Vec3::ZERO,
            }),
            CollisionLayersComponent::new(delta_v_types::collision::layers::ASTEROID),
        ));
    }
}

/// Runs exactly one `FixedUpdate` pass.
///
/// `run_fixed_update` drives the app through `Time<Real>` + `app.update()`,
/// which makes the fixed-step accumulator decide how many ticks actually
/// execute. A tick that is slower than the timestep therefore runs *more*
/// often, and a tick faster than the timestep may run *zero* times, which
/// silently invalidates a timing. Running the schedule directly removes that
/// coupling and guarantees exactly one execution of the collision systems.
fn run_one_collision_tick(app: &mut App) {
    app.world_mut().run_schedule(FixedUpdate);
}

/// Measures collision detection cost against asteroid count.
///
/// This is the benchmark that decides how many belt asteroids the engine can
/// carry. Run it explicitly:
///
/// ```text
/// cargo test -p delta-v-physics -- --ignored --nocapture benchmark_collision
/// ```
#[test]
#[ignore = "performance benchmark; run explicitly with --ignored"]
fn benchmark_collision() {
    let configs = [500usize, 1_000, 2_000, 3_000, 4_000, 5_000, 8_000, 12_000];
    let warmup = 3;
    let frames = 15;

    println!("\n=== COLLISION DETECTION BENCHMARK ===");
    println!(
        "{:>9} {:>9} {:>13} {:>11} {:>11} {:>11} {:>9} {:>7}",
        "Asteroids", "Bodies", "Pairs/tick", "Median", "Min(ms)", "Max(ms)", "ns/pair", "Budget"
    );
    println!("{}", "-".repeat(90));

    for count in configs {
        let mut app = build_collision_app();
        spawn_belt(&mut app, count);

        // Confirm the query really sees every spawned body, otherwise the
        // timing would silently measure an empty world.
        let bodies = app
            .world_mut()
            .query::<&CollisionShape>()
            .iter(app.world())
            .count();
        assert_eq!(
            bodies, count,
            "collision query must see every spawned asteroid"
        );

        for _ in 0..warmup {
            run_one_collision_tick(&mut app);
        }

        let mut times = Vec::with_capacity(frames);
        for _ in 0..frames {
            let start = Instant::now();
            run_one_collision_tick(&mut app);
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }

        times.sort_by(f64::total_cmp);
        let (Some(&min), Some(&max), Some(&median)) =
            (times.first(), times.last(), times.get(times.len() / 2))
        else {
            continue;
        };
        let pairs = (count * count.saturating_sub(1)) / 2;
        let pair_count = f64::from(u32::try_from(pairs).unwrap_or(u32::MAX));
        let ns_per_pair = if pairs == 0 {
            0.0
        } else {
            median * 1.0e6 / pair_count
        };
        let budget = if median <= 16.67 {
            "OK"
        } else if median <= 33.33 {
            "TIGHT"
        } else {
            "OVER"
        };

        println!(
            "{count:>9} {bodies:>9} {pairs:>13} {median:>11.3} {min:>11.3} {max:>11.3} {ns_per_pair:>9.2} {budget:>7}"
        );
    }

    println!("\nBudget: 16.67 ms = 60 FPS, 33.33 ms = 30 FPS");
    println!("ns/pair is the median tick divided by the O(n^2) pair count.");
}
