// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for collision shape scaling.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use delta_v_types::{CollisionShapeData, CollisionShapeType};

use crate::collision::scale_collision_shape;

// ---------------------------------------------------------------------------
// scale_collision_shape: sphere
// ---------------------------------------------------------------------------

#[test]
fn test_scale_collision_shape_sphere() {
    let shape = CollisionShapeData {
        shape_type: CollisionShapeType::Sphere { radius: 2.5 },
        offset: bevy::prelude::Vec3::ZERO,
    };

    let data = scale_collision_shape(&shape, 1.0);
    match data.shape_type {
        CollisionShapeType::Sphere { radius } => {
            assert!(
                (radius - 2.5).abs() < 0.01,
                "radius should be 2.5, got {radius}"
            );
        }
        other => panic!("expected Sphere, got {other:?}"),
    }
    assert_eq!(
        data.offset,
        bevy::prelude::Vec3::ZERO,
        "offset should default to ZERO"
    );
}

#[test]
fn test_scale_collision_shape_sphere_with_offset() {
    let shape = CollisionShapeData {
        shape_type: CollisionShapeType::Sphere { radius: 1.0 },
        offset: bevy::prelude::Vec3::new(1.0, 2.0, 3.0),
    };

    let data = scale_collision_shape(&shape, 1.0);
    assert_eq!(data.offset, bevy::prelude::Vec3::new(1.0, 2.0, 3.0));
}

// ---------------------------------------------------------------------------
// scale_collision_shape: box
// ---------------------------------------------------------------------------

#[test]
fn test_scale_collision_shape_box() {
    let shape = CollisionShapeData {
        shape_type: CollisionShapeType::Box {
            half_extents: bevy::prelude::Vec3::new(1.0, 2.0, 3.0),
        },
        offset: bevy::prelude::Vec3::ZERO,
    };

    let data = scale_collision_shape(&shape, 1.0);
    match data.shape_type {
        CollisionShapeType::Box { half_extents } => {
            assert_eq!(half_extents, bevy::prelude::Vec3::new(1.0, 2.0, 3.0));
        }
        other => panic!("expected Box, got {other:?}"),
    }
}

#[test]
fn test_scale_collision_shape_box_with_offset() {
    let shape = CollisionShapeData {
        shape_type: CollisionShapeType::Box {
            half_extents: bevy::prelude::Vec3::new(1.0, 1.0, 1.0),
        },
        offset: bevy::prelude::Vec3::new(0.5, 0.5, 0.5),
    };

    let data = scale_collision_shape(&shape, 1.0);
    assert_eq!(data.offset, bevy::prelude::Vec3::new(0.5, 0.5, 0.5));
}

// ---------------------------------------------------------------------------
// scale_collision_shape: scaling
// ---------------------------------------------------------------------------

#[test]
fn test_scale_collision_shape_sphere_with_scale() {
    let shape = CollisionShapeData {
        shape_type: CollisionShapeType::Sphere { radius: 10.0 },
        offset: bevy::prelude::Vec3::ZERO,
    };

    // Scale by 100x
    let data = scale_collision_shape(&shape, 100.0);
    match data.shape_type {
        CollisionShapeType::Sphere { radius } => {
            assert!(
                (radius - 1000.0).abs() < 0.01,
                "radius should be scaled to 1000.0, got {radius}"
            );
        }
        other => panic!("expected Sphere, got {other:?}"),
    }
}

#[test]
fn test_scale_collision_shape_sphere_with_scale_less_than_one() {
    let shape = CollisionShapeData {
        shape_type: CollisionShapeType::Sphere { radius: 100.0 },
        offset: bevy::prelude::Vec3::ZERO,
    };

    // Scale by 0.1x
    let data = scale_collision_shape(&shape, 0.1);
    match data.shape_type {
        CollisionShapeType::Sphere { radius } => {
            assert!(
                (radius - 10.0).abs() < 0.01,
                "radius should be scaled to 10.0, got {radius}"
            );
        }
        other => panic!("expected Sphere, got {other:?}"),
    }
}

#[test]
fn test_scale_collision_shape_box_with_scale() {
    let shape = CollisionShapeData {
        shape_type: CollisionShapeType::Box {
            half_extents: bevy::prelude::Vec3::new(10.0, 20.0, 30.0),
        },
        offset: bevy::prelude::Vec3::ZERO,
    };

    // Scale by 10x
    let data = scale_collision_shape(&shape, 10.0);
    match data.shape_type {
        CollisionShapeType::Box { half_extents } => {
            assert_eq!(half_extents, bevy::prelude::Vec3::new(100.0, 200.0, 300.0));
        }
        other => panic!("expected Box, got {other:?}"),
    }
}

#[test]
fn test_scale_collision_shape_offset_scaled() {
    let shape = CollisionShapeData {
        shape_type: CollisionShapeType::Sphere { radius: 5.0 },
        offset: bevy::prelude::Vec3::new(1.0, 2.0, 3.0),
    };

    // Scale by 10x
    let data = scale_collision_shape(&shape, 10.0);
    assert_eq!(
        data.offset,
        bevy::prelude::Vec3::new(10.0, 20.0, 30.0),
        "offset should be scaled"
    );
}
