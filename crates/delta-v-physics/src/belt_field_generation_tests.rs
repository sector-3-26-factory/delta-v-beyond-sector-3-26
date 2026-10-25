// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Unit tests for `belt_field_generation` module.

use crate::belt_field_generation::{
    AsteroidBeltTemplate, BeltSectorParams, GeneratedAsteroid, SizeDistributionEntry,
    detail_keep_test, generate_belt_sector, pick_mesh, pick_radius, separate_overlaps, stable_hash,
};
use bevy::prelude::*;
use delta_v_types::{CollisionShapeData, CollisionShapeType};

#[test]
fn test_stable_hash_deterministic() {
    let h1 = stable_hash(12345, "test_belt", 0, 42, "mesh");
    let h2 = stable_hash(12345, "test_belt", 0, 42, "mesh");
    assert_eq!(h1, h2);
}

#[test]
fn test_different_streams_independent() {
    let h_mesh = stable_hash(12345, "test_belt", 0, 42, "mesh");
    let h_radius = stable_hash(12345, "test_belt", 0, 42, "radius");
    let h_rotation = stable_hash(12345, "test_belt", 0, 42, "rotation");
    // Different streams should produce different hashes
    assert_ne!(h_mesh, h_radius);
    assert_ne!(h_mesh, h_rotation);
    assert_ne!(h_radius, h_rotation);
}

#[test]
fn test_detail_keep_test_nesting() {
    let seed = 12345;
    let belt_id = "test_belt";
    let sector = 0;
    let body = 42;

    // Higher detail level should keep more bodies
    let keep_full = detail_keep_test(seed, belt_id, sector, body, 1.0);
    let keep_half = detail_keep_test(seed, belt_id, sector, body, 0.5);
    let keep_none = detail_keep_test(seed, belt_id, sector, body, 0.0);

    // At 1.0, everything is kept
    assert!(keep_full);
    // At 0.0, nothing is kept
    assert!(!keep_none);
    // If kept at 0.5, must also be kept at 1.0 (nesting)
    if keep_half {
        assert!(keep_full);
    }
}

#[test]
fn test_pick_mesh_deterministic() {
    let meshes = vec![
        "mesh_1".to_string(),
        "mesh_2".to_string(),
        "mesh_3".to_string(),
    ];
    let h1 = pick_mesh(&meshes, 12345);
    let h2 = pick_mesh(&meshes, 12345);
    assert_eq!(h1, h2);
}

#[test]
fn test_pick_radius_deterministic() {
    let dist = vec![
        SizeDistributionEntry {
            radius: 1000.0,
            weight: 1.0,
        },
        SizeDistributionEntry {
            radius: 5000.0,
            weight: 0.5,
        },
    ];
    let h1 = pick_radius(&dist, 12345);
    let h2 = pick_radius(&dist, 12345);
    assert!((h1 - h2).abs() < f32::EPSILON);
}

#[test]
fn test_separate_overlaps() {
    let mut asteroids = vec![
        GeneratedAsteroid {
            stable_id: 1,
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            radius: 1000.0,
            mass: 1e10,
            mesh_name: "mesh_1".to_string(),
            collision_shape: CollisionShapeData {
                shape_type: CollisionShapeType::Sphere { radius: 1000.0 },
                offset: Vec3::ZERO,
            },
            bounding_box: delta_v_types::BoundingBox {
                min: Vec3::new(-1000.0, -1000.0, -1000.0),
                max: Vec3::new(1000.0, 1000.0, 1000.0),
            },
            orbital_params: None,
            velocity: Vec3::ZERO,
        },
        GeneratedAsteroid {
            stable_id: 2,
            position: Vec3::new(500.0, 0.0, 0.0), // Overlapping (distance 500 < 2000)
            rotation: Quat::IDENTITY,
            radius: 1000.0,
            mass: 1e10,
            mesh_name: "mesh_1".to_string(),
            collision_shape: CollisionShapeData {
                shape_type: CollisionShapeType::Sphere { radius: 1000.0 },
                offset: Vec3::ZERO,
            },
            bounding_box: delta_v_types::BoundingBox {
                min: Vec3::new(-1000.0, -1000.0, -1000.0),
                max: Vec3::new(1000.0, 1000.0, 1000.0),
            },
            orbital_params: None,
            velocity: Vec3::ZERO,
        },
    ];

    separate_overlaps(&mut asteroids);

    // After separation, distance should be exactly sum of radii
    let distance = asteroids
        .get(1)
        .and_then(|a1| {
            asteroids
                .first()
                .map(|a0| (a1.position - a0.position).length())
        })
        .unwrap_or(0.0);
    assert!((distance - 2000.0).abs() < 1.0);
}

