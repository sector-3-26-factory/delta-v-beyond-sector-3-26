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

//! Typed error enum for configuration loading failures.
//!
//! See ADR-0016 (Error handling strategy).

use std::path::PathBuf;

/// Errors that can occur while loading or validating configuration files.
///
/// All variants carry enough context to produce a precise, actionable
/// error message (ADR-0013, ADR-0016).
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A required configuration file could not be read.
    #[error("{path}: file not found or unreadable: {source}")]
    Io {
        /// Path to the file that failed.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// The file contents could not be parsed as JSON.
    #[error("{path}: JSON parse error: {source}")]
    Parse {
        /// Path to the file that failed.
        path: PathBuf,
        /// Underlying parse error.
        source: serde_json::Error,
    },

    /// The parsed JSON did not satisfy its JSON Schema.
    #[error("{path}: schema validation failed at '{pointer}': {reason}")]
    Schema {
        /// Path to the JSON file that failed validation.
        path: PathBuf,
        /// JSON Pointer (RFC 6901) to the failing location.
        pointer: String,
        /// Human-readable description of the violation.
        reason: String,
    },

    /// The schema file itself could not be read or parsed.
    #[error("{path}: failed to load schema: {reason}")]
    SchemaLoad {
        /// Path to the schema file.
        path: PathBuf,
        /// Description of what went wrong.
        reason: String,
    },

    /// A physical quantity unit is not in the allowed units registry.
    #[error(
        "{path}: invalid unit '{unit}' at '{pointer}': not in allowed units registry ({units_schema})"
    )]
    InvalidUnit {
        /// Path to the JSON data file.
        path: PathBuf,
        /// JSON Pointer to the failing value+unit object.
        pointer: String,
        /// The invalid unit string.
        unit: String,
        /// Path to the units schema file.
        units_schema: PathBuf,
    },
}

impl ConfigError {
    /// Converts a `delta_v_json::JsonError` into a `ConfigError`.
    ///
    /// This is used when the context (file path) is already known.
    pub fn from_json_error(e: delta_v_json::JsonError, _path: &PathBuf) -> Self {
        match e {
            delta_v_json::JsonError::Io { path, source } => Self::Io { path, source },
            delta_v_json::JsonError::Parse { path, source } => Self::Parse { path, source },
            delta_v_json::JsonError::Schema {
                path,
                pointer,
                reason,
            } => Self::Schema {
                path,
                pointer,
                reason,
            },
            delta_v_json::JsonError::SchemaLoad { path, reason } => {
                Self::SchemaLoad { path, reason }
            }
            delta_v_json::JsonError::InvalidUnit {
                path,
                pointer,
                unit,
                units_schema,
            } => Self::InvalidUnit {
                path,
                pointer,
                unit,
                units_schema,
            },
        }
    }
}
