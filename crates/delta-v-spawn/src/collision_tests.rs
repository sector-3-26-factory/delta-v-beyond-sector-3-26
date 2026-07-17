// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for collision shape conversion from JSON.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use delta_v_types::{CollisionShapeJson, CollisionShapeType};

use crate::collision::shape_from_json;

// ---------------------------------------------------------------------------
// shape_from_json: sphere
// ---------------------------------------------------------------------------

#[test]
fn test_shape_from_json_sphere() {
    let json = CollisionShapeJson {
        shape_type: "sphere".to_string(),
        radius: Some(delta_v_types::PhysicalQuantityJson {
            value: 2.5,
            unit: "m".to_string(),
        }),
        half_extents: None,
        offset: None,
    };

    let result = shape_from_json(&json, 1.0);
    assert!(result.is_ok(), "sphere shape should convert successfully");
    let data = result.unwrap();
    match data.shape_type {
        CollisionShapeType::Sphere { radius } => {
            assert!(
                (radius - 2.5).abs() < 0.01,
                "radius should be 2.5, got {radius}"
            );
        }
        other => panic!("expected Sphere, got {other:?}"),
    }
    assert_eq!(data.offset, Vec3::ZERO, "offset should default to ZERO");
}

#[test]
fn test_shape_from_json_sphere_with_offset() {
    let json = CollisionShapeJson {
        shape_type: "sphere".to_string(),
        radius: Some(delta_v_types::PhysicalQuantityJson {
            value: 1.0,
            unit: "m".to_string(),
        }),
        half_extents: None,
        offset: Some(delta_v_types::Vec3Json {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
    };

    let result = shape_from_json(&json, 1.0);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data.offset, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_shape_from_json_sphere_missing_radius() {
    let json = CollisionShapeJson {
        shape_type: "sphere".to_string(),
        radius: None,
        half_extents: None,
        offset: None,
    };

    let result = shape_from_json(&json, 1.0);
    assert!(result.is_err(), "sphere without radius should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("radius"),
        "error should mention radius, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// shape_from_json: box
// ---------------------------------------------------------------------------

#[test]
fn test_shape_from_json_box() {
    let json = CollisionShapeJson {
        shape_type: "box".to_string(),
        radius: None,
        half_extents: Some(delta_v_types::Vec3Json {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
        offset: None,
    };

    let result = shape_from_json(&json, 1.0);
    assert!(result.is_ok(), "box shape should convert successfully");
    let data = result.unwrap();
    match data.shape_type {
        CollisionShapeType::Box { half_extents } => {
            assert_eq!(half_extents, Vec3::new(1.0, 2.0, 3.0));
        }
        other => panic!("expected Box, got {other:?}"),
    }
}

#[test]
fn test_shape_from_json_box_with_offset() {
    let json = CollisionShapeJson {
        shape_type: "box".to_string(),
        radius: None,
        half_extents: Some(delta_v_types::Vec3Json {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }),
        offset: Some(delta_v_types::Vec3Json {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        }),
    };

    let result = shape_from_json(&json, 1.0);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data.offset, Vec3::new(0.5, 0.5, 0.5));
}

#[test]
fn test_shape_from_json_box_missing_half_extents() {
    let json = CollisionShapeJson {
        shape_type: "box".to_string(),
        radius: None,
        half_extents: None,
        offset: None,
    };

    let result = shape_from_json(&json, 1.0);
    assert!(result.is_err(), "box without half_extents should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("half_extents"),
        "error should mention half_extents, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// shape_from_json: unknown type
// ---------------------------------------------------------------------------

#[test]
fn test_shape_from_json_unknown_type() {
    let json = CollisionShapeJson {
        shape_type: "cylinder".to_string(),
        radius: None,
        half_extents: None,
        offset: None,
    };

    let result = shape_from_json(&json, 1.0);
    assert!(result.is_err(), "unknown shape type should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("Unknown"),
        "error should mention unknown type, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// shape_from_json: scaling
// ---------------------------------------------------------------------------

#[test]
fn test_shape_from_json_sphere_with_scale() {
    let json = CollisionShapeJson {
        shape_type: "sphere".to_string(),
        radius: Some(delta_v_types::PhysicalQuantityJson {
            value: 10.0,
            unit: "m".to_string(),
        }),
        half_extents: None,
        offset: None,
    };

    // Scale by 100x
    let result = shape_from_json(&json, 100.0);
    assert!(result.is_ok());
    let data = result.unwrap();
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
fn test_shape_from_json_sphere_with_scale_less_than_one() {
    let json = CollisionShapeJson {
        shape_type: "sphere".to_string(),
        radius: Some(delta_v_types::PhysicalQuantityJson {
            value: 100.0,
            unit: "m".to_string(),
        }),
        half_extents: None,
        offset: None,
    };

    // Scale by 0.1x
    let result = shape_from_json(&json, 0.1);
    assert!(result.is_ok());
    let data = result.unwrap();
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
fn test_shape_from_json_box_with_scale() {
    let json = CollisionShapeJson {
        shape_type: "box".to_string(),
        radius: None,
        half_extents: Some(delta_v_types::Vec3Json {
            x: 10.0,
            y: 20.0,
            z: 30.0,
        }),
        offset: None,
    };

    // Scale by 10x
    let result = shape_from_json(&json, 10.0);
    assert!(result.is_ok());
    let data = result.unwrap();
    match data.shape_type {
        CollisionShapeType::Box { half_extents } => {
            assert_eq!(half_extents, Vec3::new(100.0, 200.0, 300.0));
        }
        other => panic!("expected Box, got {other:?}"),
    }
}

#[test]
fn test_shape_from_json_offset_scaled() {
    let json = CollisionShapeJson {
        shape_type: "sphere".to_string(),
        radius: Some(delta_v_types::PhysicalQuantityJson {
            value: 5.0,
            unit: "m".to_string(),
        }),
        half_extents: None,
        offset: Some(delta_v_types::Vec3Json {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
    };

    // Scale by 10x
    let result = shape_from_json(&json, 10.0);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(
        data.offset,
        Vec3::new(10.0, 20.0, 30.0),
        "offset should be scaled"
    );
}

// Re-export for test use
use bevy::prelude::Vec3;
