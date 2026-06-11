// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Template loading and merging utilities.

use serde_json::Value;

use crate::error::AssetError;
use crate::paths::resolve_template_path;

/// Loads a template from a path, validates it, and returns the JSON value.
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the template file does not exist.
pub fn load_template(category: &str, name: &str) -> Result<Value, AssetError> {
    let path = resolve_template_path(category, name);
    let template_path = format!("assets/{}.json", path.trim_end_matches('/'));

    let json_content = std::fs::read_to_string(&template_path).map_err(|_| {
        AssetError::TemplateNotFound(format!("{name} (expected at {template_path})"))
    })?;

    let value: Value = serde_json::from_str(&json_content)?;
    Ok(value)
}

/// Loads a player-controlled ship template, merging base ship with player-specific data.
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the template file does not exist.
pub fn load_player_controlled_ship(ship_name: &str) -> Result<Value, AssetError> {
    // Load the player_controlled_ship template
    let player_path = resolve_template_path("ships", ship_name);
    let player_template_path = format!("assets/{}.json", player_path.trim_end_matches('/'));

    // For now, just load the player_controlled_ship template
    // In a full implementation, this would merge with the base ship template
    let json_content = std::fs::read_to_string(&player_template_path).map_err(|_| {
        AssetError::TemplateNotFound(format!("{ship_name} (expected at {player_template_path})"))
    })?;

    let value: Value = serde_json::from_str(&json_content)?;
    Ok(value)
}

/// Loads an asteroid template.
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the template file does not exist.
pub fn load_asteroid(name: &str) -> Result<Value, AssetError> {
    load_template("asteroids", name)
}

/// Loads a ship template.
///
/// # Errors
///
/// Returns [`AssetError::TemplateNotFound`] if the template file does not exist.
pub fn load_ship(name: &str) -> Result<Value, AssetError> {
    load_template("ships", name)
}
