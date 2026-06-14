// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! AI-related resources.

use bevy::prelude::*;

/// Tracks the current skirmish state.
///
/// Used by [`crate::systems::skirmish_check_system`] to determine when the
/// skirmish is over (player dead or all enemies dead).
// allow-default: Bevy requires Default on resources for init_resource.
// This is runtime state, not configuration.
#[derive(Resource, Default, Debug)]
pub struct SkirmishState {
    /// Number of NPC ships still alive.
    pub enemies_alive: usize,
    /// Total number of NPC ships that existed at world load.
    /// If zero, this is not a skirmish world.
    pub total_enemies: usize,
    /// Whether the player ship is still alive.
    pub player_alive: bool,
}
