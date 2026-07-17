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

// ---------------------------------------------------------------------------
// Distance to surface calculation
// ---------------------------------------------------------------------------

/// Calculates the distance from a point to the surface of a collision shape.
///
/// This is used for navigation/targeting distance display, showing the distance
/// to the object's surface rather than center-to-center distance.
///
/// # Arguments
///
/// * `point` - The position of the player or observer.
/// * `entity_pos` - The world position of the entity with the collision shape.
/// * `shape` - The collision shape component.
///
/// # Returns
///
/// The distance from `point` to the surface of `shape`. Returns 0.0 if the point
/// is inside or on the surface of the shape.
///
/// # Shape Support
///
/// - **Sphere**: `max(0.0, center_distance - radius)`
/// - **Box**: Distance to closest point on the box surface
/// - **`ConvexHull`**: Returns center distance (placeholder, will be updated when convex hulls are implemented)
#[must_use]
pub fn distance_to_surface(point: Vec3, entity_pos: Vec3, shape: &CollisionShape) -> f32 {
    let center_distance = (point - entity_pos).length();

    match &shape.shape_type {
        CollisionShapeType::Sphere { radius } => {
            // For spheres, subtract the radius from center distance
            (center_distance - radius).max(0.0)
        }
        CollisionShapeType::Box { half_extents } => {
            // For boxes, find the closest point on the box surface
            // The box is centered at entity_pos with the given half_extents
            // We need to account for the shape's offset
            let box_center = entity_pos + shape.offset;

            // Get the closest point on the box to the player
            // This is the point on the box that is closest to the player position
            let closest = point.clamp(box_center - *half_extents, box_center + *half_extents);

            // Distance from player to closest point on box
            (point - closest).length()
        }
        CollisionShapeType::ConvexHull => {
            // Placeholder for future convex hull support
            // When implemented, this will find the closest point on the hull
            center_distance
        }
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
