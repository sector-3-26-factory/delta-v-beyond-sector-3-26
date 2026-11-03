// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Approach-comparison benchmarks for physics, converted to Strategy Z2.
//!
//! Per ADR-0056 the directory name is the benchmark purpose, and every file
//! in it MUST declare that same purpose in its `BENCHMARK-PURPOSE` header.
//! This module is compiled into the binary behind the `bench` feature flag,
//! which is independent of `test` so benchmarks are excluded from day-to-day
//! `cargo test --workspace`. They still run on the same real plugin stack:
//! the binary is built with `--features test,bench`.
//!
//! Each benchmark is a `fn(app: &mut App)` registered via `init_registry()`,
//! with timing measured via `std::time::Instant` around `run_fixed_update`
//! calls. The shared `App` is render-settled by `main.rs` before dispatch.
//!
//! Run them with:
//! `cargo test --test benchmarks_tests_driver --features bench -p delta-v-physics-tests`
//!
//! BENCHMARK-PURPOSE: approach-comparison
//! BENCHMARK-BASELINE: sibling-variants-same-run
//! BENCHMARK-SCENARIO: physics-capacity-curve
//! BENCHMARK-RUN: cargo test --test benchmarks_tests_driver --features bench -p delta-v-physics-tests -- --ignored --nocapture

#![cfg(feature = "bench")]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::used_underscore_binding,
    unsafe_code
)]

use bevy::prelude::*;
use std::time::Instant;

use delta_v_physics::celestial::{Moon, OrbitalBody, Planet, Sun};
use delta_v_physics::collision::{CollisionLayersComponent, CollisionShape, CollisionShapeType};
use delta_v_physics::rigid_body::{MassSource, RigidBody};
use delta_v_test_framework::{reset_fixed_time, run_fixed_update};
use delta_v_types::CollisionShapeData;
use delta_v_types::collision::layers;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Converts a benchmark count to `f32`.
fn count_as_f32(value: usize) -> f32 {
    f32::from(u16::try_from(value).expect("benchmark count must fit in u16"))
}

/// Converts a sample count to `f64` for averaging frame times.
fn len_as_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("sample count must fit in u32"))
}

/// Spawns a planet (celestial body with SOI).
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

/// Spawns a dynamic body (ship/asteroid) that feels gravity.
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

