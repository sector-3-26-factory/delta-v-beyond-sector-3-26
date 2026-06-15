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
//! All schemas in `assets/json/schema/` are auto-discovered and registered
//! by their `$id` at startup, so any cross-schema `$ref` is automatically
//! resolvable without manual registration.

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;

use crate::error::JsonError;

// ---------------------------------------------------------------------------
// Schema registry — auto-discovers all schemas in assets/json/schema/
// ---------------------------------------------------------------------------

/// Builds a `HashMap` containing every `.schema.json` file found in `schema_dir`.
/// Each schema is keyed by its `$id` URI for direct lookup.
///
/// Schemas without an `$id` field or with invalid JSON are silently skipped.
///
/// # Errors
/// Returns [`JsonError::SchemaLoad`] if the schema directory cannot be read.
fn build_schema_map(schema_dir: &Path) -> Result<HashMap<String, Value>, JsonError> {
    let mut map = HashMap::new();

    let entries = std::fs::read_dir(schema_dir).map_err(|e| JsonError::SchemaLoad {
        path: schema_dir.to_owned(),
        reason: format!("cannot read schema directory: {e}"),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| JsonError::SchemaLoad {
            path: schema_dir.to_owned(),
            reason: format!("cannot read directory entry: {e}"),
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !file_name.ends_with(".schema.json") {
            continue;
        }

        // Skip files that can't be read or parsed - they're not valid schemas
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(schema): Result<Value, _> = serde_json::from_str(&text) else {
            continue;
        };

        // Skip schemas without $id - they can't be referenced cross-schema
        if let Some(id) = schema.get("$id").and_then(Value::as_str) {
            map.insert(id.to_string(), schema);
        }
    }

    Ok(map)
}

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
/// This version auto-discovers all `.schema.json` files in `schema_dir` and registers
/// them by their `$id` into a `referencing::Registry`. This allows any cross-schema
/// `$ref` (e.g., `"$ref": "https://delta-v-beyond-sector-3-26/schema/units#/$defs/mass"`)
/// to resolve automatically without manual per-schema registration.
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
    schema_dir: &Path,
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

    // Build schema map from all schemas in the schema directory
    let schema_map = build_schema_map(schema_dir)?;

    // Build a jsonschema Registry from the map for validation
    let mut registry = jsonschema::Registry::new();
    for (id, schema) in &schema_map {
        registry = registry
            .add(id, schema.clone())
            .map_err(|e| JsonError::SchemaLoad {
                path: schema_path.to_owned(),
                reason: e.to_string(),
            })?;
    }
    let registry = registry.prepare().map_err(|e| JsonError::SchemaLoad {
        path: schema_path.to_owned(),
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
/// for any absent fields, with cross-schema `$ref` support via a schema map.
///
/// This is the authoritative default-fill mechanism (ADR-0013). Serde
/// types loaded via this pipeline must **not** implement `Default` as a
/// fallback — if a required field is absent after this pass,
/// `serde_json::from_value` will return a parse error, which is the
/// correct hard failure.
///
/// Cross-schema `$ref`s (e.g., `"$ref": "https://delta-v-beyond-sector-3-26/schema/cockpit"`)
/// are resolved using the schema directory so that defaults declared in
/// external schemas are applied.
///
/// # Errors
/// Returns [`JsonError::SchemaLoad`] if the schema file cannot be read
/// or parsed.
pub fn fill_defaults(value: &mut Value, schema_path: &Path) -> Result<(), JsonError> {
    let schema_dir = schema_path.parent().unwrap_or_else(|| Path::new(""));
    let schema_text = std::fs::read_to_string(schema_path).map_err(|e| JsonError::SchemaLoad {
        path: schema_path.to_owned(),
        reason: e.to_string(),
    })?;
    let schema: Value = serde_json::from_str(&schema_text).map_err(|e| JsonError::SchemaLoad {
        path: schema_path.to_owned(),
        reason: e.to_string(),
    })?;

    // Build schema map for cross-schema $ref resolution
    let schema_map = build_schema_map(schema_dir)?;

    // Fill defaults using the schema map
    fill_defaults_recursive(value, &schema, &schema, &schema_map);
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

/// Full pipeline: read, validate with cross-schema registry, fill defaults,
/// and validate units.
///
/// This version uses `validate_with_registry` with auto-discovered schemas
/// to properly resolve cross-schema `$ref`s.
///
/// Per ADR-0008, all numeric physical quantities MUST use the
/// `{"value": N, "unit": "..."}` format with a unit from the registry.
///
/// # Errors
/// Returns [`JsonError`] if any step fails, including
/// [`JsonError::InvalidUnit`] if a unit is not in the registry.
pub fn load_validated_with_registry(
    json_path: &Path,
    schema_path: &Path,
) -> Result<Value, JsonError> {
    // Compute schema_dir from schema_path
    let schema_dir = schema_path.parent().unwrap_or_else(|| Path::new(""));

    // Read the JSON file
    let mut value = read_json(json_path)?;

    // Validate with registry to resolve cross-schema $refs
    validate_with_registry(&value, schema_path, json_path, schema_dir)?;

    // Fill defaults with schema map to resolve cross-schema $refs
    fill_defaults(&mut value, schema_path)?;

    // Validate units against the allowed units list
    // The units schema is always in the same directory as the schema
    let units_schema_path = schema_dir.join("units.schema.json");
    validate_units(&value, json_path, &units_schema_path)?;

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

/// Recursive default-fill pass with cross-schema $ref support.
///
/// `root` is the main schema document for local `$defs` resolution.
/// `schema_map` is used for cross-schema `$ref` resolution.
fn fill_defaults_recursive(
    value: &mut Value,
    schema: &Value,
    root: &Value,
    schema_map: &HashMap<String, Value>,
) {
    // Resolve $ref before processing.
    let resolved = resolve_ref(schema, root, schema_map);

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
                        let resolved_prop = resolve_ref(prop_schema, root, schema_map);
                        if let Some(prop_defaults) = get_property_defaults(resolved_prop) {
                            obj.insert(key.clone(), Value::Object(prop_defaults));
                        }
                    }
                } else if let Some(child) = obj.get_mut(key) {
                    fill_defaults_recursive(child, prop_schema, root, schema_map);
                }
            }
        }
        Value::Array(arr) => {
            // For arrays, we need to find the schema for array items.
            if let Some(items_schema) = resolved.get("items") {
                for item in arr.iter_mut() {
                    fill_defaults_recursive(item, items_schema, root, schema_map);
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

/// Resolves a `$ref`, supporting both local and cross-schema references.
///
/// Local refs (`"#/$defs/<name>"`) are resolved from the root schema.
/// Cross-schema refs (URIs like `"https://delta-v-beyond-sector-3-26/schema/cockpit"`)
/// are resolved from the schema map.
fn resolve_ref<'a>(
    schema: &'a Value,
    root: &'a Value,
    schema_map: &'a HashMap<String, Value>,
) -> &'a Value {
    let Some(ref_str) = schema.get("$ref").and_then(Value::as_str) else {
        return schema;
    };

    // Try local ref first
    if let Some(def_name) = ref_str.strip_prefix("#/$defs/") {
        return root
            .get("$defs")
            .and_then(|defs| defs.get(def_name))
            .unwrap_or(schema);
    }

    // Try cross-schema ref via schema map
    schema_map.get(ref_str).unwrap_or(schema)
}
