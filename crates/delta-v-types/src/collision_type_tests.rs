#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for collision shape type deserialization.

#![allow(clippy::float_cmp)]

use crate::collision::{CollisionShapeJson, CollisionShapeType};
use crate::physics::PhysicalQuantity;
use crate::spatial::Vec3Json;

// ---------------------------------------------------------------------------
// CollisionShapeJson round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_collision_shape_json_deserialize_sphere() {
    let json = r#"{
        "type": "sphere",
        "radius": { "value": 2.5, "unit": "m" }
    }"#;

    let shape: CollisionShapeJson = serde_json::from_str(json).expect("should parse sphere JSON");
    assert_eq!(shape.shape_type, "sphere");
    assert!(shape.is_sphere());
    assert!(!shape.is_box());
    assert!(shape.radius.is_some());
    assert!((shape.radius.as_ref().unwrap().value - 2.5).abs() < 0.01);
    assert_eq!(shape.radius.as_ref().unwrap().unit, "m");
}

#[test]
fn test_collision_shape_json_deserialize_box() {
    let json = r#"{
        "type": "box",
        "half_extents": { "x": 1.0, "y": 2.0, "z": 3.0 }
    }"#;

    let shape: CollisionShapeJson = serde_json::from_str(json).expect("should parse box JSON");
    assert_eq!(shape.shape_type, "box");
    assert!(shape.is_box());
    assert!(!shape.is_sphere());
    assert!(shape.half_extents.is_some());
    let he = &shape.half_extents.unwrap();
    assert_eq!(he.x, 1.0);
    assert_eq!(he.y, 2.0);
    assert_eq!(he.z, 3.0);
}

#[test]
fn test_collision_shape_json_deserialize_with_offset() {
    let json = r#"{
        "type": "sphere",
        "radius": { "value": 1.0, "unit": "m" },
        "offset": { "x": 0.5, "y": 1.0, "z": -0.5 }
    }"#;

    let shape: CollisionShapeJson =
        serde_json::from_str(json).expect("should parse sphere with offset");
    assert!(shape.offset.is_some());
    let offset = shape.offset.unwrap();
    assert_eq!(offset.x, 0.5);
    assert_eq!(offset.y, 1.0);
    assert_eq!(offset.z, -0.5);
}

#[test]
fn test_collision_shape_json_shape_type_method() {
    let sphere = CollisionShapeJson {
        shape_type: "sphere".to_string(),
        radius: Some(PhysicalQuantity {
            value: 1.0,
            unit: "m".to_string(),
        }),
        half_extents: None,
        offset: None,
    };
    assert_eq!(sphere.shape_type(), "sphere");

    let box_shape = CollisionShapeJson {
        shape_type: "box".to_string(),
        radius: None,
        half_extents: Some(Vec3Json {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }),
        offset: None,
    };
    assert_eq!(box_shape.shape_type(), "box");
}

// ---------------------------------------------------------------------------
// CollisionLayers constants
// ---------------------------------------------------------------------------

#[test]
fn test_collision_layers_ship() {
    let ship = crate::collision::layers::SHIP;
    assert_eq!(ship.layers, crate::collision::layers::SHIP_LAYER);
    assert_eq!(ship.mask, crate::collision::layers::ASTEROID_LAYER);
}

#[test]
fn test_collision_layers_asteroid() {
    let asteroid = crate::collision::layers::ASTEROID;
    assert_eq!(asteroid.layers, crate::collision::layers::ASTEROID_LAYER);
    assert_eq!(asteroid.mask, crate::collision::layers::SHIP_LAYER);
}

#[test]
fn test_collision_layers_no_self_collision() {
    // Ship layer should not collide with itself (mask is ASTEROID_LAYER only)
    let ship = crate::collision::layers::SHIP;
    assert_eq!(
        ship.layers & ship.mask,
        0,
        "ship should not collide with itself"
    );

    // Asteroid layer should not collide with itself
    let asteroid = crate::collision::layers::ASTEROID;
    assert_eq!(
        asteroid.layers & asteroid.mask,
        0,
        "asteroid should not collide with itself"
    );
}

// ---------------------------------------------------------------------------
// CollisionShapeData constructors
// ---------------------------------------------------------------------------

#[test]
fn test_collision_shape_data_sphere() {
    let data = crate::collision::CollisionShapeData::sphere(3.0, Vec3::new(1.0, 0.0, 0.0));
    match data.shape_type {
        CollisionShapeType::Sphere { radius } => assert!((radius - 3.0).abs() < 0.01),
        other => panic!("expected Sphere, got {other:?}"),
    }
    assert_eq!(data.offset, Vec3::new(1.0, 0.0, 0.0));
}

#[test]
fn test_collision_shape_data_box() {
    let he = Vec3::new(2.0, 3.0, 4.0);
    let data = crate::collision::CollisionShapeData::box_shape(he, Vec3::ZERO);
    match data.shape_type {
        CollisionShapeType::Box { half_extents } => assert_eq!(half_extents, he),
        other => panic!("expected Box, got {other:?}"),
    }
}

// Re-export for test use
use bevy::prelude::Vec3;
