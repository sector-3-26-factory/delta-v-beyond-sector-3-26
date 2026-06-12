// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Weapon and projectile template types.

use serde::Deserialize;

use crate::physics::PhysicalQuantity;

/// Weapon template deserialized from ship template JSON.
///
/// Defines a weapon that can be mounted on a ship. All gameplay values
/// (speed, damage, fire rate, lifetime, radius) come from JSON per ADR-0014.
#[derive(Debug, Deserialize, Clone)]
pub struct WeaponTemplateJson {
    /// Weapon type identifier (e.g., "laser", "railgun").
    #[serde(rename = "type")]
    pub weapon_type: String,
    /// Projectile speed in m/s.
    pub projectile_speed: PhysicalQuantity,
    /// Damage per hit.
    pub damage: PhysicalQuantity,
    /// Fire rate in rounds per second.
    pub fire_rate: PhysicalQuantity,
    /// Projectile lifetime in seconds.
    pub lifetime: PhysicalQuantity,
    /// Projectile collision shape radius in metres.
    pub projectile_radius: PhysicalQuantity,
}
