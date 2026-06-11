// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Collision shape conversion from JSON to physics components.

use bevy::prelude::{Component, Vec3};
use delta_v_types::CollisionShapeJson;

/// Converts a CollisionShapeJson into a physics CollisionShape.
///
/// This is the SINGLE function that handles sphere/box conversion for ALL entity types.
/// See ADR-0046 for the `Json` suffix naming convention.
///
/// # Errors
///
/// Returns an error string if the shape type is unknown.
pub fn shape_from_json(json: &CollisionShapeJson, _scale: f32) -> Result<CollisionShape, String> {
    match json.shape_type.as_str() {
        "sphere" => {
            let radius = json.radius.as_ref().map_or(0.5, |r| r.value);
            Ok(CollisionShape::Sphere {
                radius,
                offset: json.offset.map_or(Vec3::ZERO, |o| Vec3::from(o)),
            })
        }
        "box" => {
            let half_extents = json
                .half_extents
                .as_ref()
                .map_or(Vec3::new(0.5, 0.5, 0.5), |h| Vec3::from(*h));
            Ok(CollisionShape::Box {
                half_extents,
                offset: json.offset.map_or(Vec3::ZERO, |o| Vec3::from(o)),
            })
        }
        _ => Err(format!("Unknown collision shape type: {}", json.shape_type)),
    }
}

/// Collision shape component for physics simulation.
///
/// This is a Bevy Component that lives in `delta-v-physics`, not `delta-v-types`.
/// The `CollisionShapeJson` in `delta-v-types` is the JSON deserialization type.
///
/// NOTE: This is a temporary definition in `delta-v-spawn`. The actual
/// `CollisionShape` component should be moved to `delta-v-physics` in a future
/// refactoring. This definition exists here to allow the conversion function
/// to compile.
#[derive(Component, Debug, Clone)]
pub enum CollisionShape {
    /// Sphere shape with radius and optional offset.
    Sphere {
        /// Radius in metres.
        radius: f32,
        /// Offset from entity origin in metres.
        offset: Vec3,
    },
    /// Box shape with half-extents and optional offset.
    Box {
        /// Half-extents (width/2, height/2, depth/2) in metres.
        half_extents: Vec3,
        /// Offset from entity origin in metres.
        offset: Vec3,
    },
}
