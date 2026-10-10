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
    GeneratedAsteroid, SizeDistributionEntry, detail_keep_test, pick_mesh, pick_radius,
    separate_overlaps, stable_hash,
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
