// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Configuration file loader: reads, validates and merges JSON config files.
//!
//! Implements the two-layer config system described in ADR-0010:
//! 1. Default layer: `assets/config/<file>.json` (shipped with the game).
//! 2. User layer: `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/<file>.json`
//!    (optional user overrides).
//!
//! The read/validate/fill-defaults pipeline is provided by `delta-v-json`
//! (ADR-0038). This module owns the merge logic and config-specific path
//! resolution.

use std::path::{Path, PathBuf};

use delta_v_json::{error::JsonError, loader as json_loader};
use directories::ProjectDirs;
use serde_json::Value;

use crate::{error::ConfigError, keybindings::Keybindings};
use delta_v_core::{DebugConfig, DiagnosticsConfig};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Loads and validates diagnostics configuration.
///
/// Uses [`load_with_user_override`] with a no-op user path function.
/// Diagnostics config does not support user overrides; see [`load_keybindings`]
/// and [`load_debug`] for configs with XDG user override support.
///
/// # Errors
/// Returns [`ConfigError`] if the file cannot be read, parsed, or validated.
pub fn load_diagnostics() -> Result<DiagnosticsConfig, ConfigError> {
    load_with_user_override("diagnostics", || None)
}

/// Loads, validates and merges debug configuration.
///
/// Uses [`load_with_user_override`] to handle defaults + user override merging.
/// See ADR-0010 (configuration system).
///
/// # Errors
/// Returns [`ConfigError`] if any step fails. All errors include the
/// file path and a precise description (ADR-0016).
pub fn load_debug() -> Result<DebugConfig, ConfigError> {
    load_with_user_override("debug", user_debug_path)
}

/// Loads, validates and merges keybindings configuration.
///
/// Uses [`load_with_user_override`] to handle defaults + user override merging.
/// See ADR-0010 (configuration system) and ADR-0011 (keybindings).
///
/// # Errors
/// Returns [`ConfigError`] if any step fails. All errors include the
/// file path and a precise description (ADR-0016).
pub fn load_keybindings() -> Result<Keybindings, ConfigError> {
    load_with_user_override("keybindings", user_keybindings_path)
}

// ---------------------------------------------------------------------------
// Test / hot-reload entry points
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
    json_loader::load_validated(json_path, schema_path).map_err(|e| map_json_error(e, json_path))
}

/// Public wrapper around [`deep_merge`] used by tests.
pub fn merge_user_override(dst: &mut Value, src: Value) {
    deep_merge(dst, src);
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Maps a [`JsonError`] from `delta-v-json` to a [`ConfigError`].
fn map_json_error(e: JsonError, _context: &Path) -> ConfigError {
    match e {
        JsonError::Io { path, source } => ConfigError::Io { path, source },
        JsonError::Parse { path, source } => ConfigError::Parse { path, source },
        JsonError::Schema {
            path,
            pointer,
            reason,
        } => ConfigError::Schema {
            path,
            pointer,
            reason,
        },
        JsonError::SchemaLoad { path, reason } => ConfigError::SchemaLoad { path, reason },
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

/// Returns the platform-appropriate user debug config override path, if
/// the base directory can be determined.
///
/// On Linux: `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/debug.json`
/// (or `~/.config/...` if `XDG_CONFIG_HOME` is unset). See ADR-0010.
fn user_debug_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "delta-v", "delta-v-beyond-sector-3-26")
        .map(|dirs| dirs.config_dir().join("debug.json"))
}

/// Generic helper to load a config file with user override support.
///
/// Steps (per ADR-0010, ADR-0012, ADR-0013):
/// 1. Load and validate defaults from `assets/config/{name}.json`.
/// 2. Fill schema defaults.
/// 3. If user override exists at XDG path, deep-merge and re-validate.
/// 4. Deserialise into type `T`.
///
/// # Arguments
/// - `name`: config file name without extension (e.g., "keybindings", "debug")
/// - `user_path_fn`: function that returns the user override path
/// - `T`: the target deserialisation type
///
/// # Errors
/// Returns [`ConfigError`] if any step fails.
fn load_with_user_override<T>(
    name: &str,
    user_path_fn: impl Fn() -> Option<PathBuf>,
) -> Result<T, ConfigError>
where
    T: serde::de::DeserializeOwned,
{
    let defaults_path = PathBuf::from(format!("assets/config/{name}.json"));
    let schema_path = PathBuf::from(format!("assets/json/schema/{name}.schema.json"));

    // 1 + 2: load, validate, fill defaults via delta-v-json.
    log::debug!("loading defaults from {}", defaults_path.display());
    let mut merged = json_loader::load_validated(&defaults_path, &schema_path)
        .map_err(|e| map_json_error(e, &defaults_path))?;
    log::debug!(
        "defaults loaded from {}: {}",
        defaults_path.display(),
        merged
    );

    // 3: try user override.
    if let Some(user_path) = user_path_fn() {
        if user_path.exists() {
            log::info!("user override found at {}", user_path.display());
            let user_value =
                json_loader::read_json(&user_path).map_err(|e| map_json_error(e, &user_path))?;
            log::debug!("user override content: {}", user_value);
            deep_merge(&mut merged, user_value);
            log::debug!("merged config: {}", merged);
            json_loader::validate(&merged, &schema_path, &user_path)
                .map_err(|e| map_json_error(e, &user_path))?;
            log::info!("user override for {} validated and merged", name);
        } else {
            log::debug!(
                "no user override at {} (file does not exist)",
                user_path.display()
            );
        }
    }

    // 4: deserialise.
    log::info!(
        "final {} config after defaults + overrides: {}",
        name,
        merged
    );
    serde_json::from_value(merged).map_err(|e| ConfigError::Parse {
        path: defaults_path,
        source: e,
    })
}
