// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Entity template enum for type-safe entity spawning.
//!
//! This enum holds runtime types instead of `serde_json::Value` in the `SpawnEntity` event.
//! This improves type safety and follows the architecture where only `delta-v-json` and
//! `delta-v-assets` deal with JSON types.
//!
//! See ADR-0038 (entity template system) and ADR-0046 (shared types crate).

use crate::celestial::{AsteroidTemplate, PlanetTemplate, SunTemplate};
use crate::main_thruster::MainThrusterDefinition;
use crate::maneuvering_thruster::ManeuveringThrusterDefinition;
use crate::ship_templates::{AiShipTemplate, PlayerShipTemplate, ShipTemplate, StaticShipTemplate};
use crate::weapons::{ProjectileDefinition, WeaponTemplate};

/// Entity template enum holding runtime types for all entity types.
///
/// This enum replaces the `serde_json::Value` field in `SpawnEntity` with a type-safe
/// variant that contains the already-converted runtime template struct.
/// All unit conversions (ADR-0008) happen in `delta-v-assets` at load time.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum EntityTemplate {
    /// Sun template with SI units.
    Sun(SunTemplate),
    /// Planet template with SI units.
    Planet(PlanetTemplate),
    /// Asteroid template with SI units.
    Asteroid(AsteroidTemplate),
    /// Base ship template with SI units.
    Ship(ShipTemplate),
    /// Player-controlled ship template with SI units.
    PlayerShip(PlayerShipTemplate),
    /// AI-controlled ship template with SI units.
    AiShip(AiShipTemplate),
    /// Static ship template with SI units.
    StaticShip(StaticShipTemplate),
    /// Weapon template with SI units.
    Weapon(WeaponTemplate),
    /// Projectile definition with SI units.
    Projectile(ProjectileDefinition),
    /// Main thruster definition with SI units.
    MainThruster(MainThrusterDefinition),
    /// Maneuvering thruster definition with SI units.
    ManeuveringThruster(ManeuveringThrusterDefinition),
}

