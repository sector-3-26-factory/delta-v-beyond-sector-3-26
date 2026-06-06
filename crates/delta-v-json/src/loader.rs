// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Core JSON loading, schema validation and default-fill utilities.
//!
//! This module is the single implementation of the JSON pipeline
//! used by all JSON-backed crates (ADR-0038):
//!
//! 1. Read a file from disk into a [`serde_json::Value`].
//! 2. Validate the value against a JSON Schema file (ADR-0012).
//! 3. Recursively fill schema `default` values for absent fields
//!    (ADR-0013 — no silent fallbacks via `Default` impls).
//! 4. Optionally validate physical quantity units against the central
//!    units registry (ADR-0008).
//!
//! Callers then deserialise the filled `Value` into their own types.
//!
//! ## `$ref` resolution
//!
//! The fill-defaults pass resolves local `$defs` references
//! (`"$ref": "#/$defs/<name>"`) so that defaults declared inside
//! `$defs` sub-schemas are applied.
//!
//! Cross-schema references use the `$id` URI scheme:
//! `"$ref": "https://delta-v-beyond-sector-3-26/schema/units#/$defs/length"`.
//! The validator uses a `referencing::Registry` to resolve these in-memory.

use std::path::Path;

use serde_json::Value;

use crate::error::JsonError;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Reads a JSON file from disk.
///
/// # Errors
/// Returns [`JsonError::Io`] if the file cannot be opened or read.
/// Returns [`JsonError::Parse`] if the content is not valid JSON.
pub fn read_json(path: &Path) -> Result<Value, JsonError> {
    let text = std::fs::read_to_string(path).map_err(|e| JsonError::Io {
        path: path.to_owned(),
        source: e,
    })?;
    serde_json::from_str(&text).map_err(|e| JsonError::Parse {
        path: path.to_owned(),
        source: e,
    })
}

/// Validates `value` against the JSON Schema at `schema_path`.
///
/// Returns the **first** validation error as [`JsonError::Schema`].
///
/// # Errors
/// Returns [`JsonError::SchemaLoad`] if the schema cannot be loaded.
/// Returns [`JsonError::Schema`] if validation fails.
pub fn validate(value: &Value, schema_path: &Path, data_path: &Path) -> Result<(), JsonError> {
    let schema_text = std::fs::read_to_string(schema_path).map_err(|e| JsonError::SchemaLoad {
        path: schema_path.to_owned(),
        reason: e.to_string(),
    })?;
    let schema_value: Value =
        serde_json::from_str(&schema_text).map_err(|e| JsonError::SchemaLoad {
            path: schema_path.to_owned(),
            reason: e.to_string(),
        })?;
    let validator =
        jsonschema::validator_for(&schema_value).map_err(|e| JsonError::SchemaLoad {
            path: schema_path.to_owned(),
            reason: e.to_string(),
        })?;

    let errors: Vec<_> = validator.iter_errors(value).collect();
    if let Some(err) = errors.into_iter().next() {
        return Err(JsonError::Schema {
            path: data_path.to_owned(),
            pointer: err.instance_path().to_string(),
            reason: err.to_string(),
        });
    }
    Ok(())
}

/// Validates `value` against the JSON Schema at `schema_path` with cross-schema support.
///
/// This version uses a `referencing::Registry` to resolve cross-schema `$ref`s
/// by their `$id` URIs. The units schema is automatically registered.
///
/// Returns the **first** validation error as [`JsonError::Schema`].
///
/// # Errors
/// Returns [`JsonError::SchemaLoad`] if the schema cannot be loaded.
/// Returns [`JsonError::Schema`] if validation fails.
pub fn validate_with_registry(
    value: &Value,
    schema_path: &Path,
    data_path: &Path,
    units_schema_path: &Path,
) -> Result<(), JsonError> {
    // Load the main schema
    let schema_text = std::fs::read_to_string(schema_path).map_err(|e| JsonError::SchemaLoad {
        path: schema_path.to_owned(),
        reason: e.to_string(),
    })?;
    let schema_value: Value =
        serde_json::from_str(&schema_text).map_err(|e| JsonError::SchemaLoad {
            path: schema_path.to_owned(),
            reason: e.to_string(),
        })?;

    // Load the units schema and register it
    let units_text =
        std::fs::read_to_string(units_schema_path).map_err(|e| JsonError::SchemaLoad {
            path: units_schema_path.to_owned(),
            reason: e.to_string(),
        })?;
    let units_value: Value =
        serde_json::from_str(&units_text).map_err(|e| JsonError::SchemaLoad {
            path: units_schema_path.to_owned(),
            reason: e.to_string(),
        })?;

    // Build a registry with the units schema
    let units_id = units_value
        .get("$id")
        .and_then(Value::as_str)
        .ok_or_else(|| JsonError::SchemaLoad {
            path: units_schema_path.to_owned(),
            reason: "units schema missing $id".to_string(),
        })?
        .to_string();
    let registry = jsonschema::Registry::new()
        .add(&units_id, units_value)
        .map_err(|e| JsonError::SchemaLoad {
            path: units_schema_path.to_owned(),
            reason: e.to_string(),
        })?
        .prepare()
        .map_err(|e| JsonError::SchemaLoad {
            path: units_schema_path.to_owned(),
            reason: e.to_string(),
        })?;

    // Build validator with the registry for cross-schema $ref resolution
    let validator = jsonschema::options()
        .with_registry(&registry)
        .build(&schema_value)
        .map_err(|e| JsonError::SchemaLoad {
            path: schema_path.to_owned(),
            reason: e.to_string(),
        })?;

    let errors: Vec<_> = validator.iter_errors(value).collect();
    if let Some(err) = errors.into_iter().next() {
        return Err(JsonError::Schema {
            path: data_path.to_owned(),
            pointer: err.instance_path().to_string(),
            reason: err.to_string(),
        });
    }
    Ok(())
}

