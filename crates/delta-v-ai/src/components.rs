// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! AI components for driven entities.

use bevy::prelude::*;

/// Marker component for AI-driven NPC entities.
///
/// Used for skirmish tracking (counting alive enemies) and to distinguish
/// AI-driven entities from static ones.
#[derive(Component, Debug, Clone)]
pub struct NpcShip {
    /// Unique identifier for this NPC instance.
    pub entity_id: String,
}

/// Current state of the AI state machine.
///
/// The AI cycles through these states based on the assigned [`AiTask`],
/// distance to targets, and health.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiState {
    /// Wandering near a point, no target engaged.
    Patrol,
    /// Moving toward a target to engage.
    Pursue,
    /// Engaging a target with weapons.
    Attack,
    /// Retreating from combat due to low health.
    Flee,
}

/// AI behavioral parameters deserialized from the ship template.
///
/// These values depend on the ship type -- a fighter has different
/// parameters than a cargo ship.
#[derive(Component, Debug, Clone)]
pub struct AiConfig {
    /// Distance at which the AI becomes hostile and pursues.
    pub aggro_range: f32,
    /// Distance at which the AI starts attacking.
    pub attack_range: f32,
    /// Maximum distance from patrol point before returning.
    pub leash_range: f32,
    /// Health fraction (0-1) below which the AI flees.
    pub flee_health_threshold: f32,
    /// Radius for patrol wander behavior.
    pub patrol_radius: f32,
}

/// The mission assigned to this specific AI entity instance.
///
/// Unlike [`AiConfig`] (which is per ship type), this is per entity --
/// two ships of the same type can have different tasks.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiTask {
    /// Wander near spawn point, engage hostiles that come within `aggro_range`.
    Patrol,
}
