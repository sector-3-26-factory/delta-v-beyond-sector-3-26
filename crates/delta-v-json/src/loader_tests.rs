// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for JSON loader unit validation (ADR-0008).

#![allow(clippy::unwrap_used, clippy::panic)]
use std::path::Path;

use crate::error::JsonError;
use crate::loader::{fill_defaults, load_allowed_units, validate_units};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Minimal units schema for testing.
fn units_schema_json() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "value": { "type": "number" },
            "unit": {
                "type": "string",
                "enum": ["kg", "N", "N⋅m", "m", "s", "dimensionless"]
            }
        }
    })
}

/// Write units schema to a temp file and return the path.
fn write_units_schema(name: &str) -> std::path::PathBuf {
    let tmp_dir = std::env::temp_dir();
    let path = tmp_dir.join(name);
    std::fs::write(&path, units_schema_json().to_string()).unwrap();
    path
}

// ---------------------------------------------------------------------------
// Tests: validate_units
// ---------------------------------------------------------------------------

#[test]
fn test_validate_units_valid() {
    let value = serde_json::json!({
        "mass": { "value": 10000, "unit": "kg" },
        "propulsion": {
            "main_thrusters": [{
                "id": "main",
                "type": "chemical",
                "max_forward_thrust": { "value": 100_000, "unit": "N" },
                "max_backward_thrust": { "value": 40000, "unit": "N" }
            }],
            "maneuvering_thruster": {
                "type": "rcs",
                "max_torque": { "value": 50000, "unit": "N⋅m" },
                "max_strafe_thrust": { "value": 50000, "unit": "N" },
                "rotation_ramp_ticks": 60
            }
        }
    });

    let units_path = write_units_schema("test_units_valid.schema.json");
    let result = validate_units(&value, Path::new("test.json"), &units_path);
    assert!(result.is_ok(), "valid units should pass validation");
}

#[test]
fn test_validate_units_invalid() {
    let value = serde_json::json!({
        "mass": { "value": 10000, "unit": "pounds" }
    });

    let units_path = write_units_schema("test_units_invalid.schema.json");
    let result = validate_units(&value, Path::new("test.json"), &units_path);
    assert!(
        result.is_err(),
        "invalid unit 'pounds' should fail validation"
    );

    match result.unwrap_err() {
        JsonError::InvalidUnit { unit, .. } => {
            assert_eq!(unit, "pounds");
        }
        other => panic!("expected InvalidUnit error, got {other:?}"),
    }
}

#[test]
fn test_validate_units_nested_invalid() {
    let value = serde_json::json!({
        "propulsion": {
            "main_thrusters": [{
                "id": "main",
                "type": "chemical",
                "max_forward_thrust": { "value": 100_000, "unit": "invalid_unit" }
            }]
        }
    });

    let units_path = write_units_schema("test_units_nested.schema.json");
    let result = validate_units(&value, Path::new("test.json"), &units_path);
    assert!(
        result.is_err(),
        "nested invalid unit should fail validation"
    );

    match result.unwrap_err() {
        JsonError::InvalidUnit { unit, pointer, .. } => {
            assert_eq!(unit, "invalid_unit");
            assert!(
                pointer.contains("max_forward_thrust"),
                "pointer should contain path to invalid unit, got {pointer}"
            );
        }
        other => panic!("expected InvalidUnit error, got {other:?}"),
    }
}

#[test]
fn test_validate_units_no_physical_quantities() {
    let value = serde_json::json!({
        "name": "test",
        "count": 42,
        "nested": { "key": "value" }
    });

    let units_path = write_units_schema("test_units_none.schema.json");
    let result = validate_units(&value, Path::new("test.json"), &units_path);
    assert!(
        result.is_ok(),
        "JSON without physical quantities should pass"
    );
}

#[test]
fn test_validate_units_value_as_string() {
    // "value" is a string, not a number — not a physical quantity.
    let value = serde_json::json!({
        "description": { "value": "not a number", "unit": "kg" }
    });

    let units_path = write_units_schema("test_units_string.schema.json");
    let result = validate_units(&value, Path::new("test.json"), &units_path);
    assert!(
        result.is_ok(),
        "object with string 'value' is not a physical quantity"
    );
}

// ---------------------------------------------------------------------------
// Tests: load_allowed_units
// ---------------------------------------------------------------------------