/// Reads a schema file and recursively fills `default` values into `value`
/// for any absent fields.
///
/// This is the authoritative default-fill mechanism (ADR-0013). Serde
/// types loaded via this pipeline must **not** implement `Default` as a
/// fallback — if a required field is absent after this pass,
/// `serde_json::from_value` will return a parse error, which is the
/// correct hard failure.
///
/// Local `$defs` `$ref`s (`"$ref": "#/$defs/<name>"`) are resolved so
/// that defaults declared inside `$defs` sub-schemas are applied.
///
/// # Errors
/// Returns [`JsonError::SchemaLoad`] if the schema file cannot be read
/// or parsed.
pub fn fill_defaults(value: &mut Value, schema_path: &Path) -> Result<(), JsonError> {
    let schema_text = std::fs::read_to_string(schema_path).map_err(|e| JsonError::SchemaLoad {
        path: schema_path.to_owned(),
        reason: e.to_string(),
    })?;
    let schema: Value = serde_json::from_str(&schema_text).map_err(|e| JsonError::SchemaLoad {
        path: schema_path.to_owned(),
        reason: e.to_string(),
    })?;
    // Pass the root document so $ref resolution can look up $defs.
    fill_defaults_recursive(value, &schema, &schema);
    Ok(())
}

/// Convenience: read, validate and fill defaults in one call.
///
/// # Errors
/// Returns [`JsonError`] if any step fails.
pub fn load_validated(json_path: &Path, schema_path: &Path) -> Result<Value, JsonError> {
    let mut value = read_json(json_path)?;
    validate(&value, schema_path, json_path)?;
    fill_defaults(&mut value, schema_path)?;
    Ok(value)
}

/// Full pipeline: read, validate, fill defaults, and validate units.
///
/// This version uses `validate_with_registry` to properly resolve cross-schema
/// `$ref`s (e.g., `"$ref": "https://delta-v-beyond-sector-3-26/schema/units#/$defs/mass"`).
///
/// Per ADR-0008, all numeric physical quantities MUST use the
/// `{"value": N, "unit": "..."}` format with a unit from the registry.
///
/// # Errors
/// Returns [`JsonError`] if any step fails, including
/// [`JsonError::InvalidUnit`] if a unit is not in the registry.
pub fn load_validated_with_units(
    json_path: &Path,
    schema_path: &Path,
    units_schema_path: &Path,
) -> Result<Value, JsonError> {
    // Read the JSON file
    let mut value = read_json(json_path)?;

    // Validate with registry to resolve cross-schema $refs
    validate_with_registry(&value, schema_path, json_path, units_schema_path)?;

    // Fill defaults
    fill_defaults(&mut value, schema_path)?;

    // Validate units against the allowed units list
    validate_units(&value, json_path, units_schema_path)?;

    Ok(value)
}

// ---------------------------------------------------------------------------
// Unit validation (ADR-0008)
// ---------------------------------------------------------------------------

/// Recursively walks `value` and validates every `{"value": N, "unit": "..."}`
/// object against the allowed units from the units schema.
///
/// A physical quantity object is identified by having both `"value"` (number)
/// and `"unit"` (string) keys. The unit string is checked against the
/// `"enum"` list in the units schema's `properties.unit`.
///
/// # Errors
/// Returns [`JsonError::InvalidUnit`] if a unit is not in the registry.
/// Returns [`JsonError::SchemaLoad`] if the units schema cannot be read.
pub(crate) fn validate_units(
    value: &Value,
    data_path: &Path,
    units_schema_path: &Path,
) -> Result<(), JsonError> {
    let allowed_units = load_allowed_units(units_schema_path)?;
    validate_units_recursive(
        value,
        data_path,
        units_schema_path,
        &allowed_units,
        String::new(),
    )
}

