// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Collision shape conversion from JSON to physics components.

use bevy::prelude::Vec3;
use delta_v_physics::CollisionShape;
use delta_v_types::CollisionShapeJson;

/// Converts a `CollisionShapeJson` into a physics `CollisionShape`.
///
/// This is the SINGLE function that handles sphere/box conversion for ALL entity types.
/// See ADR-0046 for the `Json` suffix naming convention.
///
/// # Errors
///
/// Returns an error string if the shape type is unknown or required fields are missing.
pub fn shape_from_json(json: &CollisionShapeJson, _scale: f32) -> Result<CollisionShape, String> {
    match json.shape_type.as_str() {
        "sphere" => {
            // INVARIANT: radius is required for sphere shapes (ADR-0013 - no silent fallbacks)
            let radius = json.radius.as_ref().map(|r| r.value).ok_or_else(|| {
                "collision_shape.radius is required for sphere shapes".to_string()
            })?;
            // offset has a schema default, so it's safe to use unwrap_or
            let offset = json.offset.map_or(Vec3::ZERO, Vec3::from);
            Ok(CollisionShape::sphere(radius, offset))
        }
        "box" => {
            // INVARIANT: half_extents is required for box shapes (ADR-0013 - no silent fallbacks)
            let half_extents = json
                .half_extents
                .as_ref()
                .map(|h| Vec3::from(*h))
                .ok_or_else(|| {
                    "collision_shape.half_extents is required for box shapes".to_string()
                })?;
            // offset has a schema default, so it's safe to use unwrap_or
            let offset = json.offset.map_or(Vec3::ZERO, Vec3::from);
            Ok(CollisionShape::box_shape(half_extents, offset))
        }
        _ => Err(format!("Unknown collision shape type: {}", json.shape_type)),
    }
}
