// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Weapon and projectile template types.

use serde::Deserialize;

use crate::physics::PhysicalQuantityJson;

/// Weapon template deserialized from weapon definition JSON.
///
/// Defines a weapon that can be mounted on a ship. References a projectile
/// template by name. All gameplay values come from JSON per ADR-0014.
#[derive(Debug, Deserialize, Clone)]
pub struct WeaponTemplateJson {
    /// Weapon type identifier (e.g., "laser", "railgun").
    #[serde(rename = "type")]
    pub weapon_type: String,
    /// Name of the projectile template to use (e.g., "laser-standard").
    pub projectile_template: String,
    /// Fire rate in rounds per second.
    pub fire_rate: PhysicalQuantityJson,
}

/// Projectile template deserialized from projectile definition JSON.
///
/// Defines a projectile entity that is spawned when a weapon fires.
/// All gameplay values come from JSON per ADR-0014.
#[derive(Debug, Deserialize, Clone)]
pub struct ProjectileDefinitionJson {
    /// Projectile speed in m/s.
    pub speed: PhysicalQuantityJson,
    /// Damage per hit.
    pub damage: PhysicalQuantityJson,
    /// Projectile lifetime in seconds.
    pub lifetime: PhysicalQuantityJson,
    /// Collision shape radius in metres.
    pub radius: PhysicalQuantityJson,
}