/// Runs a benchmark with the given number of celestial and dynamic bodies.
fn run_benchmark(
    app: &mut App,
    num_celestial: usize,
    num_dynamic: usize,
    num_frames: usize,
) -> BenchmarkResult {
    // Spawn celestial bodies (Sun + planets)
    for i in 0..num_celestial {
        let index = count_as_f32(i);
        let distance = 1.496e11 * (index + 1.0);
        let mass = 5.972e24 * (index + 1.0);
        spawn_planet(app, Vec3::new(distance, 0.0, 0.0), mass, distance);
    }

    // Spawn dynamic bodies (ships/asteroids) distributed around
    for i in 0..num_dynamic {
        let index = count_as_f32(i);
        let angle = index * std::f32::consts::TAU / count_as_f32(num_dynamic);
        let radius = index.mul_add(100_000.0, 6.371e6 + 400_000.0);
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        spawn_dynamic_body(app, Vec3::new(x, y, 0.0), 1000.0);
    }

    // Warm-up frames
    for _ in 0..10 {
        reset_fixed_time(app);
        run_fixed_update(app);
    }

    // Measure frames
    let mut frame_times = Vec::with_capacity(num_frames);
    for _ in 0..num_frames {
        reset_fixed_time(app);
        let start = Instant::now();
        run_fixed_update(app);
        let elapsed = start.elapsed();
        frame_times.push(elapsed.as_secs_f64() * 1000.0);
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
    let moons_per_planet = num_moons.div_ceil(SOLAR_SYSTEM_PLANETS.len());
    let mut moon_count = 0;

    for &(_name, _mass, planet_distance, _period) in &SOLAR_SYSTEM_PLANETS {
        for moon_idx in 0..moons_per_planet {
            if moon_count >= num_moons {
                break;
            }
            let index = count_as_f32(moon_idx);
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
                    orbital_parent: Entity::PLACEHOLDER,
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

/// Runs a benchmark with the given number of celestial and dynamic bodies.
fn run_scaling_benchmark(
    app: &mut App,
    num_celestial: usize,
    num_dynamic: usize,
    num_frames: usize,
) -> BenchmarkResult {
    run_benchmark(app, num_celestial, num_dynamic, num_frames)
}

/// Benchmark scaling: 14 configs of (celestial, dynamic) bodies.
///
/// Converted from `benchmark_scaling` in `performance_benchmarks_tests.rs`.
/// The shared `App` is already render-settled by `main.rs`; we spawn bodies
/// into it and measure `run_fixed_update` cost directly.
pub fn benchmark_scaling(app: &mut App) {
    let configs = [
        (1, 10),
        (1, 20),
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

    println!("\n=== PHYSICS PERFORMANCE BENCHMARK ===");
    println!(
        "{:>8} {:>8} {:>8} {:>12} {:>12} {:>12} {:>10}",
        "Celest", "Dynamic", "Total", "Avg(ms)", "Min(ms)", "Max(ms)", "Frames"
    );
    println!("{}", "-".repeat(80));

    let mut results = Vec::new();

    for (celestial, dynamic) in configs {
        let result = run_scaling_benchmark(app, celestial, dynamic, 100);
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

    for r in &results {
        if r.avg_frame_time_ms > 16.67 {
            println!(
                "\nWARNING: {} objects exceeds 60 FPS budget ({:.2} ms > 16.67 ms)",
                r.total_objects, r.avg_frame_time_ms
            );
        }
        if r.avg_frame_time_ms > 33.33 {
            println!(
                "CRITICAL: {} objects exceeds 30 FPS budget ({:.2} ms > 33.33 ms)",
                r.total_objects, r.avg_frame_time_ms
            );
        }
    }
}

/// Benchmark that simulates the actual solar system world loading.
///
/// Converted from `benchmark_solar_system_simulation` in `performance_benchmarks_tests.rs`.
pub fn benchmark_solar_system_simulation(app: &mut App) {
    let configs = [10, 20, 50, 100, 200, 289];

    println!("\n=== SOLAR SYSTEM SIMULATION BENCHMARK (with orbital motion) ===");
    println!(
        "{:>8} {:>12} {:>12} {:>12} {:>10}",
        "Objects", "Avg(ms)", "Min(ms)", "Max(ms)", "Frames"
    );

    for num_moons in configs {
        let sun_id = spawn_benchmark_sun(app);
        spawn_benchmark_planets(app, sun_id);
        spawn_benchmark_moons(app, num_moons);

        for _ in 0..10 {
            reset_fixed_time(app);
            run_fixed_update(app);
        }

        let mut times = Vec::new();
        for _ in 0..100 {
            reset_fixed_time(app);
            let start = Instant::now();
            run_fixed_update(app);
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }

        let avg = times.iter().sum::<f64>() / len_as_f64(times.len());
        let min = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        println!(
            "{:>8} {:>12.3} {:>12.3} {:>12.3} {:>10}",
            num_moons + 9,
            avg,
            min,
            max,
            100
        );
    }
}

/// Benchmark isolating just the gravity system cost.
///
/// Converted from `benchmark_gravity_system_only` in `performance_benchmarks_tests.rs`.
pub fn benchmark_gravity_system_only(app: &mut App) {
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

        for _ in 0..10 {
            reset_fixed_time(app);
            run_fixed_update(app);
        }

        let mut times = Vec::new();
        for _ in 0..100 {
            reset_fixed_time(app);
            let start = Instant::now();
            run_fixed_update(app);
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
// ---------------------------------------------------------------------------

/// Deterministic 32-bit LCG so belt layouts are reproducible across runs.
struct Lcg(u64);

impl Lcg {
    fn next_f32(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let bits = (self.0 >> 40) as u32;
        let value = bits as f32;
        value / 16_777_216.0
    }
}

/// One astronomical unit in metres.
const AU: f32 = 1.496e11;

/// Spawns `count` asteroids spread across a main-belt-like volume.
fn spawn_belt(app: &mut App, count: usize) {
    let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let mut rng = Lcg(0x5DEE_CE66_D1CE_B00D);

    for i in 0..count {
        let a = 1.2f32.mul_add(rng.next_f32(), 2.1) * AU;
        let theta = i as f32 * golden_angle;
        let y = (rng.next_f32() - 0.5) * 4.0e10;

        app.world_mut().spawn((
            RigidBody::new(1.0e15, 1.0),
            Transform::from_translation(Vec3::new(a * theta.cos(), y, a * theta.sin())),
            CollisionShape(CollisionShapeData {
                shape_type: CollisionShapeType::Sphere { radius: 250.0 },
                offset: Vec3::ZERO,
            }),
            CollisionLayersComponent::new(layers::ASTEROID),
        ));
    }
}

/// Spawns `count` asteroids of which the first `relevant` carry a relevance value
/// above the threshold and the rest carry one below it.
fn spawn_belt_with_relevance(app: &mut App, count: usize, relevant: usize) {
    spawn_belt(app, count);
    let threshold = delta_v_physics::constants::COLLISION_RELEVANCE_PX;

    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<CollisionShape>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    for (index, entity) in entities.into_iter().enumerate() {
        let is_relevant = index < relevant;
        let screen_radius_px = if is_relevant {
            threshold * 10.0
        } else {
            threshold * 0.01
        };
        app.world_mut()
            .entity_mut(entity)
            .insert(delta_v_physics::celestial::LazyLoadMesh {
                min_screen_radius_px: 1.0,
                loaded: true,
                mesh_path: String::new(),
                current_screen_radius_px: screen_radius_px,
                collision_relevance_px: threshold,
                mesh_child_entities: Vec::new(),
            });
    }
}

/// One variant of the relevance benchmark: how many of `count` asteroids are relevant.
struct RelevanceVariant {
    label: &'static str,
    relevant: usize,
}

/// Measures collision detection cost against asteroid count.
///
/// Converted from `benchmark_collision` in `performance_benchmarks_tests.rs`.
pub fn benchmark_collision(app: &mut App) {
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
        spawn_belt(app, count);

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
            reset_fixed_time(app);
            run_fixed_update(app);
        }

        let mut times = Vec::with_capacity(frames);
        for _ in 0..frames {
            reset_fixed_time(app);
            let start = Instant::now();
            run_fixed_update(app);
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

/// Measures the collision relevance filter against the unfiltered loop.
///
/// Converted from `benchmark_collision_relevance` in `performance_benchmarks_tests.rs`.
pub fn benchmark_collision_relevance(app: &mut App) {
    let configs = [500usize, 1_000, 2_000, 4_000, 8_000];
    let warmup = 3;
    let frames = 15;

    println!("\n=== COLLISION RELEVANCE BENCHMARK (ADR-0057) ===");
    println!(
        "{:>9} {:>9} {:>10} {:>11} {:>11} {:>10} {:>10}",
        "Asteroids", "Relevant", "Pairs all", "Pairs filt", "Median(ms)", "Reduction", "ns/pair"
    );
    println!("{}", "-".repeat(80));

    for count in configs {
        let variants = [
            RelevanceVariant {
                label: "unfiltered",
                relevant: count,
            },
            RelevanceVariant {
                label: "10% relevant",
                relevant: count / 10,
            },
            RelevanceVariant {
                label: "1% relevant",
                relevant: count / 100,
            },
        ];

        let mut medians: Vec<(&'static str, f64)> = Vec::new();

        for variant in &variants {
            spawn_belt_with_relevance(app, count, variant.relevant);

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
                reset_fixed_time(app);
                run_fixed_update(app);
            }

            let mut times = Vec::with_capacity(frames);
            for _ in 0..frames {
                reset_fixed_time(app);
                let start = Instant::now();
                run_fixed_update(app);
                times.push(start.elapsed().as_secs_f64() * 1000.0);
            }

            times.sort_by(f64::total_cmp);
            let Some(&median) = times.get(times.len() / 2) else {
                continue;
            };
            medians.push((variant.label, median));

            let pairs_all = (count * count.saturating_sub(1)) / 2;
            let n_relevant = variant.relevant;
            let pairs_filtered = (n_relevant * n_relevant.saturating_sub(1)) / 2;
            let ns_per_pair = if pairs_filtered == 0 {
                0.0
            } else {
                median * 1.0e6 / f64::from(u32::try_from(pairs_filtered).unwrap_or(u32::MAX))
            };

            println!(
                "{count:>9} {n_relevant:>9} {pairs_all:>10} {pairs_filtered:>11} {median:>11.3} {:>10} {ns_per_pair:>10.2}",
                "-",
            );
        }

        if let Some((_, baseline)) = medians.first().copied()
            && baseline > 0.0
        {
            for (label, median) in medians.iter().skip(1) {
                println!(
                    "  {count:>7} asteroids, {label:<14} -> {median:>8.3} ms  ({:.1}x faster than unfiltered)",
                    baseline / median
                );
            }
        }
    }

    println!(
        "\ns/pair uses the filtered pair count n_relevant^2/2, which is what the loop actually runs."
    );
    println!(
        "Reduction is the unfiltered median divided by the filtered median, measured in the same run."
    );
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Register all benchmark functions in the global registry.
///
/// Called once at binary startup (from `main.rs` when the `bench`
/// feature is enabled). Each benchmark is registered with its
/// display name and a `fn(&mut App)` entry point.
///
/// The registration calls must be inside a function body — they cannot
/// be module-level statements.
#[cfg(feature = "bench")]
pub fn init_registry() {
    do_register("physics::benchmark_scaling", benchmark_scaling);
    do_register(
        "physics::benchmark_solar_system_simulation",
        benchmark_solar_system_simulation,
    );
    do_register(
        "physics::benchmark_gravity_system_only",
        benchmark_gravity_system_only,
    );
    do_register("physics::benchmark_collision", benchmark_collision);
    do_register(
        "physics::benchmark_collision_relevance",
        benchmark_collision_relevance,
    );
}

#[cfg(not(feature = "bench"))]
pub fn init_registry() {}

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
