// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Weapon and projectile components.

use bevy::prelude::*;

/// Component marking an entity as a projectile.
///
/// Projectiles are spawned by the weapons system and inherit the source
/// entity's velocity plus the weapon's projectile speed in the forward
/// direction (per ADR-0006: -Z is forward).
#[derive(Component, Debug, Clone)]
pub struct Projectile {
    /// The entity that fired this projectile.
    pub source: Entity,
    /// Remaining lifetime in seconds.
    pub lifetime: f32,
    /// Damage to apply on hit.
    pub damage: f32,
    /// Sound file path relative to assets/audio/ (e.g., "hit.mp3").
    /// Optional; if not provided, no sound is played on impact.
    pub hit_sound: Option<String>,
}
