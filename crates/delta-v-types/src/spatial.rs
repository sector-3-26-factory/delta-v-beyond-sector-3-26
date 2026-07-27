// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Spatial types for 3D geometry and bounding volumes.

pub use bevy::prelude::{Quat, Vec3};
use serde::Deserialize;

/// A 3-component position in metres, deserialized from JSON.
///
/// JSON format: `{"x": N, "y": N, "z": N}`
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Vec3Json {
    /// X component in metres.
    pub x: f32,
    /// Y component in metres.
    pub y: f32,
    /// Z component in metres.
    pub z: f32,
}

impl From<Vec3Json> for Vec3 {
    #[allow(clippy::use_self)]
    fn from(v: Vec3Json) -> Self {
        Vec3::new(v.x, v.y, v.z)
    }
}

/// A 4-component unit quaternion (x, y, z, w), deserialized from JSON.
///
/// JSON format: `{"x": N, "y": N, "z": N, "w": N}`
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct QuatJson {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
    /// W component (scalar part).
    pub w: f32,
}

impl From<QuatJson> for Quat {
    fn from(q: QuatJson) -> Self {
        Self::from_array([q.x, q.y, q.z, q.w])
    }
}

/// Axis-aligned bounding box in ship-local coordinates (metres).
///
/// Computed once from the glTF mesh and stored in the template JSON.
/// Used for camera position defaults, debug axes, and spatial calculations.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct BoundingBoxJson {
    /// Minimum corner (x, y, z in metres).
    pub min: Vec3Json,
    /// Maximum corner (x, y, z in metres).
    pub max: Vec3Json,
}

impl BoundingBoxJson {
    /// Returns the size (extent) of the bounding box in metres.
    #[must_use]
    pub fn size(&self) -> Vec3 {
        Vec3::from(self.max) - Vec3::from(self.min)
    }

    /// Returns the center point of the bounding box in metres.
    #[must_use]
    pub fn center(&self) -> Vec3 {
        (Vec3::from(self.min) + Vec3::from(self.max)) * 0.5
    }
}

/// Axis-aligned bounding box in ship-local coordinates (metres).
///
/// Runtime version with SI units (Vec3 instead of `Vec3Json`).
/// Used for camera position defaults, debug axes, and spatial calculations.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    /// Minimum corner (x, y, z in metres).
    pub min: Vec3,
    /// Maximum corner (x, y, z in metres).
    pub max: Vec3,
}

impl From<BoundingBoxJson> for BoundingBox {
    fn from(json: BoundingBoxJson) -> Self {
        Self {
            min: Vec3::from(json.min),
            max: Vec3::from(json.max),
        }
    }
}

impl BoundingBox {
    /// Returns the size (extent) of the bounding box in metres.
    #[must_use]
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Returns the center point of the bounding box in metres.
    #[must_use]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the half-extents of the bounding box in metres.
    #[must_use]
    pub fn half_extents(&self) -> Vec3 {
        self.size() * 0.5
    }
}