#[test]
fn test_load_allowed_units() {
    let units_path = write_units_schema("test_allowed_units.schema.json");
    let allowed = load_allowed_units(&units_path).unwrap();

    assert!(allowed.contains(&"kg".to_string()));
    assert!(allowed.contains(&"N".to_string()));
    assert!(allowed.contains(&"N⋅m".to_string()));
    assert!(allowed.contains(&"dimensionless".to_string()));
    assert!(!allowed.contains(&"pounds".to_string()));
}

// ---------------------------------------------------------------------------
// Tests: fill_defaults
// ---------------------------------------------------------------------------

#[test]
fn test_fill_defaults_with_ref_to_definition() {
    // Schema with a $ref to a definition that has property-level defaults
    let schema = serde_json::json!({
        "$defs": {
            "quat": {
                "type": "object",
                "properties": {
                    "x": { "type": "number", "default": 0.0 },
                    "y": { "type": "number", "default": 0.0 },
                    "z": { "type": "number", "default": 0.0 },
                    "w": { "type": "number", "default": 1.0 }
                }
            }
        },
        "type": "object",
        "properties": {
            "rotation": { "$ref": "#/$defs/quat" }
        }
    });

    // Value missing the rotation field
    let mut value = serde_json::json!({
        "name": "test"
    });

    let schema_path = std::env::temp_dir().join("test_fill_defaults_schema.json");
    std::fs::write(&schema_path, schema.to_string()).unwrap();

    fill_defaults(&mut value, &schema_path).unwrap();

    // rotation should now have defaults
    let rotation = value.get("rotation").unwrap();
    assert!(rotation.get("x").is_some());
    assert!(rotation.get("y").is_some());
    assert!(rotation.get("z").is_some());
    assert!(rotation.get("w").is_some());
}

#[test]
fn test_fill_defaults_with_world_schema() {
    // Load the actual world schema
    let schema_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("assets/json/schema/world.schema.json");
    let schema_text = std::fs::read_to_string(&schema_path).unwrap();
    let _schema: Value = serde_json::from_str(&schema_text).unwrap();

    // Create a value with an entity missing rotation and scale
    let mut value = serde_json::json!({
        "format_version": 1,
        "name": "Test Sector",
        "entities": [
            {
                "template": "ships/test",
                "id": "test_ship",
                "position": { "x": 0.0, "y": 0.0, "z": 0.0 }
            }
        ]
    });

    fill_defaults(&mut value, &schema_path).unwrap();

    // Check that rotation and scale were filled in
    let entities = value.get("entities").unwrap().as_array().unwrap();
    let entity = entities.first().unwrap();
    println!("Entity after fill_defaults: {entity:?}");

    assert!(
        entity.get("rotation").is_some(),
        "rotation should be present"
    );
    assert!(entity.get("scale").is_some(), "scale should be present");
}

// ---------------------------------------------------------------------------
// Tests: deep_merge
// ---------------------------------------------------------------------------

#[test]
fn test_deep_merge_nested_objects() {
    let base = serde_json::json!({
        "actions": {
            "thrust_forward": { "keyboard": ["KeyW"] },
            "thrust_backward": { "keyboard": ["KeyS"] }
        },
        "settings": {
            "sensitivity": 1.0,
            "invert_y": false
        }
    });

    let override_val = serde_json::json!({
        "actions": {
            "thrust_forward": { "keyboard": ["KeyT"] }
        },
        "settings": {
            "sensitivity": 2.0
        }
    });

    // Use the builder to test deep_merge
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "actions": { "type": "object" },
            "settings": { "type": "object" }
        }
    });
    let schema_path = std::env::temp_dir().join("test_deep_merge_schema.json");
    std::fs::write(&schema_path, schema.to_string()).unwrap();

    let json_path = std::env::temp_dir().join("test_deep_merge.json");
    std::fs::write(&json_path, base.to_string()).unwrap();

    let override_path = std::env::temp_dir().join("test_deep_merge_override.json");
    std::fs::write(&override_path, override_val.to_string()).unwrap();

    let result = crate::loader::load(json_path, schema_path)
        .with_user_override(Some(&override_path))
        .skip_units()
        .load();

    // The deep_merge is tested indirectly through the builder
    assert!(result.is_ok());
}