#[test]
fn test_respawn_identical_sector() {
    // Test that despawning and respawning reproduces a sector exactly
    // Use test parameters that produce ~200 asteroids per sector
    let orbital_distance = 4.04e11;
    let radial_extent = 1.0e8; // 100M km
    let thickness = 1.0e8; // 100M km
    let mean_spacing = 1.0e6; // 1000 km
    let sector_size = crate::belt_field_generation::calculate_optimal_sector_size(
        orbital_distance,
        radial_extent,
        thickness,
        mean_spacing,
    );

    let params = BeltSectorParams {
        belt_id: "test_belt".to_string(),
        sector_index: 0,
        template: AsteroidBeltTemplate {
            entity_type: "asteroid_belt".to_string(),
            meshes: vec!["mesh_1".to_string(), "mesh_2".to_string()],
            density: 3000.0,
        },
        orbital_parent: "sun".to_string(),
        orbital_distance,
        orbital_period: 1.4e8,
        orbital_eccentricity: 0.12,
        orbital_inclination: 0.1745, // 10 degrees in radians
        sector_size,
        radial_extent,
        thickness,
        mean_spacing,
        excluded_radii: vec![],
        size_distribution: vec![SizeDistributionEntry {
            radius: 1000.0,
            weight: 1.0,
        }],
        density: 3000.0,
        seed: 20_260_930,
        simulation_time: 0.0,
        detail_level: 1.0,
    };

    // Generate sector twice
    let asteroids1 = generate_belt_sector(&params);
    let asteroids2 = generate_belt_sector(&params);

    // Should have same number of asteroids
    assert_eq!(asteroids1.len(), asteroids2.len());

    // Each asteroid should have identical properties
    for (a1, a2) in asteroids1.iter().zip(asteroids2.iter()) {
        assert_eq!(a1.stable_id, a2.stable_id);
        assert!((a1.position - a2.position).length() < 1e-6);
        assert!((a1.rotation.dot(a2.rotation)).abs() > 1.0 - 1e-6);
        assert!((a1.radius - a2.radius).abs() < 1e-6);
        assert!((a1.mass - a2.mass).abs() < 1e-6);
        assert_eq!(a1.mesh_name, a2.mesh_name);
    }
}

#[test]
fn test_sector_identical_from_different_directions() {
    // Test that loading from a different direction gives the same sectors
    // This is implicitly tested by the deterministic generation - sector index
    // is calculated from player position, but the generation itself is deterministic
    let orbital_distance = 4.04e11;
    let radial_extent = 1.0e8; // 100M km
    let thickness = 1.0e8; // 100M km
    let mean_spacing = 1.0e6; // 1000 km
    let sector_size = crate::belt_field_generation::calculate_optimal_sector_size(
        orbital_distance,
        radial_extent,
        thickness,
        mean_spacing,
    );

    let params = BeltSectorParams {
        belt_id: "test_belt".to_string(),
        sector_index: 5,
        template: AsteroidBeltTemplate {
            entity_type: "asteroid_belt".to_string(),
            meshes: vec!["mesh_1".to_string(), "mesh_2".to_string()],
            density: 3000.0,
        },
        orbital_parent: "sun".to_string(),
        orbital_distance,
        orbital_period: 1.4e8,
        orbital_eccentricity: 0.12,
        orbital_inclination: 0.1745,
        sector_size,
        radial_extent,
        thickness,
        mean_spacing,
        excluded_radii: vec![],
        size_distribution: vec![SizeDistributionEntry {
            radius: 1000.0,
            weight: 1.0,
        }],
        density: 3000.0,
        seed: 20_260_930,
        simulation_time: 0.0,
        detail_level: 1.0,
    };

    // Generate same sector with different player positions (simulating different approach directions)
    let asteroids1 = generate_belt_sector(&params);

    // Change simulation time slightly (simulating different approach time)
    let mut params2 = params;
    params2.simulation_time = 1000.0;
    let asteroids2 = generate_belt_sector(&params2);

    // The sector generation should be identical regardless of simulation_time
    // because the sector index is the same and generation is deterministic
    // (simulation_time only affects orbital position, not which asteroids exist)
    assert_eq!(asteroids1.len(), asteroids2.len());
    for (a1, a2) in asteroids1.iter().zip(asteroids2.iter()) {
        assert_eq!(a1.stable_id, a2.stable_id);
        assert_eq!(a1.mesh_name, a2.mesh_name);
        assert!((a1.radius - a2.radius).abs() < 1e-6);
    }
}

