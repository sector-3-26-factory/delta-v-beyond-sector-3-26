// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! World definition loader.
//!
//! Delegates the read/validate/fill-defaults pipeline to `delta-v-json`
//! (ADR-0038). Missing or invalid files are hard errors (ADR-0013).
//!
//! See ADR-0019 (Asset pipeline) and ADR-0020 (Save and load format).

use std::path::{Path, PathBuf};

use delta_v_json::{error::JsonError, loader as json_loader};

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

/// Loads and validates a world definition from explicit paths.
///
/// Used by tests and future tooling that needs to load arbitrary world
/// files.
///
/// # Errors
/// Returns [`WorldError`] if the file cannot be read, parsed, or validated.
pub fn load_world_from_paths(json_path: &Path, schema_path: &Path) -> Result<WorldDef, WorldError> {
    let value = json_loader::load_validated(json_path, schema_path)
        .map_err(|e| map_json_error(e, json_path))?;
    serde_json::from_value(value).map_err(|e| WorldError::Parse {
        path: json_path.to_owned(),
        source: e,
    })
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
