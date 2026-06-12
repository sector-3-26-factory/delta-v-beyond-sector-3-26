#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for `fill_defaults` with array items schema.

use crate::loader::fill_defaults;

/// Creates a temporary file with the given content and returns its path.
fn write_temp_file(name: &str, content: &str) -> std::path::PathBuf {
    let tmp_dir = std::env::temp_dir();
    let path = tmp_dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn test_fill_defaults_with_array_items() {
    // Schema where array items have defaults
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "entities": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "enabled": { "type": "boolean", "default": true },
                        "count": { "type": "number", "default": 0 }
                    }
                }
            }
        }
    });

    let schema_path = write_temp_file("test_array_schema.json", &schema.to_string());

    // Value with array items missing defaults
    let mut value = serde_json::json!({
        "entities": [
            { "name": "entity1" },
            { "name": "entity2", "enabled": false }
        ]
    });

    fill_defaults(&mut value, &schema_path).unwrap();

    let entities = value["entities"].as_array().unwrap();

    // First entity should have defaults filled
    assert_eq!(entities[0]["name"], "entity1");
    assert_eq!(
        entities[0]["enabled"], true,
        "default should be filled for enabled"
    );
    assert_eq!(
        entities[0]["count"], 0,
        "default should be filled for count"
    );

    // Second entity should keep its explicit value but get missing defaults
    assert_eq!(entities[1]["name"], "entity2");
    assert_eq!(
        entities[1]["enabled"], false,
        "explicit value should be preserved"
    );
    assert_eq!(
        entities[1]["count"], 0,
        "default should be filled for count"
    );
}

#[test]
fn test_fill_defaults_nested_array_in_object() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "config": {
                "type": "object",
                "properties": {
                    "items": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "active": { "type": "boolean", "default": true }
                            }
                        }
                    }
                }
            }
        }
    });

    let schema_path = write_temp_file("test_nested_array_schema.json", &schema.to_string());

    let mut value = serde_json::json!({
        "config": {
            "items": [
                { "id": "a" },
                { "id": "b", "active": false }
            ]
        }
    });

    fill_defaults(&mut value, &schema_path).unwrap();

    let items = value["config"]["items"].as_array().unwrap();
    assert_eq!(items[0]["active"], true, "default should be filled");
    assert_eq!(
        items[1]["active"], false,
        "explicit value should be preserved"
    );
}

#[test]
fn test_fill_defaults_empty_array() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "value": { "type": "number", "default": 42 }
                    }
                }
            }
        }
    });

    let schema_path = write_temp_file("test_empty_array_schema.json", &schema.to_string());

    let mut value = serde_json::json!({
        "items": []
    });

    fill_defaults(&mut value, &schema_path).unwrap();

    // Empty array should remain empty
    let items = value["items"].as_array().unwrap();
    assert!(items.is_empty(), "empty array should remain empty");
}
