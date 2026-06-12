// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Weapon and projectile components.

use bevy::prelude::*;

/// Component marking an entity as a projectile.
///
/// Projectiles are spawned by the weapons system and inherit the source
/// entity's velocity plus the weapon's projectile speed in the forward
/// direction (per ADR-0006: -Z is forward).
#[derive(Component, Debug, Clone, Copy)]
pub struct Projectile {
    /// The entity that fired this projectile.
    pub source: Entity,
    /// Remaining lifetime in seconds.
    pub lifetime: f32,
    /// Damage to apply on hit.
    pub damage: f32,
}
