// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Health component for combat.

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

/// Marker component for entities that can be targeted in combat.
///
/// These entities appear in the targeting list and can be selected as the
/// player's current target.
#[derive(Component, Debug, Clone, Copy)]
pub struct Targetable;
