// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! AI configuration and task types.

use serde::Deserialize;

use crate::physics::PhysicalQuantityJson;

/// AI behavioral parameters deserialized from the ship template JSON.
///
/// These values depend on the ship type -- a fighter has different
/// parameters than a cargo ship. All physical quantities use the
/// `PhysicalQuantityJson` value+unit pattern (ADR-0008).
#[derive(Debug, Deserialize, Clone)]
pub struct AiConfigJson {
    /// Distance at which the AI becomes hostile and pursues (metres).
    pub aggro_range: PhysicalQuantityJson,
    /// Distance at which the AI starts attacking (metres).
    pub attack_range: PhysicalQuantityJson,
    /// Maximum distance from patrol point before returning (metres).
    pub leash_range: PhysicalQuantityJson,
    /// Health fraction (0-1) below which the AI flees.
    pub flee_health_threshold: f64,
    /// Radius for patrol wander behaviour (metres).
    pub patrol_radius: PhysicalQuantityJson,
}

/// AI task assigned to a specific entity instance in the world definition.
///
/// Unlike [`AiConfigJson`] (which is per ship type), this is per entity --
/// two ships of the same type can have different tasks.
#[derive(Debug, Deserialize, Clone)]
pub enum AiTaskJson {
    /// Wander near spawn point, engage hostiles that come within `aggro_range`.
    #[serde(rename = "patrol")]
    Patrol,
}
