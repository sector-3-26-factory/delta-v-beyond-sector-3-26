// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Template loading and merging utilities.
//!
//! This module provides the single source of truth for loading entity templates
//! from JSON files. All template loading uses `delta-v-json` for validation
//! (ADR-0038, ADR-0040).
//!
//! See ADR-0049 for the template system reorganization.

use std::path::Path;

use delta_v_json::load;
use serde_json::Value;

use crate::error::AssetError;
use crate::paths::{get_workspace_root, resolve_template_path};

/// Validates that referenced sound files exist in the assets/audio directory.
/// Per ADR-0013, missing referenced files are hard errors at load time.
fn validate_sound_files(template: &Value, template_path: &Path) -> Result<(), AssetError> {
    let workspace_root = get_workspace_root();
    let audio_root = workspace_root.join("assets/audio");

    // Check ship sounds (thrust, hit) from player_controlled_ship template
    if let Some(sounds) = template.get("sounds").and_then(|s| s.as_object()) {
        for (key, value) in sounds {
            if let Some(sound_path) = value.as_str() {
                let full_path = audio_root.join(sound_path);
                if !full_path.exists() {
                    return Err(AssetError::Validation(format!(
                        "sound file not found: {} (referenced in {} at .sounds.{})",
                        full_path.display(),
                        template_path.display(),
                        key
                    )));
                }
            }
        }
    }

    // Check weapon sounds from ship template
    if let Some(weapons) = template.get("weapons").and_then(|w| w.as_array()) {
        for (i, weapon) in weapons.iter().enumerate() {
            if let Some(sound) = weapon.get("sound").and_then(|s| s.as_str()) {
                let full_path = audio_root.join(sound);
                if !full_path.exists() {
                    return Err(AssetError::Validation(format!(
                        "weapon sound file not found: {} (referenced in {} at .weapons[{}].sound)",
                        full_path.display(),
                        template_path.display(),
                        i
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Loads a template from a path, validates it, and returns the JSON value.
///
/// This is the SINGLE function for loading templates. All crates should use
/// this function instead of loading templates directly.
///
/// # Arguments
///
/// * `category` - The entity type category (e.g., "ships", "asteroids") used for schema lookup
/// * `name` - The template name, which may include the category prefix (e.g., "ships/space-fighter-comrade1280")
/// * `filename` - The specific JSON filename to load (e.g., `ship.json`, `player_controlled_ship.json`)
/// * `schema_name` - The schema filename (e.g., `ship.schema.json`, `player_controlled_ship.schema.json`)
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the template file does not exist.
/// Returns [`AssetError::Validation`] if the template fails schema validation.
/// Returns [`AssetError::InvalidUnit`] if a physical quantity has an invalid unit.
///
/// # Panics
///
/// Panics if `CARGO_MANIFEST_DIR` is not set or the workspace directory structure
/// is unexpected. This should never happen in normal cargo builds.
#[allow(clippy::expect_used)] // CARGO_MANIFEST_DIR is always set by cargo; workspace structure is fixed
pub fn load_template(
    category: &str,
    name: &str,
    filename: &str,
    schema_name: &str,
) -> Result<Value, AssetError> {
    let template_path = if name.contains('/') {
        format!("assets/templates/{name}/{filename}")
    } else {
        format!(
            "assets/{}/{filename}",
            resolve_template_path(category, name)
        )
    };
    let full_path = get_workspace_root().join(&template_path);
    let schema_path = get_workspace_root().join(format!("assets/json/schema/{schema_name}"));

    load_template_from_paths(&full_path, &schema_path)
}

/// Loads a player-controlled ship template, merging base ship with player-specific data.
///
/// Per ADR-0043, player-controlled ship templates are co-located with a base ship template
/// in the same directory. This function loads the `player_controlled_ship.json` and merges
/// it with the co-located `ship.json`.
///
/// Returns a tuple of (`entity_type`, `template_path`, `merged_template`, `mesh_template_path`).
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the template file does not exist.
/// Returns [`AssetError::Validation`] if the template fails schema validation.
pub fn load_player_controlled_ship(
    ship_name: &str,
) -> Result<(String, String, Value, String), AssetError> {
    let player_template = load_template(
        "ships",
        ship_name,
        "player_controlled_ship.json",
        "player_controlled_ship.schema.json",
    )?;
    let base_ship = load_template("ships", ship_name, "ship.json", "ship.schema.json")?;

    let template_path = format!("templates/{ship_name}/player_controlled_ship.json");
    let ship_template_path = format!("templates/{ship_name}/ship.json");
    let mesh_template_path = ship_template_path.replace("ship.json", "mesh.glb");

    let mut merged = base_ship;
    if let (Some(merged_obj), Some(player_obj)) =
        (merged.as_object_mut(), player_template.as_object())
    {
        for (key, value) in player_obj {
            if key != "entity_type" {
                merged_obj.insert(key.clone(), value.clone());
            }
        }
        merged_obj.insert(
            "entity_type".to_string(),
            Value::String("player_controlled_ship".to_string()),
        );
    }

    // Validate referenced sound files exist (ADR-0013: no silent fallbacks).
    let full_template_path = get_workspace_root().join(&template_path);
    validate_sound_files(&merged, &full_template_path)?;

    Ok((
        "player_controlled_ship".to_string(),
        template_path,
        merged,
        mesh_template_path,
    ))
}

/// Loads an AI-controlled ship template, merging base ship with AI-specific data.
///
/// AI-controlled ship templates are co-located with a base ship template in the same
/// directory. This function loads the `ai_controlled_ship.json` and merges it with
/// the co-located `ship.json`.
///
/// Returns a tuple of (`entity_type`, `template_path`, `merged_template`, `mesh_template_path`).
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the template file does not exist.
/// Returns [`AssetError::Validation`] if the template fails schema validation.
pub fn load_ai_controlled_ship(
    ship_name: &str,
) -> Result<(String, String, Value, String), AssetError> {
    let ai_template = load_template(
        "ships",
        ship_name,
        "ai_controlled_ship.json",
        "ai_controlled_ship.schema.json",
    )?;
    let base_ship = load_template("ships", ship_name, "ship.json", "ship.schema.json")?;

    let template_path = format!("templates/{ship_name}/ai_controlled_ship.json");
    let ship_template_path = format!("templates/{ship_name}/ship.json");
    let mesh_template_path = ship_template_path.replace("ship.json", "mesh.glb");

    let mut merged = base_ship;
    if let (Some(merged_obj), Some(ai_obj)) = (merged.as_object_mut(), ai_template.as_object()) {
        for (key, value) in ai_obj {
            if key != "entity_type" {
                merged_obj.insert(key.clone(), value.clone());
            }
        }
        merged_obj.insert(
            "entity_type".to_string(),
            Value::String("ai_controlled_ship".to_string()),
        );
    }

    // Validate referenced sound files exist (ADR-0013: no silent fallbacks).
    let full_template_path = get_workspace_root().join(&template_path);
    validate_sound_files(&merged, &full_template_path)?;

    Ok((
        "ai_controlled_ship".to_string(),
        template_path,
        merged,
        mesh_template_path,
    ))
}

/// Loads an asteroid template.
///
/// Returns a tuple of (`entity_type`, `template_path`, `template`, `mesh_template_path`).
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the template file does not exist.
/// Returns [`AssetError::Validation`] if the template fails schema validation.
pub fn load_asteroid(name: &str) -> Result<(String, String, Value, String), AssetError> {
    // INVARIANT: The name may or may not have the "asteroids/" prefix.
    // If it has the prefix, we strip it for the template_path; otherwise, we use the name as-is.
    // The `unwrap_or` is intentional: callers may pass either "asteroids/my-asteroid" or "my-asteroid".
    // Both are valid and result in the same template being loaded.
    let template_name = name.strip_prefix("asteroids/").unwrap_or(name);
    let template_path = format!("templates/asteroids/{template_name}/asteroid.json");
    let mesh_path = template_path.replace("asteroid.json", "mesh.glb");
    let template = load_template("asteroids", name, "asteroid.json", "asteroid.schema.json")?;
    Ok(("asteroid".to_string(), template_path, template, mesh_path))
}

/// Loads a ship template.
///
/// Returns a tuple of (`entity_type`, `template_path`, `template`, `mesh_template_path`).
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the template file does not exist.
/// Returns [`AssetError::Validation`] if the template fails schema validation.
pub fn load_ship(name: &str) -> Result<(String, String, Value, String), AssetError> {
    // INVARIANT: The name may or may not have the "ships/" prefix.
    // If it has the prefix, we strip it for the template_path; otherwise, we use the name as-is.
    // The `unwrap_or` is intentional: callers may pass either "ships/my-ship" or "my-ship".
    // Both are valid and result in the same template being loaded.
    let template_name = name.strip_prefix("ships/").unwrap_or(name);
    let template_path = format!("templates/ships/{template_name}/ship.json");
    let mesh_path = template_path.replace("ship.json", "mesh.glb");
    let template = load_template("ships", name, "ship.json", "ship.schema.json")?;

    // Validate referenced sound files exist (ADR-0013: no silent fallbacks).
    let full_template_path = get_workspace_root().join(&template_path);
    validate_sound_files(&template, &full_template_path)?;

    Ok(("ship".to_string(), template_path, template, mesh_path))
}

/// Loads and validates a template from explicit paths.
///
/// Used by tests and future tooling.
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the template file does not exist.
/// Returns [`AssetError::Validation`] if the template fails schema validation.
/// Returns [`AssetError::InvalidUnit`] if a physical quantity has an invalid unit.
fn load_template_from_paths(template_path: &Path, schema_path: &Path) -> Result<Value, AssetError> {
    let template = load(template_path.to_path_buf(), schema_path.to_path_buf())
        .load()
        .map_err(|e| map_json_error(e, template_path))?;

    Ok(template)
}

/// Maps a `delta-v-json` error to an `AssetError`.
fn map_json_error(e: delta_v_json::error::JsonError, _context: &Path) -> AssetError {
    use delta_v_json::error::JsonError;

    match e {
        JsonError::Io { path, source } => AssetError::Io { path, source },
        JsonError::Parse { path, source } => AssetError::JsonParse { path, source },
        JsonError::Schema {
            path,
            pointer,
            reason,
        } => AssetError::Validation(format!("{}:{}: {}", path.display(), pointer, reason)),
        JsonError::SchemaLoad { path, reason } => AssetError::SchemaLoad { path, reason },
        JsonError::InvalidUnit {
            path,
            pointer,
            unit,
            units_schema,
        } => AssetError::InvalidUnit {
            path,
            pointer,
            unit,
            units_schema,
        },
    }
}

/// Loads a weapon definition by name.
///
/// Weapon definitions are stored at `assets/weapons/<name>/weapon.json`.
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the weapon file does not exist.
/// Returns [`AssetError::Validation`] if the weapon fails schema validation.
pub fn load_weapon_definition(name: &str) -> Result<Value, AssetError> {
    let template_path = get_workspace_root().join(format!("assets/weapons/{name}/weapon.json"));
    let schema_path = get_workspace_root().join("assets/json/schema/weapon.schema.json");
    load_template_from_paths(&template_path, &schema_path)
}

/// Loads a projectile definition by name.
///
/// Projectile definitions are stored at `assets/projectiles/<name>/projectile.json`.
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the projectile file does not exist.
/// Returns [`AssetError::Validation`] if the projectile fails schema validation.
pub fn load_projectile_definition(name: &str) -> Result<Value, AssetError> {
    let template_path =
        get_workspace_root().join(format!("assets/projectiles/{name}/projectile.json"));
    let schema_path = get_workspace_root().join("assets/json/schema/projectile.schema.json");
    load_template_from_paths(&template_path, &schema_path)
}
