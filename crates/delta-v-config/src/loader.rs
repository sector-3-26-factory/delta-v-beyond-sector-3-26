// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Configuration file loader: reads, validates and merges JSON config files.
//!
//! Implements the two-layer config system described in ADR-0010:
//! 1. Default layer: `assets/config/<file>.json` (shipped with the game).
//! 2. User layer: `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/<file>.json`
//!    (optional user overrides).
//!
//! Both layers are validated against their JSON Schema (ADR-0012).
//! Missing defaults file is a hard error (ADR-0013).

use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde_json::Value;

use crate::{error::ConfigError, keybindings::Keybindings};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Loads, validates and merges keybindings configuration.
///
/// Steps (per ADR-0010, ADR-0012, ADR-0013):
/// 1. Read `assets/config/keybindings.json` (hard error if missing).
/// 2. Validate against `assets/json/schema/keybindings.schema.json`.
/// 3. Fill schema `default` values for omitted optional fields.
/// 4. If a user override exists at the XDG path, deep-merge and re-validate.
/// 5. Deserialise into [`Keybindings`].
///
/// # Errors
/// Returns [`ConfigError`] if any step fails. All errors include the
/// file path and a precise description (ADR-0016).
pub fn load_keybindings() -> Result<Keybindings, ConfigError> {
    let defaults_path = PathBuf::from("assets/config/keybindings.json");
    let schema_path = PathBuf::from("assets/json/schema/keybindings.schema.json");

    // 1 + 2 + 3: load, validate, fill defaults.
    let mut merged = load_and_validate(&defaults_path, &schema_path)?;

    // 4: try user override.
    if let Some(user_path) = user_keybindings_path() {
        if user_path.exists() {
            let user_value = read_json(&user_path)?;
            deep_merge(&mut merged, user_value);
            validate_against_schema(&merged, &schema_path, &user_path)?;
        }
    }

    // 5: deserialise.
    serde_json::from_value(merged).map_err(|e| ConfigError::Parse {
        path: defaults_path,
        source: e,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Public entry point used by tests and the hot-reload path.
///
/// Reads `json_path`, validates against `schema_path`, fills defaults,
/// and returns the resulting [`Value`].
///
/// # Errors
/// Returns [`ConfigError`] if the file cannot be read, parsed, or
/// validated.
pub fn load_and_validate_from_paths(
    json_path: &Path,
    schema_path: &Path,
) -> Result<Value, ConfigError> {
    load_and_validate(json_path, schema_path)
}

/// Public wrapper around [`deep_merge`] used by tests.
pub fn merge_user_override(dst: &mut Value, src: Value) {
    deep_merge(dst, src);
}

/// Reads a JSON file, validates it against a schema, fills in schema
/// defaults, and returns the resulting [`Value`].
fn load_and_validate(json_path: &Path, schema_path: &Path) -> Result<Value, ConfigError> {
    let mut value = read_json(json_path)?;
    validate_against_schema(&value, schema_path, json_path)?;
    fill_defaults(&mut value, schema_path)?;
    Ok(value)
}

/// Reads and parses a JSON file.
fn read_json(path: &Path) -> Result<Value, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.to_owned(),
        source: e,
    })?;
    serde_json::from_str(&text).map_err(|e| ConfigError::Parse {
        path: path.to_owned(),
        source: e,
    })
}

/// Validates a JSON value against a schema file.
///
/// Returns `Ok(())` when the value is valid.
/// Returns [`ConfigError::Schema`] with a precise pointer and reason on
/// the first validation error.
fn validate_against_schema(
    value: &Value,
    schema_path: &Path,
    data_path: &Path,
) -> Result<(), ConfigError> {
    let schema_text =
        std::fs::read_to_string(schema_path).map_err(|e| ConfigError::SchemaLoad {
            path: schema_path.to_owned(),
            reason: e.to_string(),
        })?;
    let schema_value: Value =
        serde_json::from_str(&schema_text).map_err(|e| ConfigError::SchemaLoad {
            path: schema_path.to_owned(),
            reason: e.to_string(),
        })?;

    let validator =
        jsonschema::validator_for(&schema_value).map_err(|e| ConfigError::SchemaLoad {
            path: schema_path.to_owned(),
            reason: e.to_string(),
        })?;

    // Collect first error only (sufficient for a hard-error startup failure).
    let errors: Vec<_> = validator.iter_errors(value).collect();
    if let Some(err) = errors.into_iter().next() {
        return Err(ConfigError::Schema {
            path: data_path.to_owned(),
            pointer: err.instance_path().to_string(),
            reason: err.to_string(),
        });
    }

    Ok(())
}

/// Recursively fills in schema `default` values for fields that are absent
/// from `value`.
///
/// This is a structural walk over the schema's `properties` map. It does
/// not evaluate `oneOf`/`anyOf`/`if-then-else` combinators, which are
/// forbidden by ADR-0012 rule 8.
fn fill_defaults(value: &mut Value, schema_path: &Path) -> Result<(), ConfigError> {
    let schema_text =
        std::fs::read_to_string(schema_path).map_err(|e| ConfigError::SchemaLoad {
            path: schema_path.to_owned(),
            reason: e.to_string(),
        })?;
    let schema: Value =
        serde_json::from_str(&schema_text).map_err(|e| ConfigError::SchemaLoad {
            path: schema_path.to_owned(),
            reason: e.to_string(),
        })?;

    fill_defaults_recursive(value, &schema);
    Ok(())
}

/// Recursive default-fill pass.
///
/// For each property in the schema that has a `default` and is absent in
/// `value`, inserts the default. Then recurses into present object
/// properties.
fn fill_defaults_recursive(value: &mut Value, schema: &Value) {
    let Value::Object(obj) = value else {
        return;
    };
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return;
    };
    for (key, prop_schema) in properties {
        if !obj.contains_key(key) {
            if let Some(default) = prop_schema.get("default") {
                obj.insert(key.clone(), default.clone());
            }
        } else if let Some(child) = obj.get_mut(key) {
            fill_defaults_recursive(child, prop_schema);
        }
    }
}

/// Deep-merges `src` on top of `dst`.
///
/// Rules (ADR-0010):
/// - Objects: merged recursively; `src` keys override `dst` keys.
/// - Arrays, scalars, null: `src` replaces `dst` wholesale.
fn deep_merge(dst: &mut Value, src: Value) {
    match (dst, src) {
        (Value::Object(dst_map), Value::Object(src_map)) => {
            for (k, v) in src_map {
                let entry = dst_map.entry(k).or_insert(Value::Null);
                deep_merge(entry, v);
            }
        }
        (dst, src) => *dst = src,
    }
}

/// Returns the platform-appropriate user keybindings override path, if
/// the base directory can be determined.
///
/// On Linux: `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/keybindings.json`
/// (or `~/.config/...` if `XDG_CONFIG_HOME` is unset). See ADR-0010.
fn user_keybindings_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "delta-v", "delta-v-beyond-sector-3-26")
        .map(|dirs| dirs.config_dir().join("keybindings.json"))
}
