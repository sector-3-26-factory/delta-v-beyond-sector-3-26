// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Ship ECS components.
//!
//! See ADR-0005 (plugin architecture).

use bevy::prelude::*;

/// Marker component identifying the player-controlled ship entity.
///
/// At most one entity carries this component at any time. This component
/// is added during ship spawning in response to a `SpawnEntity` event
/// with `entity_type: "local_player_ship"`.
#[derive(Component, Debug)]
pub struct PlayerShip;
