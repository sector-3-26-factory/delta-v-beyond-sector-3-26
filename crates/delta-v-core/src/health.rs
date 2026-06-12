// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Health and weapon components for combat.

use bevy::prelude::*;

/// Component for entities that can take damage.
///
/// Applied to ships, asteroids, and other destructible entities.
/// When health reaches zero, the entity is considered destroyed.
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    /// Current health points.
    pub current: f32,
    /// Maximum health points.
    pub max: f32,
}

impl Health {
    /// Creates a new Health component with full health.
    #[must_use]
    pub const fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    /// Applies damage to this health component.
    ///
    /// Clamps current health to a minimum of 0.0.
    /// Returns `true` if the entity is destroyed (health reaches 0).
    pub fn apply_damage(&mut self, damage: f32) -> bool {
        self.current = (self.current - damage).max(0.0);
        self.current <= 0.0
    }

    /// Returns `true` if the entity is destroyed.
    #[must_use]
    pub const fn is_destroyed(&self) -> bool {
        self.current <= 0.0
    }
}

/// Component marking an entity as having a weapon.
///
/// Contains runtime weapon state (cooldown) and configuration values
/// read from the weapon template JSON at spawn time (per ADR-0014).
#[derive(Component, Debug, Clone, Copy)]
pub struct Weapon {
    /// Index of this weapon slot.
    pub slot: u32,
    /// Cooldown remaining in seconds.
    pub cooldown: f32,
    /// Projectile speed in m/s.
    pub projectile_speed: f32,
    /// Damage per hit.
    pub damage: f32,
    /// Fire rate in rounds per second.
    pub fire_rate: f32,
    /// Projectile lifetime in seconds.
    pub lifetime: f32,
    /// Projectile collision radius in metres.
    pub projectile_radius: f32,
}
