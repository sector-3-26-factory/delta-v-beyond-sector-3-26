// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Template loading and validation.
//!
//! Loads entity templates from files and validates them against their
//! schemas per ADR-0038.

use std::path::{Path, PathBuf};

use delta_v_json::loader as json_loader;
use serde_json::Value;

use crate::error::WorldError;

/// Loads and validates a template file.
///
/// # Arguments
///
/// * `template_path` - Relative path to the template (e.g., `templates/ships/local_player_ship.json`)
/// * `entity_type` - The entity type discriminator (e.g., `"local_player_ship"`)
///
/// # Returns
///
/// The loaded and validated template JSON.
///
/// # Errors
///
/// Returns [`WorldError`] if the file cannot be read, parsed, or fails schema validation.
pub fn load_template(template_path: &str, entity_type: &str) -> Result<Value, WorldError> {
    let root = PathBuf::from("assets");
    let template_file = root.join(template_path);
    let schema_file = root.join(format!("json/schema/{entity_type}.schema.json"));

    load_template_from_paths(&template_file, &schema_file, entity_type)
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
) -> Result<Value, WorldError> {
    // Load and validate template against its schema.
    let template = json_loader::load_validated(template_path, schema_path)
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
                "entity_type mismatch: template has '{}', expected '{}'",
                type_in_template, entity_type
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
    }
}
