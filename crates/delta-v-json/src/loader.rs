// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Core JSON loading, schema validation and default-fill utilities.
//!
//! This module is the single implementation of the pipeline that every
//! JSON-backed crate uses (ADR-0038):
//!
//! 1. Read a file from disk into a [`serde_json::Value`].
//! 2. Validate the value against a JSON Schema file (ADR-0012).
//! 3. Recursively fill schema `default` values for absent fields
//!    (ADR-0013 — no silent fallbacks via `Default` impls).
//!
//! Callers then deserialise the filled `Value` into their own types.
//!
//! ## `$ref` resolution
//!
//! The fill-defaults pass resolves local `$defs` references
//! (`"$ref": "#/$defs/<name>"`) so that defaults declared inside
//! `$defs` sub-schemas are applied. Only fragment-only, `$defs`-path
//! references are supported; external or complex refs are forbidden by
//! ADR-0012.

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

    let Value::Object(obj) = value else {
        return;
    };
    let Some(properties) = resolved.get("properties").and_then(Value::as_object) else {
        return;
    };
    for (key, prop_schema) in properties {
        if !obj.contains_key(key) {
            if let Some(default) = prop_schema.get("default") {
                obj.insert(key.clone(), default.clone());
            }
        } else if let Some(child) = obj.get_mut(key) {
            fill_defaults_recursive(child, prop_schema, root);
        }
    }
}

/// Resolves a local `$ref` of the form `"#/$defs/<name>"`.
///
/// If `schema` has a `"$ref"` key whose value starts with `"#/$defs/"`,
/// returns the referenced sub-schema from `root["$defs"][<name>]`.
/// If the reference cannot be resolved, returns `schema` unchanged
/// (which will produce a parse error downstream — the correct failure).
///
/// Only `#/$defs/<name>` references are supported. All other forms are
/// forbidden by ADR-0012 rule 8.
fn resolve_ref<'a>(schema: &'a Value, root: &'a Value) -> &'a Value {
    let Some(ref_str) = schema.get("$ref").and_then(Value::as_str) else {
        return schema;
    };
    let Some(def_name) = ref_str.strip_prefix("#/$defs/") else {
        // Non-local ref: return unchanged; downstream error is correct.
        return schema;
    };
    root.get("$defs")
        .and_then(|defs| defs.get(def_name))
        .unwrap_or(schema)
}
