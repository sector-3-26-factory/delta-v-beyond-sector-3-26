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

use delta_v_json::{error::JsonError, load};
use directories::ProjectDirs;
use serde_json::Value;

use crate::{error::ConfigError, keybindings::Keybindings};
use delta_v_core::{DebugConfig, DiagnosticsConfig, FlightAssistConfig};

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

/// Loads, validates and merges flight-assist configuration.
///
/// Uses [`load_with_user_override`] to handle defaults + user override merging.
/// See ADR-0010 (configuration system).
///
/// # Errors
/// Returns [`ConfigError`] if any step fails. All errors include the
/// file path and a precise description (ADR-0016).
pub fn load_flight_assist() -> Result<FlightAssistConfig, ConfigError> {
    load_with_user_override("flight-assist", user_flight_assist_path)
}

// ---------------------------------------------------------------------------
// Test entry points
// ---------------------------------------------------------------------------

/// Entry point used by tests.
///
/// Reads `json_path`, validates against `schema_path`, fills defaults,
/// and returns the resulting [`Value`].
///
/// # Errors
/// Returns [`ConfigError`] if the file cannot be read, parsed, or
/// validated.
#[allow(dead_code)]
pub(crate) fn load_and_validate_from_paths(
    json_path: &Path,
    schema_path: &Path,
) -> Result<Value, ConfigError> {
    load(json_path.to_path_buf(), schema_path.to_path_buf())
        .load()
        .map_err(|e| map_json_error(e, json_path))
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
        JsonError::InvalidUnit {
            path,
            pointer,
            unit,
            units_schema,
        } => ConfigError::InvalidUnit {
            path,
            pointer,
            unit,
            units_schema,
        },
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

/// Returns the platform-appropriate user flight-assist config override path, if
/// the base directory can be determined.
///
/// On Linux: `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/flight-assist.json`
/// (or `~/.config/...` if `XDG_CONFIG_HOME` is unset). See ADR-0010.
fn user_flight_assist_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "delta-v", "delta-v-beyond-sector-3-26")
        .map(|dirs| dirs.config_dir().join("flight-assist.json"))
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

    let merged = load(defaults_path.clone(), schema_path)
        .with_user_override(user_path_fn().as_deref())
        .load()
        .map_err(|e| map_json_error(e, &defaults_path))?;

    serde_json::from_value(merged).map_err(|e| ConfigError::Parse {
        path: defaults_path,
        source: e,
    })
}
