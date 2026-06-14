// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Typed errors for the AI crate.

use thiserror::Error;

/// Errors that can occur in AI systems.
#[derive(Debug, Error)]
pub enum AiError {
    /// The AI template failed to deserialize.
    #[error("failed to deserialize AI template: {0}")]
    Deserialization(String),

    /// Missing required field in AI configuration.
    #[error("missing required AI config field: {0}")]
    MissingField(String),
}
