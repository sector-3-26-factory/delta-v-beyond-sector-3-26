// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Collision shape conversion from runtime types to physics components.

use bevy::prelude::Vec3;
use delta_v_types::CollisionShapeData;

/// Applies scale to a `CollisionShapeData`.
///
/// This is the SINGLE function that handles sphere/box scaling for ALL entity types.
///
/// # Arguments
///
/// * `shape` - The collision shape data from the template.
/// * `scale` - The scale factor to apply to the shape dimensions.
///
/// # Panics
///
/// Panics if the shape type is unknown (should never happen with valid templates).
#[must_use]
pub fn scale_collision_shape(shape: &CollisionShapeData, scale: f32) -> CollisionShapeData {
    match shape.shape_type {
        delta_v_types::CollisionShapeType::Sphere { radius } => {
            let scaled_radius = radius * scale;
            let offset = shape.offset * scale;
            CollisionShapeData::sphere(scaled_radius, offset)
        }
        delta_v_types::CollisionShapeType::Box { half_extents } => {
            let scaled_half_extents = half_extents * scale;
            let offset = shape.offset * scale;
            CollisionShapeData::box_shape(scaled_half_extents, offset)
        }
        delta_v_types::CollisionShapeType::ConvexHull => {
            // ConvexHull not yet implemented - return a default sphere shape
            // This should be replaced with proper ConvexHull support when implemented
            CollisionShapeData::sphere(0.0, Vec3::ZERO)
        }
    }
}
