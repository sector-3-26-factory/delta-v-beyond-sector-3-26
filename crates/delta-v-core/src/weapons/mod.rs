// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Weapon component for combat.

use bevy::prelude::*;

/// Component marking an entity as having a weapon.
///
/// Contains runtime weapon state (cooldown) and configuration values
/// read from the weapon template JSON at spawn time (per ADR-0014).
#[derive(Component, Debug, Clone)]
pub struct Weapon {
    /// Index of this weapon slot.
    pub slot: u32,
    /// Cooldown remaining in seconds.
    pub cooldown: f32,
    /// Name of the weapon definition (e.g., "laser-standard").
    pub weapon_name: String,
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
    /// Pre-resolved fire sound file path relative to assets/ (e.g., "audio/fire.mp3" or "components/weapons/laser-standard/fire.mp3").
    /// None if no fire sound.
    pub fire_sound: Option<String>,
    /// Pre-resolved hit sound file path relative to assets/ (e.g., "audio/hit.mp3" or "components/projectiles/laser-standard/hit.mp3").
    /// None if no hit sound.
    pub hit_sound: Option<String>,
}
