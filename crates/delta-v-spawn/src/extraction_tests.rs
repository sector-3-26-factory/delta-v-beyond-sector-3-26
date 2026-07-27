// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for template field extraction helpers.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp)]

use crate::template_extraction::{
    compute_debug_axis_length, extract_bounding_box, extract_mass, resolve_mass, scale_bounding_box,
};
use delta_v_types::{
    AsteroidTemplate, BoundingBox, CollisionShapeData, CollisionShapeType, EntityTemplate,
};

// ---------------------------------------------------------------------------
// extract_mass
// ---------------------------------------------------------------------------

#[test]
fn test_extract_mass_basic() {
    let template = EntityTemplate::Asteroid(AsteroidTemplate {
        entity_type: "asteroid".to_string(),
        mass: 10_000.0,
        bounding_box: BoundingBox {
            min: bevy::prelude::Vec3::new(0.0, 0.0, 0.0),
            max: bevy::prelude::Vec3::new(1.0, 1.0, 1.0),
        },
        collision_shape: CollisionShapeData {
            shape_type: CollisionShapeType::Sphere { radius: 1.0 },
            offset: bevy::prelude::Vec3::ZERO,
        },
        is_gravity_source: false,
    });
    let mass = extract_mass(&template);
    assert!(
        (mass - 10_000.0).abs() < 0.01,
        "mass should be 10000, got {mass}"
    );
}

#[test]
fn test_extract_mass_small_value() {
    let template = EntityTemplate::Asteroid(AsteroidTemplate {
        entity_type: "asteroid".to_string(),
        mass: 0.5,
        bounding_box: BoundingBox {
            min: bevy::prelude::Vec3::new(0.0, 0.0, 0.0),
            max: bevy::prelude::Vec3::new(1.0, 1.0, 1.0),
        },
        collision_shape: CollisionShapeData {
            shape_type: CollisionShapeType::Sphere { radius: 1.0 },
            offset: bevy::prelude::Vec3::ZERO,
        },
        is_gravity_source: false,
    });
    let mass = extract_mass(&template);
    assert!((mass - 0.5).abs() < 0.01, "mass should be 0.5, got {mass}");
}

// ---------------------------------------------------------------------------
// extract_bounding_box
// ---------------------------------------------------------------------------

#[test]
fn test_extract_bounding_box_basic() {
    let template = EntityTemplate::Asteroid(AsteroidTemplate {
        entity_type: "asteroid".to_string(),
        mass: 1000.0,
        bounding_box: BoundingBox {
            min: bevy::prelude::Vec3::new(-1.0, -2.0, -3.0),
            max: bevy::prelude::Vec3::new(1.0, 2.0, 3.0),
        },
        collision_shape: CollisionShapeData {
            shape_type: CollisionShapeType::Sphere { radius: 1.0 },
            offset: bevy::prelude::Vec3::ZERO,
        },
        is_gravity_source: false,
    });

    let bbox = extract_bounding_box(&template);
    assert!((bbox.min.x - (-1.0)).abs() < 0.01);
    assert!((bbox.min.y - (-2.0)).abs() < 0.01);
    assert!((bbox.min.z - (-3.0)).abs() < 0.01);
    assert!((bbox.max.x - 1.0).abs() < 0.01);
    assert!((bbox.max.y - 2.0).abs() < 0.01);
    assert!((bbox.max.z - 3.0).abs() < 0.01);
}

#[test]
fn test_extract_bounding_box_asymmetric() {
    let template = EntityTemplate::Asteroid(AsteroidTemplate {
        entity_type: "asteroid".to_string(),
        mass: 1000.0,
        bounding_box: BoundingBox {
            min: bevy::prelude::Vec3::new(0.0, 0.0, 0.0),
            max: bevy::prelude::Vec3::new(10.0, 5.0, 2.0),
        },
        collision_shape: CollisionShapeData {
            shape_type: CollisionShapeType::Sphere { radius: 1.0 },
            offset: bevy::prelude::Vec3::ZERO,
        },
        is_gravity_source: false,
    });

    let bbox = extract_bounding_box(&template);
    assert!((bbox.min.x - 0.0).abs() < 0.01);
    assert!((bbox.min.y - 0.0).abs() < 0.01);
    assert!((bbox.min.z - 0.0).abs() < 0.01);
    assert!((bbox.max.x - 10.0).abs() < 0.01);
    assert!((bbox.max.y - 5.0).abs() < 0.01);
    assert!((bbox.max.z - 2.0).abs() < 0.01);
}

