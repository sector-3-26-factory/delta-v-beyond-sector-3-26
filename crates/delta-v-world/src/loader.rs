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

//! World definition loader.
//!
//! Delegates the read/validate/fill-defaults pipeline to `delta-v-json`
//! (ADR-0038). Missing or invalid files are hard errors (ADR-0013).
//!
//! See ADR-0019 (Asset pipeline) and ADR-0020 (Save and load format).

use std::path::{Path, PathBuf};

use delta_v_json::{error::JsonError, load};

use crate::{error::WorldError, world_def::WorldDef};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Loads and validates a world definition from the given path.
///
/// Reads the world JSON, validates it against `assets/json/schema/world.schema.json`,
/// fills in schema defaults, and deserialises into [`WorldDef`].
///
/// # Errors
/// Returns [`WorldError`] if the file cannot be read, parsed, or validated.
pub fn load_world(json_path: &Path) -> Result<WorldDef, WorldError> {
    let schema_path = PathBuf::from("assets/json/schema/world.schema.json");
    load_world_from_paths(json_path, &schema_path)
}

/// Loads and validates the default world definition.
///
/// Reads `assets/worlds/default.world.json`, validates it against
/// `assets/json/schema/world.schema.json`, fills in schema defaults, and
/// deserialises into [`WorldDef`].
///
/// # Errors
/// Returns [`WorldError`] if the file cannot be read, parsed, or validated.
pub fn load_default_world() -> Result<WorldDef, WorldError> {
    let json_path = PathBuf::from("assets/worlds/default.world.json");
    load_world(&json_path)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Loads and validates a world definition from explicit paths.
///
/// Used internally by the public API and by tests.
fn load_world_from_paths(json_path: &Path, schema_path: &Path) -> Result<WorldDef, WorldError> {
    let value = load(json_path.to_path_buf(), schema_path.to_path_buf())
        .load()
        .map_err(|e| map_json_error(e, json_path))?;
    serde_json::from_value(value).map_err(|e| WorldError::Parse {
        path: json_path.to_owned(),
        source: e,
    })
}

/// Loads and validates a world definition from explicit paths.
///
/// Used by tests and future tooling that needs to load arbitrary world
/// files.
///
/// # Errors
/// Returns [`WorldError`] if the file cannot be read, parsed, or validated.
#[cfg(test)]
pub(crate) fn load_test_world_from_paths(
    json_path: &Path,
    schema_path: &Path,
) -> Result<WorldDef, WorldError> {
    load_world_from_paths(json_path, schema_path)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Maps a [`JsonError`] from `delta-v-json` to a [`WorldError`].
fn map_json_error(e: JsonError, _context: &Path) -> WorldError {
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
