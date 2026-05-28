// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Typed error enum for JSON loading and validation failures.
//!
//! See ADR-0016 (Error handling strategy) and
//! ADR-0038 (Shared JSON utilities crate).

use std::path::PathBuf;

/// Errors that can occur while loading or validating a JSON file.
///
/// All variants carry enough context for a precise, actionable error
/// message (ADR-0013, ADR-0016).
#[derive(Debug, thiserror::Error)]
pub enum JsonError {
    /// A required file could not be read.
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
        /// Path to the JSON data file that failed validation.
        path: PathBuf,
        /// JSON Pointer (RFC 6901) to the failing location.
        pointer: String,
        /// Human-readable description of the violation.
        reason: String,
    },

    /// The schema file itself could not be read, parsed, or compiled.
    #[error("{path}: failed to load schema: {reason}")]
    SchemaLoad {
        /// Path to the schema file.
        path: PathBuf,
        /// Description of what went wrong.
        reason: String,
    },
}