// ---------------------------------------------------------------------------
// compute_debug_axis_length
// ---------------------------------------------------------------------------

#[test]
fn test_compute_debug_axis_length_cube() {
    let bbox = BoundingBox {
        min: bevy::prelude::Vec3::new(-1.0, -1.0, -1.0),
        max: bevy::prelude::Vec3::new(1.0, 1.0, 1.0),
    };
    // Size = (2, 2, 2), max_dim = 2, axis_length = 2 * 1.2 = 2.4
    let length = compute_debug_axis_length(&bbox);
    assert!(
        (length - 2.4).abs() < 0.01,
        "axis length should be 2.4, got {length}"
    );
}

#[test]
fn test_compute_debug_axis_length_rectangular() {
    let bbox = BoundingBox {
        min: bevy::prelude::Vec3::new(-5.0, -1.0, -2.0),
        max: bevy::prelude::Vec3::new(5.0, 1.0, 2.0),
    };
    // Size = (10, 2, 4), max_dim = 10, axis_length = 10 * 1.2 = 12
    let length = compute_debug_axis_length(&bbox);
    assert!(
        (length - 12.0).abs() < 0.01,
        "axis length should be 12.0, got {length}"
    );
}

#[test]
fn test_compute_debug_axis_length_unit() {
    let bbox = BoundingBox {
        min: bevy::prelude::Vec3::new(0.0, 0.0, 0.0),
        max: bevy::prelude::Vec3::new(1.0, 1.0, 1.0),
    };
    // Size = (1, 1, 1), max_dim = 1, axis_length = 1.2
    let length = compute_debug_axis_length(&bbox);
    assert!(
        (length - 1.2).abs() < 0.01,
        "axis length should be 1.2, got {length}"
    );
}

// ---------------------------------------------------------------------------
// scale_bounding_box
// ---------------------------------------------------------------------------

#[test]
fn test_scale_bounding_box() {
    let bbox = BoundingBox {
        min: bevy::prelude::Vec3::new(-10.0, -10.0, -10.0),
        max: bevy::prelude::Vec3::new(10.0, 10.0, 10.0),
    };

    let scaled = scale_bounding_box(&bbox, 100.0);
    assert_eq!(scaled.min.x, -1000.0);
    assert_eq!(scaled.min.y, -1000.0);
    assert_eq!(scaled.min.z, -1000.0);
    assert_eq!(scaled.max.x, 1000.0);
    assert_eq!(scaled.max.y, 1000.0);
    assert_eq!(scaled.max.z, 1000.0);
}

#[test]
fn test_scale_bounding_box_fractional() {
    let bbox = BoundingBox {
        min: bevy::prelude::Vec3::new(-100.0, -100.0, -100.0),
        max: bevy::prelude::Vec3::new(100.0, 100.0, 100.0),
    };

    let scaled = scale_bounding_box(&bbox, 0.1);
    assert_eq!(scaled.min.x, -10.0);
    assert_eq!(scaled.min.y, -10.0);
    assert_eq!(scaled.min.z, -10.0);
    assert_eq!(scaled.max.x, 10.0);
    assert_eq!(scaled.max.y, 10.0);
    assert_eq!(scaled.max.z, 10.0);
}

// ---------------------------------------------------------------------------
// resolve_mass
// ---------------------------------------------------------------------------

#[test]
fn test_resolve_mass_with_override() {
    let result = resolve_mass(1000.0, Some(5000.0));
    assert_eq!(result, 5000.0, "should use override value");
}

#[test]
fn test_resolve_mass_without_override() {
    let result = resolve_mass(1000.0, None);
    assert_eq!(result, 1000.0, "should use template mass when no override");
}

#[test]
fn test_resolve_mass_zero_override() {
    // Note: zero override is technically valid (though may be invalid for physics)
    let result = resolve_mass(1000.0, Some(0.0));
    assert_eq!(result, 0.0, "should use override value even if zero");
}
