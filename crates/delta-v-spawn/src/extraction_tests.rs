// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for template field extraction helpers.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp)]

use serde_json::Value;

use delta_v_types::{BoundingBoxJson, Vec3Json};

use crate::template_extraction::{
    compute_debug_axis_length, extract_bounding_box, extract_mass, extract_vec3, resolve_mass,
    scale_bounding_box,
};

// ---------------------------------------------------------------------------
// extract_mass
// ---------------------------------------------------------------------------

#[test]
fn test_extract_mass_basic() {
    let template = serde_json::json!({
        "mass": { "value": 10_000.0, "unit": "kg" }
    });
    let mass = extract_mass(&template);
    assert!(
        (mass - 10_000.0).abs() < 0.01,
        "mass should be 10000, got {mass}"
    );
}

#[test]
fn test_extract_mass_small_value() {
    let template = serde_json::json!({
        "mass": { "value": 0.5, "unit": "kg" }
    });
    let mass = extract_mass(&template);
    assert!((mass - 0.5).abs() < 0.01, "mass should be 0.5, got {mass}");
}

#[test]
#[should_panic(expected = "mass.value should be present and valid per schema")]
fn test_extract_mass_missing_field() {
    let template = serde_json::json!({ "name": "test" });
    let _ = extract_mass(&template);
}

#[test]
#[should_panic(expected = "mass.value should be present and valid per schema")]
fn test_extract_mass_missing_value() {
    let template = serde_json::json!({ "mass": { "unit": "kg" } });
    let _ = extract_mass(&template);
}

// ---------------------------------------------------------------------------
// extract_vec3
// ---------------------------------------------------------------------------

#[test]
fn test_extract_vec3_basic() {
    let mut json = serde_json::Map::new();
    json.insert("x".to_string(), Value::from(1.0));
    json.insert("y".to_string(), Value::from(2.0));
    json.insert("z".to_string(), Value::from(3.0));

    let v = extract_vec3(&json);
    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
    assert_eq!(v.z, 3.0);
}

#[test]
fn test_extract_vec3_negative() {
    let mut json = serde_json::Map::new();
    json.insert("x".to_string(), Value::from(-1.5));
    json.insert("y".to_string(), Value::from(0.0));
    json.insert("z".to_string(), Value::from(4.2));

    let v = extract_vec3(&json);
    assert_eq!(v.x, -1.5);
    assert_eq!(v.y, 0.0);
    assert_eq!(v.z, 4.2);
}

#[test]
#[should_panic(expected = "x should be present and valid")]
fn test_extract_vec3_missing_x() {
    let mut json = serde_json::Map::new();
    json.insert("y".to_string(), Value::from(2.0));
    json.insert("z".to_string(), Value::from(3.0));
    let _ = extract_vec3(&json);
}

// ---------------------------------------------------------------------------
// extract_bounding_box
// ---------------------------------------------------------------------------

#[test]
fn test_extract_bounding_box_basic() {
    let template = serde_json::json!({
        "bounding_box": {
            "min": { "x": -1.0, "y": -2.0, "z": -3.0 },
            "max": { "x": 1.0, "y": 2.0, "z": 3.0 }
        }
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
    let template = serde_json::json!({
        "bounding_box": {
            "min": { "x": 0.0, "y": 0.0, "z": 0.0 },
            "max": { "x": 10.0, "y": 5.0, "z": 2.0 }
        }
    });

    let bbox = extract_bounding_box(&template);
    assert!((bbox.min.x - 0.0).abs() < 0.01);
    assert!((bbox.min.y - 0.0).abs() < 0.01);
    assert!((bbox.min.z - 0.0).abs() < 0.01);
    assert!((bbox.max.x - 10.0).abs() < 0.01);
    assert!((bbox.max.y - 5.0).abs() < 0.01);
    assert!((bbox.max.z - 2.0).abs() < 0.01);
}

#[test]
#[should_panic(expected = "bounding_box should be present per schema")]
fn test_extract_bounding_box_missing() {
    let template = serde_json::json!({ "name": "test" });
    let _ = extract_bounding_box(&template);
}

// ---------------------------------------------------------------------------
// compute_debug_axis_length
// ---------------------------------------------------------------------------

#[test]
fn test_compute_debug_axis_length_cube() {
    let bbox = BoundingBoxJson {
        min: Vec3Json {
            x: -1.0,
            y: -1.0,
            z: -1.0,
        },
        max: Vec3Json {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
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
    let bbox = BoundingBoxJson {
        min: Vec3Json {
            x: -5.0,
            y: -1.0,
            z: -2.0,
        },
        max: Vec3Json {
            x: 5.0,
            y: 1.0,
            z: 2.0,
        },
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
    let bbox = BoundingBoxJson {
        min: Vec3Json {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        max: Vec3Json {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
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
    let bbox = BoundingBoxJson {
        min: Vec3Json {
            x: -10.0,
            y: -10.0,
            z: -10.0,
        },
        max: Vec3Json {
            x: 10.0,
            y: 10.0,
            z: 10.0,
        },
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
    let bbox = BoundingBoxJson {
        min: Vec3Json {
            x: -100.0,
            y: -100.0,
            z: -100.0,
        },
        max: Vec3Json {
            x: 100.0,
            y: 100.0,
            z: 100.0,
        },
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
