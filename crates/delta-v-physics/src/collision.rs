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

//! Collision detection and response using avian3d.
//!
//! Per M3 requirements, this module provides collision shapes and
//! collision layers for static obstacles (asteroids) and dynamic bodies
//! (ships).
//!
//! See ADR-0009 (Newtonian physics with gravity) for the physics model.
//! See ADR-0044 for the prohibition on visual data in Rust code.

use std::ops::Deref;

use bevy::prelude::*;

pub use delta_v_types::{CollisionLayers, CollisionShapeData, CollisionShapeType};

/// Collision shape component for physics bodies.
///
/// This is a newtype wrapper around [`CollisionShapeData`] (from `delta-v-types`)
/// that adds the Bevy `Component` derive. Implements [`Deref`] for transparent
/// access to the inner data — code using `CollisionShape` can access
/// `shape_type` and `offset` fields directly.
#[derive(Component, Debug, Clone, Copy)]
pub struct CollisionShape(pub CollisionShapeData);

impl Deref for CollisionShape {
    type Target = CollisionShapeData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CollisionShape {
    /// Creates a new sphere collision shape.
    #[must_use]
    pub const fn sphere(radius: f32, offset: Vec3) -> Self {
        Self(CollisionShapeData::sphere(radius, offset))
    }

    /// Creates a new box collision shape.
    #[must_use]
    pub const fn box_shape(half_extents: Vec3, offset: Vec3) -> Self {
        Self(CollisionShapeData::box_shape(half_extents, offset))
    }
}

/// Event emitted when a collision is detected.
#[derive(Message, Debug)]
pub struct CollisionDetected {
    /// The entity that was hit.
    pub target: Entity,
    /// The entity that caused the collision (may be the same for static objects).
    pub other: Entity,
    /// The point of impact in world coordinates.
    pub point: Vec3,
    /// The normal at the point of impact (points from target to other).
    pub normal: Vec3,
    /// The relative velocity at impact (other - target).
    pub relative_velocity: Vec3,
    /// The penetration depth (how much the shapes overlap).
    pub penetration_depth: f32,
}

/// Marker component for static (non-moving) collision objects.
#[derive(Component, Debug, Clone, Copy)]
pub struct StaticBody;

/// Marker component for dynamic (moving) collision objects.
#[derive(Component, Debug, Clone, Copy)]
pub struct DynamicBody;

/// Collision layers component for filtering collisions.
///
/// Per avian3d conventions, we use a u32 bitmask where each bit
/// represents a layer. Entities on different layers don't collide
/// unless their masks overlap.
#[derive(Component, Debug, Clone, Copy)]
pub struct CollisionLayersComponent(pub CollisionLayers);

impl Deref for CollisionLayersComponent {
    type Target = CollisionLayers;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CollisionLayersComponent {
    /// Creates a new collision layers component.
    #[must_use]
    pub const fn new(layers: CollisionLayers) -> Self {
        Self(layers)
    }
}
