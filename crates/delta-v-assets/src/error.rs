// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Asset loading errors.

use thiserror::Error;

/// Errors that can occur during asset loading.
#[derive(Debug, Error)]
pub enum AssetError {
    /// Template file not found.
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Failed to parse JSON.
    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),
}
