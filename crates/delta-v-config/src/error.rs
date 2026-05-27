// AGENTS: before modifying this file, read AGENTS.md at the repository root.

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
}