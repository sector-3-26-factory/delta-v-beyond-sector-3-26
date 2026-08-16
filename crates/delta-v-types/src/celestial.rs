// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Celestial body types for suns, planets, and asteroids.
//!
//! These are plain data types used for JSON deserialization and shared across
//! crates. Per ADR-0046, this crate MUST NOT contain systems, plugins, or Bevy
//! `Component` derives — only plain types and serde structs.

use serde::Deserialize;

use crate::collision::CollisionShapeData;
use crate::physics::PhysicalQuantityJson;
use crate::spatial::{BoundingBox, BoundingBoxJson};

/// RGB color for light sources.
///
/// Each component is in the range 0.0-1.0.
/// Per ADR-0039, all defaults are in the schema, not in Rust code.
#[derive(Debug, Deserialize, Clone)]
pub struct LightColorJson {
    /// Red component (0.0-1.0).
    pub r: f32,
    /// Green component (0.0-1.0).
    pub g: f32,
    /// Blue component (0.0-1.0).
    pub b: f32,
}

/// JSON schema type for sun templates.
///
/// Deserialized from `assets/templates/suns/*/sun.json`.
/// Per ADR-0008, physical quantities use value+unit format.
/// Per ADR-0039, all defaults are in the schema, not in Rust code.
#[derive(Debug, Deserialize, Clone)]
pub struct SunTemplateJson {
    /// Entity type discriminator. Must be "sun".
    pub entity_type: String,
    /// Mass in kilograms.
    pub mass: PhysicalQuantityJson,
    /// Collision shape for the sun.
    pub collision_shape: super::CollisionShapeJson,
    /// Axis-aligned bounding box of the mesh in sun-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBoxJson,
    /// If true, this body generates gravity on other entities.
    /// Default: true (from schema).
    pub is_gravity_source: bool,
    /// Rotation period in hours.
    pub rotation_period: Option<PhysicalQuantityJson>,
    /// Light intensity in lux.
    /// Default: 10000.0 (from schema).
    pub light_intensity: f32,
    /// Light color as RGB values.
    /// Default: {r: 1.0, g: 0.95, b: 0.8} (from schema).
    pub light_color: LightColorJson,
    /// Light range in metres.
    /// Default: 6e12 (from schema).
    pub light_range: PhysicalQuantityJson,
}

/// JSON schema type for planet templates.
///
/// Deserialized from `assets/templates/planets/*/planet.json`.
/// Per ADR-0008, physical quantities use value+unit format.
/// Per ADR-0039, all defaults are in the schema, not in Rust code.
#[derive(Debug, Deserialize, Clone)]
pub struct PlanetTemplateJson {
    /// Entity type discriminator. Must be "planet".
    pub entity_type: String,
    /// Mass in kilograms.
    pub mass: PhysicalQuantityJson,
    /// Collision shape for the planet.
    pub collision_shape: super::CollisionShapeJson,
    /// Axis-aligned bounding box of the mesh in planet-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBoxJson,
    /// If true, this body generates gravity on other entities.
    /// Default: true (from schema).
    pub is_gravity_source: bool,
    /// If true, mesh animations (e.g., moving clouds) will be played.
    /// Default: true (from schema).
    pub animations_enabled: bool,
}

/// JSON schema type for asteroid templates.
///
/// Deserialized from `assets/templates/asteroids/*/asteroid.json`.
/// Per ADR-0008, physical quantities use value+unit format.
/// Per ADR-0039, all defaults are in the schema, not in Rust code.
#[derive(Debug, Deserialize, Clone)]
pub struct AsteroidTemplateJson {
    /// Entity type discriminator. Must be "asteroid".
    pub entity_type: String,
    /// Mass in kilograms.
    pub mass: PhysicalQuantityJson,
    /// Collision shape for the asteroid.
    pub collision_shape: super::CollisionShapeJson,
    /// Axis-aligned bounding box of the mesh in asteroid-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBoxJson,
    /// If true, this body generates gravity on other entities.
    /// Default: false (from schema).
    pub is_gravity_source: bool,
}

/// JSON schema type for moon templates.
///
/// Deserialized from `assets/templates/moons/*/moon.json`.
/// Per ADR-0008, physical quantities use value+unit format.
/// Per ADR-0039, all defaults are in the schema, not in Rust code.
#[derive(Debug, Deserialize, Clone)]
pub struct MoonTemplateJson {
    /// Entity type discriminator. Must be "moon".
    pub entity_type: String,
    /// Mass in kilograms.
    pub mass: PhysicalQuantityJson,
    /// Collision shape for the moon.
    pub collision_shape: super::CollisionShapeJson,
    /// Axis-aligned bounding box of the mesh in moon-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBoxJson,
    /// If true, this body generates gravity on other entities.
    /// Default: true (from schema).
    pub is_gravity_source: bool,
    /// If true, mesh animations (e.g., moving clouds) will be played.
    /// Default: true (from schema).
    pub animations_enabled: bool,
}

