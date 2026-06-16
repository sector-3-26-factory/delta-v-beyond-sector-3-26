// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! JSON loading pipeline with builder pattern.
//!
//! This module provides the [`load`] function as the single entry point
//! for the JSON pipeline used by all JSON-backed crates (ADR-0038):
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
//!
//! ## Public API
//!
//! The public API consists of:
//! - [`load`] - entry point returning a [`JsonLoader`] builder
//! - [`JsonLoader::with_user_override`] - add user override for deep-merge
//! - [`JsonLoader::skip_units`] - skip unit validation for non-physical JSON
//! - [`JsonLoader::load`] - execute the pipeline
//!
//! Internal helpers (`read_json`, `validate`, `validate_with_registry`, `fill_defaults`)
//! are not part of the public API.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
// Internal helpers
// ---------------------------------------------------------------------------

/// Reads a JSON file from disk.
///
/// # Errors
/// Returns [`JsonError::Io`] if the file cannot be opened or read.
/// Returns [`JsonError::Parse`] if the content is not valid JSON.
fn read_json(path: &Path) -> Result<Value, JsonError> {
    let text = std::fs::read_to_string(path).map_err(|e| JsonError::Io {
        path: path.to_owned(),
        source: e,
    })?;
    serde_json::from_str(&text).map_err(|e| JsonError::Parse {
        path: path.to_owned(),
        source: e,
    })
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
fn validate_with_registry(
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
pub(crate) fn fill_defaults(value: &mut Value, schema_path: &Path) -> Result<(), JsonError> {
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

// ---------------------------------------------------------------------------
// Builder pattern (new API)
// ---------------------------------------------------------------------------

/// Main entry point for JSON loading with a builder pattern.
///
/// This function returns a [`JsonLoader`] builder that can be configured
/// with optional parameters before calling `.load()`.
///
/// # Example
///
/// ```no_run
/// use delta_v_json::load;
/// use std::path::PathBuf;
///
/// // Simple load with defaults
/// let value = load(PathBuf::from("config.json"), PathBuf::from("config.schema.json"))
///     .load()?;
///
/// // With user override
/// let value = load(PathBuf::from("config.json"), PathBuf::from("config.schema.json"))
///     .with_user_override(Some(&PathBuf::from("user.json")))
///     .load()?;
///
/// // Skip unit validation for non-physical JSON
/// let value = load(PathBuf::from("config.json"), PathBuf::from("config.schema.json"))
///     .skip_units()
///     .load()?;
/// # Ok::<(), delta_v_json::JsonError>(())
/// ```
pub fn load(json_path: PathBuf, schema_path: PathBuf) -> JsonLoader {
    JsonLoader::new(json_path, schema_path)
}

/// Builder for JSON loading with configurable options.
///
/// Created by the [`load`] function. Use methods to configure
/// the loading behavior, then call `.load()` to execute.
///
/// # Example
///
/// ```no_run
/// use delta_v_json::load;
/// use std::path::PathBuf;
///
/// let value = load(
///     PathBuf::from("config.json"),
///     PathBuf::from("config.schema.json"),
/// )
/// .with_user_override(Some(&PathBuf::from("user.json")))
/// .load()?;
/// # Ok::<(), delta_v_json::JsonError>(())
/// ```
#[derive(Debug)]
pub struct JsonLoader {
    json_path: PathBuf,
    schema_path: PathBuf,
    user_override_path: Option<PathBuf>,
    validate_units: bool,
}

impl JsonLoader {
    /// Creates a new `JsonLoader` with the given paths.
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn new(json_path: PathBuf, schema_path: PathBuf) -> Self {
        Self {
            json_path,
            schema_path,
            user_override_path: None,
            validate_units: true,
        }
    }

    /// Adds a user override file to deep-merge on top of the defaults.
    ///
    /// The user override is loaded, deep-merged with the defaults,
    /// and the result is re-validated against the schema.
    ///
    /// # Errors
    ///
    /// If the user override file exists but fails to load or validate,
    /// an error is returned. If the file doesn't exist, it's silently skipped.
    #[must_use]
    pub fn with_user_override(mut self, path: Option<&Path>) -> Self {
        if let Some(path) = path {
            self.user_override_path = Some(path.to_path_buf());
        }
        self
    }

    /// Skips unit validation for physical quantities.
    ///
    /// Use this for JSON files that don't contain physical quantities
    /// with `{"value", "unit"}` objects, or when you want to bypass
    /// the unit registry check.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn skip_units(mut self) -> Self {
        self.validate_units = false;
        self
    }

    /// Executes the JSON loading pipeline.
    ///
    /// The pipeline performs:
    /// 1. Read JSON from `json_path`
    /// 2. Validate against schema (with cross-schema `$ref` support)
    /// 3. Fill schema defaults
    /// 4. (Optional) Deep-merge user override and re-validate
    /// 5. (Optional) Validate physical quantity units
    ///
    /// # Errors
    ///
    /// Returns [`JsonError`] if any step fails.
    pub fn load(self) -> Result<Value, JsonError> {
        let schema_dir = self.schema_path.parent().unwrap_or_else(|| Path::new(""));

        // Step 1: Read the JSON file
        let mut value = read_json(&self.json_path)?;

        // Step 2: Validate with registry to resolve cross-schema $refs
        validate_with_registry(&value, &self.schema_path, &self.json_path, schema_dir)?;

        // Step 3: Fill defaults with schema map to resolve cross-schema $refs
        fill_defaults(&mut value, &self.schema_path)?;

        // Step 4: Apply user override if provided
        if let Some(user_path) = &self.user_override_path {
            if user_path.exists() {
                let user_value = read_json(user_path)?;
                deep_merge(&mut value, user_value);
                // Re-validate after merge
                validate_with_registry(&value, &self.schema_path, user_path, schema_dir)?;
            } else {
                tracing::info!("user override file not found at {}", user_path.display());
            }
        }

        // Step 5: Validate units if enabled
        if self.validate_units {
            let units_schema_path = schema_dir.join("units.schema.json");
            validate_units(&value, &self.json_path, &units_schema_path)?;
        }

        Ok(value)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Deep-merges `src` on top of `dst`.
///
/// Rules:
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
                && !allowed_units.contains(unit_str)
            {
                return Err(JsonError::InvalidUnit {
                    path: data_path.to_owned(),
                    pointer,
                    unit: unit_str.clone(),
                    units_schema: units_schema_path.to_owned(),
                });
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

    if has_defaults { Some(defaults) } else { None }
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
