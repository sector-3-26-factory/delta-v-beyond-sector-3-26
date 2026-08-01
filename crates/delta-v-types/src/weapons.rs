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

/// Runtime weapon template with SI units.
///
/// This is the converted version of [`WeaponTemplateJson`] with all
/// physical quantities converted to SI base units.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct WeaponTemplate {
    /// Weapon type identifier (e.g., "laser", "railgun").
    pub weapon_type: String,
    /// Name of the projectile template to use (e.g., "laser-standard").
    pub projectile_template: String,
    /// Fire rate in Hertz (SI base unit).
    pub fire_rate: f32,
}

impl From<WeaponTemplateJson> for WeaponTemplate {
    fn from(json: WeaponTemplateJson) -> Self {
        Self {
            weapon_type: json.weapon_type,
            projectile_template: json.projectile_template,
            fire_rate: json.fire_rate.to_hertz(),
        }
    }
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

/// Runtime projectile definition with SI units.
///
/// This is the converted version of [`ProjectileDefinitionJson`] with all
/// physical quantities converted to SI base units.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct ProjectileDefinition {
    /// Projectile speed in metres per second (SI base unit).
    pub speed: f32,
    /// Damage per hit in hit points.
    pub damage: f32,
    /// Projectile lifetime in seconds (SI base unit).
    pub lifetime: f32,
    /// Collision shape radius in metres (SI base unit).
    pub radius: f32,
}

impl From<ProjectileDefinitionJson> for ProjectileDefinition {
    fn from(json: ProjectileDefinitionJson) -> Self {
        Self {
            speed: json.speed.to_meters_per_second(),
            damage: json.damage.to_hit_points(),
            lifetime: json.lifetime.to_seconds(),
            radius: json.radius.to_meters(),
        }
    }
}