impl EntityTemplate {
    /// Returns the entity type string for this template.
    #[must_use]
    pub const fn entity_type(&self) -> &'static str {
        match self {
            Self::Sun(_) => "sun",
            Self::Planet(_) => "planet",
            Self::Asteroid(_) => "asteroid",
            Self::Ship(_) | Self::StaticShip(_) => "ship",
            Self::PlayerShip(_) => "player_controlled_ship",
            Self::AiShip(_) => "ai_controlled_ship",
            Self::Weapon(_) => "weapon",
            Self::Projectile(_) => "projectile",
            Self::MainThruster(_) => "main_thruster",
            Self::ManeuveringThruster(_) => "maneuvering_thruster",
        }
    }

    /// Returns true if this template is a ship type (player, AI, or static).
    #[must_use]
    pub const fn is_ship(&self) -> bool {
        matches!(
            self,
            Self::Ship(_) | Self::PlayerShip(_) | Self::AiShip(_) | Self::StaticShip(_)
        )
    }

    /// Returns true if this template is a celestial body (sun, planet, asteroid).
    #[must_use]
    pub const fn is_celestial(&self) -> bool {
        matches!(self, Self::Sun(_) | Self::Planet(_) | Self::Asteroid(_))
    }

    /// Returns the mass in kilograms for this template.
    #[must_use]
    pub const fn mass(&self) -> f32 {
        match self {
            Self::Sun(t) => t.mass,
            Self::Planet(t) => t.mass,
            Self::Asteroid(t) => t.mass,
            Self::Ship(t) => t.mass,
            Self::PlayerShip(t) => t.mass,
            Self::AiShip(t) => t.mass,
            Self::StaticShip(t) => t.mass,
            Self::Weapon(_)
            | Self::Projectile(_)
            | Self::MainThruster(_)
            | Self::ManeuveringThruster(_) => 0.0,
        }
    }

    /// Returns the collision shape data for this template.
    #[must_use]
    pub fn collision_shape(&self) -> &crate::collision::CollisionShapeData {
        match self {
            Self::Sun(t) => &t.collision_shape,
            Self::Planet(t) => &t.collision_shape,
            Self::Asteroid(t) => &t.collision_shape,
            Self::Ship(t) => &t.collision_shape,
            Self::PlayerShip(t) => &t.collision_shape,
            Self::AiShip(t) => &t.collision_shape,
            Self::StaticShip(t) => &t.collision_shape,
            Self::Weapon(_)
            | Self::Projectile(_)
            | Self::MainThruster(_)
            | Self::ManeuveringThruster(_) => {
                static EMPTY: crate::collision::CollisionShapeData =
                    crate::collision::CollisionShapeData {
                        shape_type: crate::collision::CollisionShapeType::Sphere { radius: 0.0 },
                        offset: crate::spatial::Vec3::ZERO,
                    };
                &EMPTY
            }
        }
    }

    /// Returns the bounding box for this template.
    #[must_use]
    pub fn bounding_box(&self) -> &crate::spatial::BoundingBox {
        match self {
            Self::Sun(t) => &t.bounding_box,
            Self::Planet(t) => &t.bounding_box,
            Self::Asteroid(t) => &t.bounding_box,
            Self::Ship(t) => &t.bounding_box,
            Self::PlayerShip(t) => &t.bounding_box,
            Self::AiShip(t) => &t.bounding_box,
            Self::StaticShip(t) => &t.bounding_box,
            Self::Weapon(_)
            | Self::Projectile(_)
            | Self::MainThruster(_)
            | Self::ManeuveringThruster(_) => {
                static EMPTY: crate::spatial::BoundingBox = crate::spatial::BoundingBox {
                    min: crate::spatial::Vec3::ZERO,
                    max: crate::spatial::Vec3::ZERO,
                };
                &EMPTY
            }
        }
    }

    /// Returns the health for this template (for ships and celestial bodies).
    #[must_use]
    pub const fn health(&self) -> f32 {
        match self {
            Self::Ship(t) => t.health,
            Self::PlayerShip(t) => t.health,
            Self::AiShip(t) => t.health,
            Self::StaticShip(t) => t.health,
            Self::Sun(t) => t.mass,      // Suns use mass as health proxy
            Self::Planet(t) => t.mass,   // Planets use mass as health proxy
            Self::Asteroid(t) => t.mass, // Asteroids use mass as health proxy
            Self::Weapon(_)
            | Self::Projectile(_)
            | Self::MainThruster(_)
            | Self::ManeuveringThruster(_) => 0.0,
        }
    }

    /// Returns the weapons list for this template (for ships).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn weapons(&self) -> &[String] {
        match self {
            Self::Ship(t) => &t.weapons,
            Self::PlayerShip(t) => &t.weapons,
            Self::AiShip(t) => &t.weapons,
            Self::StaticShip(t) => &t.weapons,
            Self::Sun(_)
            | Self::Planet(_)
            | Self::Asteroid(_)
            | Self::Weapon(_)
            | Self::Projectile(_)
            | Self::MainThruster(_)
            | Self::ManeuveringThruster(_) => &[],
        }
    }

    /// Returns the inertia scale for this template (for ships).
    #[must_use]
    pub const fn inertia_scale(&self) -> f32 {
        match self {
            Self::Ship(t) => t.inertia_scale,
            Self::PlayerShip(t) => t.inertia_scale,
            Self::AiShip(t) => t.inertia_scale,
            Self::StaticShip(t) => t.inertia_scale,
            Self::Sun(_)
            | Self::Planet(_)
            | Self::Asteroid(_)
            | Self::Weapon(_)
            | Self::Projectile(_)
            | Self::MainThruster(_)
            | Self::ManeuveringThruster(_) => 1.0,
        }
    }

    /// Returns the propulsion config for this template (for ships).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn propulsion(&self) -> Option<&crate::propulsion::ShipPropulsionTemplate> {
        match self {
            Self::Ship(t) => Some(&t.propulsion),
            Self::PlayerShip(t) => Some(&t.propulsion),
            Self::AiShip(t) => Some(&t.propulsion),
            Self::StaticShip(t) => Some(&t.propulsion),
            Self::Sun(_)
            | Self::Planet(_)
            | Self::Asteroid(_)
            | Self::Weapon(_)
            | Self::Projectile(_)
            | Self::MainThruster(_)
            | Self::ManeuveringThruster(_) => None,
        }
    }

    /// Returns the AI config for this template (for AI ships).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn ai_config(&self) -> Option<&crate::ai::AiConfig> {
        match self {
            Self::AiShip(t) => Some(&t.ai),
            _ => None,
        }
    }

    /// Returns the cameras for this template (for player ships).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cameras(&self) -> Option<&crate::ship_templates::ShipCamerasTemplate> {
        match self {
            Self::PlayerShip(t) => Some(&t.cameras),
            _ => None,
        }
    }

    /// Returns the cockpit definition for this template (for player ships).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cockpit(&self) -> Option<&crate::ship_templates::CockpitDefinition> {
        match self {
            Self::PlayerShip(t) => Some(&t.cockpit),
            _ => None,
        }
    }

    /// Returns the max weapons count for this template (for ships).
    #[must_use]
    pub const fn max_weapons_count(&self) -> usize {
        match self {
            Self::Ship(t) => t.max_weapons_count,
            Self::PlayerShip(t) => t.max_weapons_count,
            Self::AiShip(t) => t.max_weapons_count,
            Self::StaticShip(t) => t.max_weapons_count,
            _ => 0,
        }
    }

    /// Returns the max propulsions count for this template (for ships).
    #[must_use]
    pub const fn max_propulsions_count(&self) -> usize {
        match self {
            Self::Ship(t) => t.max_propulsions_count,
            Self::PlayerShip(t) => t.max_propulsions_count,
            Self::AiShip(t) => t.max_propulsions_count,
            Self::StaticShip(t) => t.max_propulsions_count,
            _ => 0,
        }
    }

    /// Returns the main thruster names for this template (for ships).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn main_thruster_names(&self) -> &[String] {
        match self {
            Self::Ship(t) => &t.propulsion.main_thruster_names,
            Self::PlayerShip(t) => &t.propulsion.main_thruster_names,
            Self::AiShip(t) => &t.propulsion.main_thruster_names,
            Self::StaticShip(t) => &t.propulsion.main_thruster_names,
            _ => &[],
        }
    }

    /// Returns the maneuvering thruster name for this template (for ships).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn maneuvering_thruster_name(&self) -> &str {
        match self {
            Self::Ship(t) => &t.propulsion.maneuvering_thruster,
            Self::PlayerShip(t) => &t.propulsion.maneuvering_thruster,
            Self::AiShip(t) => &t.propulsion.maneuvering_thruster,
            Self::StaticShip(t) => &t.propulsion.maneuvering_thruster,
            _ => "",
        }
    }

    /// Returns the `is_gravity_source` flag for this template (for celestial bodies).
    #[must_use]
    pub const fn is_gravity_source(&self) -> bool {
        match self {
            Self::Sun(t) => t.is_gravity_source,
            Self::Planet(t) => t.is_gravity_source,
            Self::Asteroid(t) => t.is_gravity_source,
            _ => false,
        }
    }

    /// Returns the light intensity for this template (for suns).
    #[must_use]
    pub const fn light_intensity(&self) -> f32 {
        match self {
            Self::Sun(t) => t.light_intensity,
            _ => 0.0,
        }
    }

    /// Returns the light color for this template (for suns).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn light_color(&self) -> &crate::celestial::LightColorJson {
        if let Self::Sun(t) = self {
            &t.light_color
        } else {
            static DEFAULT: crate::celestial::LightColorJson = crate::celestial::LightColorJson {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            };
            &DEFAULT
        }
    }

    /// Returns the light range for this template (for suns).
    #[must_use]
    pub const fn light_range(&self) -> f32 {
        match self {
            Self::Sun(t) => t.light_range,
            _ => 0.0,
        }
    }

    /// Returns the rotation period for this template (for suns).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn rotation_period(&self) -> Option<f32> {
        match self {
            Self::Sun(t) => t.rotation_period,
            _ => None,
        }
    }

    /// Returns the orbital parent for this template (for planets).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn orbital_parent(&self) -> Option<&str> {
        match self {
            Self::Planet(t) => Some(&t.orbital_parent),
            _ => None,
        }
    }

    /// Returns the orbital distance for this template (for planets).
    #[must_use]
    pub const fn orbital_distance(&self) -> f32 {
        match self {
            Self::Planet(t) => t.orbital_distance,
            _ => 0.0,
        }
    }

    /// Returns the orbital period for this template (for planets).
    #[must_use]
    pub const fn orbital_period(&self) -> f32 {
        match self {
            Self::Planet(t) => t.orbital_period,
            _ => 0.0,
        }
    }

    /// Returns the orbital eccentricity for this template (for planets).
    #[must_use]
    pub const fn orbital_eccentricity(&self) -> f32 {
        match self {
            Self::Planet(t) => t.orbital_eccentricity,
            _ => 0.0,
        }
    }

    /// Returns the orbital inclination for this template (for planets).
    #[must_use]
    pub const fn orbital_inclination(&self) -> f32 {
        match self {
            Self::Planet(t) => t.orbital_inclination,
            _ => 0.0,
        }
    }

    /// Returns the initial orbital angle for this template (for planets).
    #[must_use]
    pub const fn initial_orbital_angle(&self) -> f32 {
        match self {
            Self::Planet(t) => t.initial_orbital_angle,
            _ => 0.0,
        }
    }
}