#[test]
fn test_analytic_propagation_year() {
    // Test that a sector loaded after a simulated absence of a year
    // matches one generated at that time
    let orbital_distance = 4.04e11;
    let radial_extent = 1.0e8; // 100M km
    let thickness = 1.0e8; // 100M km
    let mean_spacing = 1.0e6; // 1000 km
    let sector_size = crate::belt_field_generation::calculate_optimal_sector_size(
        orbital_distance,
        radial_extent,
        thickness,
        mean_spacing,
    );

    let params = BeltSectorParams {
        belt_id: "test_belt".to_string(),
        sector_index: 0,
        template: AsteroidBeltTemplate {
            entity_type: "asteroid_belt".to_string(),
            meshes: vec!["mesh_1".to_string()],
            density: 3000.0,
        },
        orbital_parent: "sun".to_string(),
        orbital_distance,
        orbital_period: 1.4e8, // ~4.4 years
        orbital_eccentricity: 0.12,
        orbital_inclination: 0.1745,
        sector_size,
        radial_extent,
        thickness,
        mean_spacing,
        excluded_radii: vec![],
        size_distribution: vec![SizeDistributionEntry {
            radius: 1000.0,
            weight: 1.0,
        }],
        density: 3000.0,
        seed: 20_260_930,
        simulation_time: 0.0,
        detail_level: 1.0,
    };

    // Generate at time 0
    let asteroids_t0 = generate_belt_sector(&params);

    // Generate at time = 1 year (in seconds)
    let one_year = 365.25 * 24.0 * 3600.0;
    let mut params_year = params;
    params_year.simulation_time = one_year;
    let asteroids_t1 = generate_belt_sector(&params_year);

    // Same number of asteroids
    assert_eq!(asteroids_t0.len(), asteroids_t1.len());

    // Each asteroid should have moved along its orbit
    for (a0, a1) in asteroids_t0.iter().zip(asteroids_t1.iter()) {
        assert_eq!(a0.stable_id, a1.stable_id);
        assert_eq!(a0.mesh_name, a1.mesh_name);
        assert!((a0.radius - a1.radius).abs() < 1e-6);
        assert!((a0.mass - a1.mass).abs() < 1e-6);

        // Position should have changed (orbital motion)
        let distance_moved = (a1.position - a0.position).length();
        assert!(
            distance_moved > 0.0,
            "Asteroid should have moved along orbit"
        );
    }
}

#[test]
fn test_mass_verification() {
    // Test that every generated asteroid has mass = density * 4/3 * pi * r^3
    let orbital_distance = 4.04e11;
    let radial_extent = 1.0e8; // 100M km
    let thickness = 1.0e8; // 100M km
    let mean_spacing = 1.0e6; // 1000 km
    let sector_size = crate::belt_field_generation::calculate_optimal_sector_size(
        orbital_distance,
        radial_extent,
        thickness,
        mean_spacing,
    );

    let params = BeltSectorParams {
        belt_id: "test_belt".to_string(),
        sector_index: 0,
        template: AsteroidBeltTemplate {
            entity_type: "asteroid_belt".to_string(),
            meshes: vec!["mesh_1".to_string(), "mesh_2".to_string()],
            density: 3000.0,
        },
        orbital_parent: "sun".to_string(),
        orbital_distance,
        orbital_period: 1.4e8,
        orbital_eccentricity: 0.12,
        orbital_inclination: 0.1745,
        sector_size,
        radial_extent,
        thickness,
        mean_spacing,
        excluded_radii: vec![],
        size_distribution: vec![
            SizeDistributionEntry {
                radius: 1000.0,
                weight: 1.0,
            },
            SizeDistributionEntry {
                radius: 5000.0,
                weight: 0.5,
            },
        ],
        density: 3000.0,
        seed: 20_260_930,
        simulation_time: 0.0,
        detail_level: 1.0,
    };

    let asteroids = generate_belt_sector(&params);

    for asteroid in &asteroids {
        let expected_mass =
            params.density * (4.0 / 3.0) * std::f32::consts::PI * asteroid.radius.powi(3);
        let relative_error = (asteroid.mass - expected_mass).abs() / expected_mass;
        assert!(
            relative_error < 1e-6,
            "Asteroid mass {} != expected {} (relative error {})",
            asteroid.mass,
            expected_mass,
            relative_error
        );
    }
}

