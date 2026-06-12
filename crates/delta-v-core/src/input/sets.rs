// AGENTS: before modifying this file, read AGENTS.md at the repository root.

use bevy::prelude::*;

/// System set ordering for input systems that run in `FixedUpdate`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSet {
    /// Translate raw key events to [`super::LogicalAction`]s.
    Translate,
    /// Log the active actions at DEBUG level (M1 proof-of-pipeline).
    Log,
}
