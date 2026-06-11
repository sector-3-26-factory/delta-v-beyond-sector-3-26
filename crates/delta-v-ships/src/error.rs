// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Ship-specific errors.
//!
//! See ADR-0016 (Error handling strategy).

/// Errors that can occur during ship spawning or operation.
#[derive(Debug, thiserror::Error)]
pub enum ShipError {
    /// A required template field is missing or invalid.
    #[error("invalid ship template: {0}")]
    InvalidTemplate(String),

    /// The ship template failed to deserialize.
    #[error("failed to deserialize ship template: {0}")]
    Deserialization(String),
}
