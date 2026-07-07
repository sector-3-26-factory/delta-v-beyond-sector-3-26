// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Collision shape types for deserializing collision shapes from JSON.
//!
//! These types are used by `delta-v-spawn` to convert JSON collision shapes
//! into physics `CollisionShape` components.

use bevy::prelude::Vec3;
use serde::Deserialize;

use crate::physics::PhysicalQuantityJson;
use crate::spatial::Vec3Json;

/// Collision layers for categorizing entities.
///
/// Per avian3d conventions, we use a u32 bitmask where each bit
/// represents a layer. Entities on different layers don't collide
/// unless their masks overlap.
#[derive(Debug, Clone, Copy)]
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
    /// Ships are on layer 1 and can collide with ships and asteroids.
    pub const SHIP_LAYER: u32 = 1;

    /// Layer for asteroids (static obstacles).
    ///
    /// Asteroids are on layer 2 and can collide with ships (`SHIP_LAYER`).
    pub const ASTEROID_LAYER: u32 = 2;

    /// Collision layers for ships: on `SHIP_LAYER`, can collide with `SHIP_LAYER` and `ASTEROID_LAYER`.
    pub const SHIP: super::CollisionLayers =
        super::CollisionLayers::new(SHIP_LAYER, SHIP_LAYER | ASTEROID_LAYER);

    /// Collision layers for asteroids: on `ASTEROID_LAYER`, can collide with `SHIP_LAYER`.
    pub const ASTEROID: super::CollisionLayers =
        super::CollisionLayers::new(ASTEROID_LAYER, SHIP_LAYER);

    /// Layer for projectiles (weapons fire).
    ///
    /// Projectiles are on layer 3 and can collide with ships (`SHIP_LAYER`)
    /// and asteroids (`ASTEROID_LAYER`).
    pub const PROJECTILE_LAYER: u32 = 3;

    /// Collision layers for projectiles: on `PROJECTILE_LAYER`, can collide with `SHIP_LAYER` and `ASTEROID_LAYER`.
    pub const PROJECTILE: super::CollisionLayers =
        super::CollisionLayers::new(PROJECTILE_LAYER, SHIP_LAYER | ASTEROID_LAYER);
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

/// Plain data for a collision shape (no Bevy `Component` derive).
///
/// This is the shared data type used across crates. The physics domain
/// wraps it in a [`super::CollisionShape`] newtype (in `delta-v-physics`)
/// that adds `#[derive(Component)]` and implements `Deref` for transparent access.
#[derive(Debug, Clone, Copy)]
pub struct CollisionShapeData {
    /// The shape type and its parameters.
    pub shape_type: CollisionShapeType,
    /// Offset of the collision shape center from the entity origin in metres.
    pub offset: Vec3,
}

impl CollisionShapeData {
    /// Creates a new sphere collision shape.
    #[must_use]
    pub const fn sphere(radius: f32, offset: Vec3) -> Self {
        Self {
            shape_type: CollisionShapeType::Sphere { radius },
            offset,
        }
    }

    /// Creates a new box collision shape.
    #[must_use]
    pub const fn box_shape(half_extents: Vec3, offset: Vec3) -> Self {
        Self {
            shape_type: CollisionShapeType::Box { half_extents },
            offset,
        }
    }
}

/// Collision shape deserialized from template JSON.
///
/// This is the single JSON collision shape type for ALL entity types
/// (ships, asteroids, planets, stations). See ADR-0046 for the `Json` suffix
/// naming convention.
///
/// JSON format:
/// ```json
/// {
///   "type": "sphere",
///   "radius": {"value": 1.0, "unit": "m"}
/// }
/// ```
/// or
/// ```json
/// {
///   "type": "box",
///   "half_extents": {"x": 1.0, "y": 1.0, "z": 1.0}
/// }
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct CollisionShapeJson {
    /// The shape type: "sphere" or "box".
    #[serde(rename = "type")]
    pub shape_type: String,
    /// Radius for sphere shapes (metres).
    pub radius: Option<PhysicalQuantityJson>,
    /// Half-extents for box shapes (metres).
    pub half_extents: Option<Vec3Json>,
    /// Offset of the collision shape center from the entity origin in metres.
    /// Default (0,0,0) from schema.
    pub offset: Option<Vec3Json>,
}

impl CollisionShapeJson {
    /// Returns the shape type ("sphere" or "box").
    #[must_use]
    pub fn shape_type(&self) -> &str {
        &self.shape_type
    }

    /// Returns true if this is a sphere shape.
    #[must_use]
    pub fn is_sphere(&self) -> bool {
        self.shape_type == "sphere"
    }

    /// Returns true if this is a box shape.
    #[must_use]
    pub fn is_box(&self) -> bool {
        self.shape_type == "box"
    }
}