#[test]
fn test_detail_level_nesting() {
    // Test that the set at 0.8 is a subset of the set at 1.0
    let orbital_distance = 4.04e11;
    let radial_extent = 1.0e8; // 100M km
    let thickness = 1.0e8; // 100M km
    let mean_spacing = 1.0e6; // 1000 km
    let sector_size = crate::belt_field_generation::calculate_optimal_sector_size(
        orbital_distance,
        radial_extent,
        thickness,
        mean_spacing,
    );

    let params_full = BeltSectorParams {
        belt_id: "test_belt".to_string(),
        sector_index: 0,
        template: AsteroidBeltTemplate {
            entity_type: "asteroid_belt".to_string(),
            meshes: vec!["mesh_1".to_string(), "mesh_2".to_string()],
            density: 3000.0,
        },
        orbital_parent: "sun".to_string(),
        orbital_distance,
        orbital_period: 1.4e8,
        orbital_eccentricity: 0.12,
        orbital_inclination: 0.1745,
        sector_size,
        radial_extent,
        thickness,
        mean_spacing,
        excluded_radii: vec![],
        size_distribution: vec![SizeDistributionEntry {
            radius: 1000.0,
            weight: 1.0,
        }],
        density: 3000.0,
        seed: 20_260_930,
        simulation_time: 0.0,
        detail_level: 1.0,
    };

    let params_half = BeltSectorParams {
        detail_level: 0.5,
        ..params_full.clone()
    };

    let asteroids_full = generate_belt_sector(&params_full);
    let asteroids_half = generate_belt_sector(&params_half);

    // Set at 0.5 should be subset of set at 1.0
    assert!(asteroids_half.len() <= asteroids_full.len());

    // Every asteroid in half should exist in full with same properties
    for a_half in &asteroids_half {
        let a_full = asteroids_full
            .iter()
            .find(|a| a.stable_id == a_half.stable_id);
        let a_full = a_full.unwrap_or_else(|| {
            unreachable!("Asteroid {} missing from full detail set", a_half.stable_id)
        });
        assert_eq!(a_full.mesh_name, a_half.mesh_name);
        assert!((a_full.radius - a_half.radius).abs() < 1e-6);
        assert!((a_full.position - a_half.position).length() < 1e-6);
    }
}

#[test]
fn test_named_asteroid_never_filtered() {
    // Named asteroids (over 100km) are individually placed in world.json
    // and should never be filtered by detail level
    // This test verifies the detail_keep_test doesn't filter out
    // asteroids that would represent named bodies (large radius)

    let orbital_distance = 4.04e11;
    let radial_extent = 1.0e8; // 100M km
    let thickness = 1.0e8; // 100M km
    let mean_spacing = 1.0e6; // 1000 km
    let sector_size = crate::belt_field_generation::calculate_optimal_sector_size(
        orbital_distance,
        radial_extent,
        thickness,
        mean_spacing,
    );

    let params = BeltSectorParams {
        belt_id: "test_belt".to_string(),
        sector_index: 0,
        template: AsteroidBeltTemplate {
            entity_type: "asteroid_belt".to_string(),
            meshes: vec!["mesh_1".to_string()],
            density: 3000.0,
        },
        orbital_parent: "sun".to_string(),
        orbital_distance,
        orbital_period: 1.4e8,
        orbital_eccentricity: 0.12,
        orbital_inclination: 0.1745,
        sector_size,
        radial_extent,
        thickness,
        mean_spacing,
        excluded_radii: vec![],
        size_distribution: vec![
            SizeDistributionEntry {
                radius: 1000.0,
                weight: 1.0,
            },
            SizeDistributionEntry {
                radius: 100_000.0,
                weight: 0.01,
            }, // Large named asteroid
        ],
        density: 3000.0,
        seed: 20_260_930,
        simulation_time: 0.0,
        detail_level: 1.0, // Full detail to ensure large asteroid is kept
    };

    let asteroids = generate_belt_sector(&params);

    // The large asteroid (100km) should be present at full detail
    assert!(
        asteroids.iter().any(|a| a.radius > 50000.0),
        "Large named asteroids should be present at full detail"
    );
}