/// Runtime sun template with SI units.
///
/// This is the converted version of [`SunTemplateJson`] with all
/// physical quantities converted to SI base units.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct SunTemplate {
    /// Entity type discriminator ("sun").
    pub entity_type: String,
    /// Mass in kilograms (SI base unit).
    pub mass: f32,
    /// Collision shape data for the sun.
    pub collision_shape: CollisionShapeData,
    /// Axis-aligned bounding box of the mesh in sun-local coordinates (metres).
    pub bounding_box: BoundingBox,
    /// If true, this body generates gravity on other entities.
    pub is_gravity_source: bool,
    /// Rotation period in seconds (SI base unit).
    pub rotation_period: Option<f32>,
    /// Light intensity in lux.
    pub light_intensity: f32,
    /// Light color as RGB values.
    pub light_color: super::LightColorJson,
    /// Light range in metres (SI base unit).
    pub light_range: f32,
}

impl From<SunTemplateJson> for SunTemplate {
    fn from(json: SunTemplateJson) -> Self {
        Self {
            entity_type: json.entity_type,
            mass: json.mass.to_kilograms(),
            collision_shape: json.collision_shape.into(),
            bounding_box: json.bounding_box.into(),
            is_gravity_source: json.is_gravity_source,
            rotation_period: json.rotation_period.map(|p| p.to_seconds()),
            light_intensity: json.light_intensity,
            light_color: json.light_color,
            light_range: json.light_range.to_meters(),
        }
    }
}

/// Runtime planet template with SI units.
///
/// This is the converted version of [`PlanetTemplateJson`] with all
/// physical quantities converted to SI base units.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct PlanetTemplate {
    /// Entity type discriminator ("planet").
    pub entity_type: String,
    /// Mass in kilograms (SI base unit).
    pub mass: f32,
    /// Collision shape data for the planet.
    pub collision_shape: CollisionShapeData,
    /// Axis-aligned bounding box of the mesh in planet-local coordinates (metres).
    pub bounding_box: BoundingBox,
    /// If true, this body generates gravity on other entities.
    pub is_gravity_source: bool,
    /// If true, mesh animations (e.g., moving clouds) will be played.
    pub animations_enabled: bool,
}

impl From<PlanetTemplateJson> for PlanetTemplate {
    fn from(json: PlanetTemplateJson) -> Self {
        Self {
            entity_type: json.entity_type,
            mass: json.mass.to_kilograms(),
            collision_shape: json.collision_shape.into(),
            bounding_box: json.bounding_box.into(),
            is_gravity_source: json.is_gravity_source,
            animations_enabled: json.animations_enabled,
        }
    }
}

/// Runtime asteroid template with SI units.
///
/// This is the converted version of [`AsteroidTemplateJson`] with all
/// physical quantities converted to SI base units.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct AsteroidTemplate {
    /// Entity type discriminator ("asteroid").
    pub entity_type: String,
    /// Mass in kilograms (SI base unit).
    pub mass: f32,
    /// Collision shape data for the asteroid.
    pub collision_shape: CollisionShapeData,
    /// Axis-aligned bounding box of the mesh in asteroid-local coordinates (metres).
    pub bounding_box: BoundingBox,
    /// If true, this body generates gravity on other entities.
    pub is_gravity_source: bool,
}

impl From<AsteroidTemplateJson> for AsteroidTemplate {
    fn from(json: AsteroidTemplateJson) -> Self {
        Self {
            entity_type: json.entity_type,
            mass: json.mass.to_kilograms(),
            collision_shape: json.collision_shape.into(),
            bounding_box: json.bounding_box.into(),
            is_gravity_source: json.is_gravity_source,
        }
    }
}

/// Runtime moon template with SI units.
///
/// This is the converted version of [`MoonTemplateJson`] with all
/// physical quantities converted to SI base units.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct MoonTemplate {
    /// Entity type discriminator ("moon").
    pub entity_type: String,
    /// Mass in kilograms (SI base unit).
    pub mass: f32,
    /// Collision shape data for the moon.
    pub collision_shape: CollisionShapeData,
    /// Axis-aligned bounding box of the mesh in moon-local coordinates (metres).
    pub bounding_box: BoundingBox,
    /// If true, this body generates gravity on other entities.
    pub is_gravity_source: bool,
    /// If true, mesh animations (e.g., moving clouds) will be played.
    pub animations_enabled: bool,
}

impl From<MoonTemplateJson> for MoonTemplate {
    fn from(json: MoonTemplateJson) -> Self {
        Self {
            entity_type: json.entity_type,
            mass: json.mass.to_kilograms(),
            collision_shape: json.collision_shape.into(),
            bounding_box: json.bounding_box.into(),
            is_gravity_source: json.is_gravity_source,
            animations_enabled: json.animations_enabled,
        }
    }
}
