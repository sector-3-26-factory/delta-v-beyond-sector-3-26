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

//! Celestial body types for suns and planets.
//!
//! These are plain data types used for JSON deserialization and shared across
//! crates. Per ADR-0046, this crate MUST NOT contain systems, plugins, or Bevy
//! `Component` derives — only plain types and serde structs.

use serde::Deserialize;

use crate::physics::PhysicalQuantityJson;

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
    pub bounding_box: super::BoundingBoxJson,
    /// If true, this body generates gravity on other entities.
    /// Default: true (from schema).
    pub is_gravity_source: bool,
    /// Rotation period in hours.
    pub rotation_period: Option<PhysicalQuantityJson>,
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
    pub bounding_box: super::BoundingBoxJson,
    /// If true, this body generates gravity on other entities.
    /// Default: true (from schema).
    pub is_gravity_source: bool,
    /// ID of the parent body this planet orbits.
    pub orbital_parent: String,
    /// Orbital distance in metres.
    pub orbital_distance: PhysicalQuantityJson,
    /// Orbital period in seconds.
    pub orbital_period: PhysicalQuantityJson,
    /// Orbital eccentricity (0 = circular, 0.1-0.9 = increasingly elliptical).
    /// Default: 0.0 (from schema).
    pub orbital_eccentricity: f32,
    /// Orbital inclination in degrees.
    /// Default: 0.0 (from schema).
    pub orbital_inclination: PhysicalQuantityJson,
    /// Initial orbital angle in degrees.
    /// Default: 0.0 (from schema).
    pub initial_orbital_angle: PhysicalQuantityJson,
}
