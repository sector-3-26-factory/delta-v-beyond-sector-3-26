// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Weapon configuration resources.

use bevy::prelude::*;

/// Resource tracking the player's weapon firing state.
///
/// Used for edge detection — fire on press, not on hold.
/// Per ADR-0014, this is runtime state, not configuration.
// allow-default: Bevy requires Default on resources for init_resource.
// This is per-tick state, not configuration.
#[derive(Resource, Default, Debug)]
pub struct WeaponState {
    /// Whether the fire button was held last tick (for edge detection).
    pub fire_held_prev: bool,
}
