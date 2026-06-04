// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Template loading and validation.
//!
//! Loads entity templates from files and validates them against their
//! schemas per ADR-0038.
//!
//! For `player_controlled_ship` templates, the loader additionally resolves
//! the `ship_template` reference: it loads the referenced ship template,
//! merges the two (ship properties + player cameras), and validates the
//! merged result against `ship.schema.json`.

use std::path::{Path, PathBuf};

use delta_v_json::loader as json_loader;
use serde_json::Value;

use crate::error::WorldError;

/// Loads and validates a template file.
///
/// For `player_controlled_ship` templates, the `ship_template` field is
/// resolved and the referenced ship template is merged in before validation.
///
/// # Arguments
///
/// * `template_path` - Relative path to the template (e.g., `templates/ships/player_ship/template.json`)
/// * `entity_type` - The entity type discriminator (e.g., `"player_controlled_ship"`)
///
/// # Returns
///
/// The loaded, merged (if applicable), and validated template JSON.
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

    let template = load_template_from_paths(&template_file, &schema_file, entity_type)?;

    // For player_controlled_ship, resolve and merge the referenced ship template.
    if entity_type == "player_controlled_ship" {
        return merge_ship_template(&template, &assets_root);
    }

    Ok(template)
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
                "entity_type mismatch: template has '{type_in_template}', expected '{entity_type}'"
            ),
        });
    }

    Ok(template)
}

/// Loads and merges a referenced ship template into the player template.
///
/// The `player_controlled_ship` template references a base ship template via
/// the `ship_template` field. This function:
/// 1. Reads the `ship_template` path from the player template
/// 2. Loads and validates the referenced ship template
/// 3. Merges: ship template provides base properties, player template adds cameras
/// 4. Sets `entity_type` to `player_controlled_ship` on the merged result
///
/// # Errors
///
/// Returns [`WorldError`] if the ship template cannot be loaded or validated.
fn merge_ship_template(player_template: &Value, root: &Path) -> Result<Value, WorldError> {
    let ship_template_path = player_template
        .get("ship_template")
        .and_then(Value::as_str)
        .ok_or_else(|| WorldError::Schema {
            path: PathBuf::from("<player_controlled_ship template>"),
            pointer: "/ship_template".to_string(),
            reason: "player_controlled_ship template missing 'ship_template' field".to_string(),
        })?;

    // Load the referenced ship template.
    let ship_template_file = root.join(ship_template_path);
    let ship_schema_file = root.join("json/schema/ship.schema.json");
    let ship_template = json_loader::load_validated(&ship_template_file, &ship_schema_file)
        .map_err(|e| map_json_error(e, &ship_template_file))?;

    // Verify the referenced template is a ship.
    let ship_type = ship_template
        .get("entity_type")
        .and_then(Value::as_str)
        .ok_or_else(|| WorldError::Schema {
            path: ship_template_file.clone(),
            pointer: "/entity_type".to_string(),
            reason: "referenced ship template missing 'entity_type' field".to_string(),
        })?;

    if ship_type != "ship" {
        return Err(WorldError::Schema {
            path: ship_template_file,
            pointer: "/entity_type".to_string(),
            reason: format!(
                "referenced ship template must have entity_type 'ship', found '{ship_type}'"
            ),
        });
    }

    // Merge: start with ship template properties, then overlay player-specific fields.
    let mut merged = ship_template;

    if let (Some(merged_obj), Some(player_obj)) =
        (merged.as_object_mut(), player_template.as_object())
    {
        for (key, value) in player_obj {
            // Skip entity_type (we set it below) and ship_template (metadata, not a ship property).
            if key != "entity_type" && key != "ship_template" {
                merged_obj.insert(key.clone(), value.clone());
            }
        }
        // Set entity_type to player_controlled_ship.
        merged_obj.insert(
            "entity_type".to_string(),
            Value::String("player_controlled_ship".to_string()),
        );
    }

    Ok(merged)
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
