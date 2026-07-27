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

/// Runtime AI configuration with SI units.
///
/// This is the converted version of [`AiConfigJson`] with all
/// physical quantities converted to SI base units (metres).
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct AiConfig {
    /// Distance at which the AI becomes hostile and pursues (metres, SI base unit).
    pub aggro_range: f32,
    /// Distance at which the AI starts attacking (metres, SI base unit).
    pub attack_range: f32,
    /// Maximum distance from patrol point before returning (metres, SI base unit).
    pub leash_range: f32,
    /// Health fraction (0-1) below which the AI flees.
    pub flee_health_threshold: f32,
    /// Radius for patrol wander behaviour (metres, SI base unit).
    pub patrol_radius: f32,
}

impl From<AiConfigJson> for AiConfig {
    fn from(json: AiConfigJson) -> Self {
        Self {
            aggro_range: json.aggro_range.to_meters(),
            attack_range: json.attack_range.to_meters(),
            leash_range: json.leash_range.to_meters(),
            // INVARIANT: flee_health_threshold is a health fraction (0-1), f64->f32
            // truncation is acceptable for this gameplay value.
            #[allow(clippy::cast_possible_truncation)]
            flee_health_threshold: json.flee_health_threshold as f32,
            patrol_radius: json.patrol_radius.to_meters(),
        }
    }
}

/// Runtime AI task assigned to a specific entity instance.
///
/// This is the converted version of [`AiTaskJson`] for use at runtime.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiTask {
    /// Wander near spawn point, engage hostiles that come within `aggro_range`.
    Patrol,
}

impl From<AiTaskJson> for AiTask {
    fn from(json: AiTaskJson) -> Self {
        match json {
            AiTaskJson::Patrol => Self::Patrol,
        }
    }
}
