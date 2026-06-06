// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Template loading and validation.
//!
//! Loads entity templates from files and validates them against their
//! schemas per ADR-0038.
//!
//! The loader now loads a single template file. Merging of `player_controlled_ship`
//! templates is done in the world loader (lib.rs)
//! since it needs to know the `player_controlled` flag from the world definition.

use std::path::{Path, PathBuf};

use delta_v_json::loader as json_loader;
use serde_json::Value;

use crate::error::WorldError;

/// Path to the units schema file (relative to assets root).
const UNITS_SCHEMA_PATH: &str = "json/schema/units.schema.json";

/// Resolves a short template path to the full template file path.
///
/// Short format: `"ships/debug-ship-cube"` + `entity_type` `"ship"` → `"templates/ships/debug-ship-cube/ship.json"`
/// If the path already looks like a full path (starts with `templates/`), it is returned as-is.
pub fn resolve_template_path(short_path: &str, entity_type: &str) -> String {
    if short_path.starts_with("templates/") {
        // Already a full path like "templates/ships/debug-ship-cube/ship.json"
        return short_path.to_string();
    }
    format!("templates/{short_path}/{entity_type}.json")
}

/// Loads and validates a template file.
///
/// # Arguments
///
/// * `template_path` - Relative path to the template (e.g., `templates/ships/debug-ship-cube/ship.json`)
/// * `entity_type` - The entity type discriminator (e.g., `"ship"` or `"player_controlled_ship"`)
///
/// # Returns
///
/// The loaded and validated template JSON.
///
/// # Errors
///
/// Returns [`WorldError`] if the file cannot be read, parsed, or fails schema validation.
/// # Panics
///
/// Panics if `CARGO_MANIFEST_DIR` is not set or the workspace directory structure
/// is unexpected. This should never happen in normal cargo builds.
#[allow(clippy::expect_used)] // CARGO_MANIFEST_DIR is always set by cargo; workspace structure is fixed
pub fn load_template(template_path: &str, entity_type: &str) -> Result<Value, WorldError> {
    // Determine the workspace root from CARGO_MANIFEST_DIR (crates/delta-v-world -> workspace root).
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("CARGO_MANIFEST_DIR parent (crates dir) must exist")
        .parent()
        .expect("workspace root must exist");
    let assets_root = workspace_root.join("assets");

    let template_file = assets_root.join(template_path);
    let schema_file = assets_root.join(format!("json/schema/{entity_type}.schema.json"));
    let units_schema_file = assets_root.join(UNITS_SCHEMA_PATH);

    load_template_from_paths(
        &template_file,
        &schema_file,
        entity_type,
        &units_schema_file,
    )
}

/// Loads and validates a template from explicit paths.
///
/// Used by tests and future tooling.
///
/// # Errors
///
/// Returns [`WorldError`] if validation fails.
fn load_template_from_paths(
    template_path: &Path,
    schema_path: &Path,
    entity_type: &str,
    units_schema_path: &Path,
) -> Result<Value, WorldError> {
    // Load and validate template against its schema with unit validation.
    let template =
        json_loader::load_validated_with_units(template_path, schema_path, units_schema_path)
            .map_err(|e| map_json_error(e, template_path))?;

    // Verify entity_type field matches the expected type (ADR-0013: hard error).
    let type_in_template = template
        .get("entity_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WorldError::Schema {
            path: template_path.to_owned(),
            pointer: "/entity_type".to_string(),
            reason: "template missing or invalid 'entity_type' field".to_string(),
        })?
        .to_string();

    if type_in_template != entity_type {
        return Err(WorldError::Schema {
            path: template_path.to_owned(),
            pointer: "/entity_type".to_string(),
            reason: format!(
                "entity_type mismatch: template has '{type_in_template}', expected '{entity_type}'"
            ),
        });
    }

    Ok(template)
}

/// Maps a `delta-v-json` error to a `WorldError`.
fn map_json_error(e: delta_v_json::error::JsonError, _context: &Path) -> WorldError {
    use delta_v_json::error::JsonError;

    match e {
        JsonError::Io { path, source } => WorldError::Io { path, source },
        JsonError::Parse { path, source } => WorldError::Parse { path, source },
        JsonError::Schema {
            path,
            pointer,
            reason,
        } => WorldError::Schema {
            path,
            pointer,
            reason,
        },
        JsonError::SchemaLoad { path, reason } => WorldError::SchemaLoad { path, reason },
        JsonError::InvalidUnit {
            path,
            pointer,
            unit,
            units_schema,
        } => WorldError::InvalidUnit {
            path,
            pointer,
            unit,
            units_schema,
        },
    }
}