/// Loads the allowed unit strings from the units schema file.
///
/// Reads the `"enum"` array from `properties.unit` in the units schema.
pub(crate) fn load_allowed_units(units_schema_path: &Path) -> Result<Vec<String>, JsonError> {
    let schema_text =
        std::fs::read_to_string(units_schema_path).map_err(|e| JsonError::SchemaLoad {
            path: units_schema_path.to_owned(),
            reason: e.to_string(),
        })?;
    let schema: Value = serde_json::from_str(&schema_text).map_err(|e| JsonError::SchemaLoad {
        path: units_schema_path.to_owned(),
        reason: e.to_string(),
    })?;

    let allowed = schema
        .get("properties")
        .and_then(|p| p.get("unit"))
        .and_then(|u| u.get("enum"))
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(allowed)
}

/// Recursively walks the JSON tree and validates physical quantity units.
///
/// Returns the **first** invalid unit found.
fn validate_units_recursive(
    value: &Value,
    data_path: &Path,
    units_schema_path: &Path,
    allowed_units: &[String],
    pointer: String,
) -> Result<(), JsonError> {
    match value {
        Value::Object(map) => {
            // Check if this object is a physical quantity (has "value" as number and "unit" as string).
            if let (Some(Value::Number(_)), Some(Value::String(unit_str))) =
                (map.get("value"), map.get("unit"))
            {
                if !allowed_units.contains(unit_str) {
                    return Err(JsonError::InvalidUnit {
                        path: data_path.to_owned(),
                        pointer,
                        unit: unit_str.clone(),
                        units_schema: units_schema_path.to_owned(),
                    });
                }
            }

            // Recurse into children.
            for (key, child) in map {
                let child_pointer = if pointer.is_empty() {
                    format!("/{key}")
                } else {
                    format!("{pointer}/{key}")
                };
                validate_units_recursive(
                    child,
                    data_path,
                    units_schema_path,
                    allowed_units,
                    child_pointer,
                )?;
            }
        }
        Value::Array(arr) => {
            // Recurse into array elements.
            for (index, element) in arr.iter().enumerate() {
                let child_pointer = format!("{pointer}[{index}]");
                validate_units_recursive(
                    element,
                    data_path,
                    units_schema_path,
                    allowed_units,
                    child_pointer,
                )?;
            }
        }
        _ => {}
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Recursive default-fill pass.
///
/// `root` is always the full schema document so that `$ref` strings of
/// the form `"#/$defs/<name>"` can be resolved.
fn fill_defaults_recursive(value: &mut Value, schema: &Value, root: &Value) {
    // Resolve $ref before processing.
    let resolved = resolve_ref(schema, root);

    match value {
        Value::Object(obj) => {
            let Some(properties) = resolved.get("properties").and_then(Value::as_object) else {
                return;
            };
            for (key, prop_schema) in properties {
                if !obj.contains_key(key) {
                    // First check for a top-level default on the property schema.
                    if let Some(default) = prop_schema.get("default") {
                        obj.insert(key.clone(), default.clone());
                    } else {
                        // If no top-level default, check if the property schema has a $ref
                        // that points to a definition with property-level defaults.
                        // In that case, create an object with those defaults.
                        let resolved_prop = resolve_ref(prop_schema, root);
                        if let Some(prop_defaults) = get_property_defaults(resolved_prop) {
                            obj.insert(key.clone(), Value::Object(prop_defaults));
                        }
                    }
                } else if let Some(child) = obj.get_mut(key) {
                    fill_defaults_recursive(child, prop_schema, root);
                }
            }
        }
        Value::Array(arr) => {
            // For arrays, we need to find the schema for array items.
            // The schema should have an "items" property that describes the items.
            if let Some(items_schema) = resolved.get("items") {
                for item in arr.iter_mut() {
                    fill_defaults_recursive(item, items_schema, root);
                }
            }
        }
        _ => {}
    }
}

/// Extracts default values for all properties from a schema.
///
/// Returns a `Map` of property names to their default values,
/// or `None` if the schema has no properties with defaults.
fn get_property_defaults(schema: &Value) -> Option<serde_json::Map<String, Value>> {
    let properties = schema.get("properties")?.as_object()?;
    let mut defaults = serde_json::Map::new();
    let mut has_defaults = false;

    for (prop_name, prop_def) in properties {
        if let Some(default) = prop_def.get("default") {
            defaults.insert(prop_name.clone(), default.clone());
            has_defaults = true;
        }
    }

    if has_defaults {
        Some(defaults)
    } else {
        None
    }
}

/// Resolves a local `$ref` of the form `"#/$defs/<name>"`.
///
/// Returns the referenced sub-schema from `root["$defs"][<name>]`.
/// If the reference cannot be resolved, returns `schema` unchanged
/// (which will produce a parse error downstream — the correct failure).
fn resolve_ref<'a>(schema: &'a Value, root: &'a Value) -> &'a Value {
    let Some(ref_str) = schema.get("$ref").and_then(Value::as_str) else {
        return schema;
    };

    // Only local refs are supported for default-filling.
    // Cross-schema refs are handled by the jsonschema library during validation.
    let Some(def_name) = ref_str.strip_prefix("#/$defs/") else {
        return schema;
    };

    root.get("$defs")
        .and_then(|defs| defs.get(def_name))
        .unwrap_or(schema)
}