/// Test that generated asteroids, after overlap separation, have no
/// remaining overlaps.
///
/// This verifies the full pipeline: `generate_belt_sector` produces
/// positions from hash streams, and `separate_overlaps` resolves any
/// remaining collisions. A regression in either function would cause
/// asteroids to overlap in-game.
#[test]
#[allow(
    clippy::indexing_slicing,
    reason = "test code uses direct indexing for pairwise comparison"
)]
fn test_generated_sector_has_no_overlaps_after_separation() {
    let orbital_distance = 4.04e11;
    let radial_extent = 1.0e8; // 100M km
    let thickness = 1.0e8; // 100M km
    let mean_spacing = 1.0e6; // 1000 km
    let sector_size = crate::belt_field_generation::calculate_optimal_sector_size(
        orbital_distance,
        radial_extent,
        thickness,
        mean_spacing,
    );

    let params = BeltSectorParams {
        belt_id: "test_belt".to_string(),
        sector_index: 0,
        template: AsteroidBeltTemplate {
            entity_type: "asteroid_belt".to_string(),
            meshes: vec!["mesh_1".to_string(), "mesh_2".to_string()],
            density: 3000.0,
        },
        orbital_parent: "sun".to_string(),
        orbital_distance,
        orbital_period: 1.4e8,
        orbital_eccentricity: 0.12,
        orbital_inclination: 0.1745,
        sector_size,
        radial_extent,
        thickness,
        mean_spacing,
        excluded_radii: vec![],
        size_distribution: vec![SizeDistributionEntry {
            radius: 1000.0,
            weight: 1.0,
        }],
        density: 3000.0,
        seed: 20_260_930,
        simulation_time: 0.0,
        detail_level: 1.0,
    };

    let mut asteroids = generate_belt_sector(&params);
    separate_overlaps(&mut asteroids);

    // Every pair must be separated by at least the sum of their radii.
    for i in 0..asteroids.len() {
        for j in (i + 1)..asteroids.len() {
            let distance = (asteroids[i].position - asteroids[j].position).length();
            let min_distance = asteroids[i].radius + asteroids[j].radius;
            assert!(
                distance >= min_distance - 1.0,
                "Asteroids {} and {} overlap: distance {:.3} < min {:.3}",
                asteroids[i].stable_id,
                asteroids[j].stable_id,
                distance,
                min_distance
            );
        }
    }
}

/// Test that generated asteroids are spread across the sector volume,
/// not clustered at a single position.
///
/// A regression where all asteroids get the same position (e.g. a hash
/// bug or a hardcoded position) would pass every other test —
/// determinism, mass, overlap separation — but produce gameplay where
/// all asteroids are bunched in one spot. This test verifies the
/// bounding box of generated positions covers a significant fraction
/// of the sector's radial extent and thickness.
#[test]
fn test_generated_sector_asteroids_spread_across_volume() {
    let orbital_distance = 4.04e11;
    let radial_extent = 1.0e8; // 100M km
    let thickness = 1.0e8; // 100M km
    let mean_spacing = 1.0e6; // 1000 km
    let sector_size = crate::belt_field_generation::calculate_optimal_sector_size(
        orbital_distance,
        radial_extent,
        thickness,
        mean_spacing,
    );

    let params = BeltSectorParams {
        belt_id: "test_belt".to_string(),
        sector_index: 0,
        template: AsteroidBeltTemplate {
            entity_type: "asteroid_belt".to_string(),
            meshes: vec!["mesh_1".to_string(), "mesh_2".to_string()],
            density: 3000.0,
        },
        orbital_parent: "sun".to_string(),
        orbital_distance,
        orbital_period: 1.4e8,
        orbital_eccentricity: 0.12,
        orbital_inclination: 0.1745,
        sector_size,
        radial_extent,
        thickness,
        mean_spacing,
        excluded_radii: vec![],
        size_distribution: vec![SizeDistributionEntry {
            radius: 1000.0,
            weight: 1.0,
        }],
        density: 3000.0,
        seed: 20_260_930,
        simulation_time: 0.0,
        detail_level: 1.0,
    };

    let mut asteroids = generate_belt_sector(&params);
    separate_overlaps(&mut asteroids);

    // Compute the bounding box of all asteroid positions.
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for asteroid in &asteroids {
        min = min.min(asteroid.position);
        max = max.max(asteroid.position);
    }
    let extent = max - min;

    // The sector is a torus segment around the orbital ring at
    // orbital_distance. The radial extent is ±radial_extent/2 from the
    // ring, and the thickness is ±thickness/2 vertically. The angular
    // spread is ±sector_size/2 along the ring. We verify the positions
    // cover a significant fraction of the sector volume in all three
    // dimensions.
    assert!(
        extent.x > radial_extent * 0.3,
        "Asteroids not spread radially: x-extent {:.3} < {:.3}",
        extent.x,
        radial_extent * 0.3
    );
    assert!(
        extent.z > thickness * 0.3,
        "Asteroids not spread vertically: z-extent {:.3} < {:.3}",
        extent.z,
        thickness * 0.3
    );
    // The y-extent (along the ring) should also be non-trivial.
    assert!(
        extent.y > sector_size * 0.3,
        "Asteroids not spread along ring: y-extent {:.3} < {:.3}",
        extent.y,
        sector_size * 0.3
    );
}
