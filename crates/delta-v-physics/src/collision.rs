// See AGENTS.md
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

//! Collision detection and response using avian3d.
//!
//! Per M3 requirements, this module provides collision shapes and
//! collision layers for static obstacles (asteroids) and dynamic bodies
//! (ships).
//!
//! See ADR-0009 (Newtonian physics with gravity) for the physics model.
//! See ADR-0044 for the prohibition on visual data in Rust code.

use bevy::prelude::*;

/// Collision shape types for physics bodies.
#[derive(Component, Debug, Clone, Copy)]
pub struct CollisionShape {
    /// The shape type and its parameters.
    pub shape_type: CollisionShapeType,
}

/// Supported collision shape types.
#[derive(Debug, Clone, Copy)]
pub enum CollisionShapeType {
    /// A sphere with a given radius.
    Sphere {
        /// Radius of the sphere in metres.
        radius: f32,
    },
    /// A box with half-extents (width/2, height/2, depth/2).
    Box {
        /// Half-extents of the box in metres.
        half_extents: Vec3,
    },
    /// A convex hull (not yet implemented).
    ConvexHull,
}

impl CollisionShape {
    /// Creates a new sphere collision shape.
    #[must_use]
    pub const fn sphere(radius: f32) -> Self {
        Self {
            shape_type: CollisionShapeType::Sphere { radius },
        }
    }

    /// Creates a new box collision shape.
    #[must_use]
    pub const fn box_shape(half_extents: Vec3) -> Self {
        Self {
            shape_type: CollisionShapeType::Box { half_extents },
        }
    }
}

/// Collision layers for categorizing entities.
///
/// Per avian3d conventions, we use a u32 bitmask where each bit
/// represents a layer. Entities on different layers don't collide
/// unless their masks overlap.
#[derive(Debug, Clone, Copy, Default)]
pub struct CollisionLayers {
    /// Which layers this entity belongs to.
    pub layers: u32,
    /// Which layers this entity can collide with.
    pub mask: u32,
}

impl CollisionLayers {
    /// Creates new collision layers.
    #[must_use]
    pub const fn new(layers: u32, mask: u32) -> Self {
        Self { layers, mask }
    }
}

/// Layer constants for collision categories.
#[allow(clippy::mixed_attributes_style)]
pub mod layers {
    //! Collision layer constants.

    /// Layer for ships (player and NPCs).
    ///
    /// Ships are on layer 1 and can collide with asteroids (`ASTEROID_LAYER`).
    pub const SHIP_LAYER: u32 = 1;

    /// Layer for asteroids (static obstacles).
    ///
    /// Asteroids are on layer 2 and can collide with ships (`SHIP_LAYER`).
    pub const ASTEROID_LAYER: u32 = 2;

    /// Collision layers for ships: on `SHIP_LAYER`, can collide with `ASTEROID_LAYER`.
    pub const SHIP: super::CollisionLayers =
        super::CollisionLayers::new(SHIP_LAYER, ASTEROID_LAYER);

    /// Collision layers for asteroids: on `ASTEROID_LAYER`, can collide with `SHIP_LAYER`.
    pub const ASTEROID: super::CollisionLayers =
        super::CollisionLayers::new(ASTEROID_LAYER, SHIP_LAYER);
}

/// Event emitted when a collision is detected.
#[derive(Event, Debug)]
pub struct CollisionDetected {
    /// The entity that was hit.
    pub target: Entity,
    /// The entity that caused the collision (may be the same for static objects).
    pub other: Entity,
    /// The point of impact in world coordinates.
    pub point: Vec3,
    /// The normal at the point of impact.
    pub normal: Vec3,
    /// The relative velocity at impact.
    pub relative_velocity: Vec3,
}

/// Marker component for static (non-moving) collision objects.
#[derive(Component, Debug, Clone, Copy)]
pub struct StaticBody;

/// Marker component for dynamic (moving) collision objects.
#[derive(Component, Debug, Clone, Copy)]
pub struct DynamicBody;
