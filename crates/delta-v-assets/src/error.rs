// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Asset loading errors.
//!
//! See ADR-0016 (Error handling strategy).

use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur during asset loading.
#[derive(Debug, Error)]
pub enum AssetError {
    /// Template file not found.
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Failed to parse JSON.
    #[error("{path}: JSON parse error: {source}")]
    JsonParse {
        /// Path to the file that failed.
        path: PathBuf,
        /// Underlying parse error.
        source: serde_json::Error,
    },

    /// Validation error.
    #[error("{0}")]
    Validation(String),

    /// Schema file could not be loaded.
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

    /// I/O error.
    #[error("{path}: I/O error: {source}")]
    Io {
        /// Path to the file that failed.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
}
