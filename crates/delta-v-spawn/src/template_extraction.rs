// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Template field extraction helpers.
//!
//! These functions extract values from validated template JSON values.
//! They are used by domain spawners to get gameplay values from templates.

use serde_json::Value;

use delta_v_types::{BoundingBoxJson, CollisionShapeJson, Vec3Json};

/// Extracts a mass value from a validated template JSON value.
///
/// # Panics
///
/// Panics if `mass.value` is missing or not a valid number. This is safe because
/// the schema requires this field and it is validated by `delta-v-json` (ADR-0013).
#[allow(clippy::expect_used, clippy::cast_possible_truncation)]
#[must_use]
pub fn extract_mass(template: &Value) -> f32 {
    // INVARIANT: mass.value is required by schema and validated by delta-v-json (ADR-0013)
    template
        .get("mass")
        .and_then(Value::as_object)
        .and_then(|m| m.get("value"))
        .and_then(Value::as_f64)
        .expect("mass.value should be present and valid per schema") as f32
}

/// Extracts a `Vec3` from a JSON object with x, y, z fields.
///
/// # Panics
///
/// Panics if any of `x`, `y`, `z` fields are missing or not valid numbers.
/// This is safe because the schema requires these fields and they are validated
/// by `delta-v-json` (ADR-0013).
#[allow(clippy::expect_used, clippy::cast_possible_truncation)]
#[must_use]
pub fn extract_vec3(json: &serde_json::Map<String, Value>) -> Vec3Json {
    // INVARIANT: x, y, z are required by schema and validated by delta-v-json (ADR-0013)
    Vec3Json {
        x: json
            .get("x")
            .and_then(Value::as_f64)
            .expect("x should be present and valid") as f32,
        y: json
            .get("y")
            .and_then(Value::as_f64)
            .expect("y should be present and valid") as f32,
        z: json
            .get("z")
            .and_then(Value::as_f64)
            .expect("z should be present and valid") as f32,
    }
}

/// Extracts a `BoundingBoxJson` from a validated template JSON value.
///
/// # Panics
///
/// Panics if `bounding_box`, `min`, or `max` fields are missing or invalid.
/// This is safe because the schema requires these fields and they are validated
/// by `delta-v-json` (ADR-0013).
#[allow(clippy::expect_used)]
#[must_use]
pub fn extract_bounding_box(template: &Value) -> BoundingBoxJson {
    // INVARIANT: bounding_box.min and bounding_box.max are required by schema (ADR-0013)
    let bbox = template
        .get("bounding_box")
        .expect("bounding_box should be present per schema");

    let min = extract_vec3(
        bbox.get("min")
            .and_then(Value::as_object)
            .expect("min should be an object"),
    );
    let max = extract_vec3(
        bbox.get("max")
            .and_then(Value::as_object)
            .expect("max should be an object"),
    );

    BoundingBoxJson { min, max }
}

/// Extracts a `CollisionShapeJson` from a validated template JSON value.
///
/// # Panics
///
/// Panics if `collision_shape` is missing from the template. This is safe because
/// the schema requires this field and it is validated by `delta-v-json` (ADR-0013).
#[allow(clippy::expect_used)]
#[must_use]
pub fn extract_collision_shape(template: &Value) -> CollisionShapeJson {
    // INVARIANT: collision_shape is required by schema and validated by delta-v-json (ADR-0013)
    let shape = template
        .get("collision_shape")
        .expect("collision_shape should be present per schema");
    serde_json::from_value(shape.clone())
        .expect("collision_shape must be valid JSON (validated by delta-v-json)")
}

/// Computes debug axis length from a `BoundingBoxJson` (120% of longest side).
#[must_use]
pub fn compute_debug_axis_length(bbox: &BoundingBoxJson) -> f32 {
    let size = bbox.size();
    let max_dim = size.x.max(size.y).max(size.z);
    max_dim * 1.2
}

/// Scales a `BoundingBoxJson` by the given scale factor.
///
/// Both min and max corners are multiplied by the scale.
#[must_use]
pub fn scale_bounding_box(bbox: &BoundingBoxJson, scale: f32) -> BoundingBoxJson {
    BoundingBoxJson {
        min: Vec3Json {
            x: bbox.min.x * scale,
            y: bbox.min.y * scale,
            z: bbox.min.z * scale,
        },
        max: Vec3Json {
            x: bbox.max.x * scale,
            y: bbox.max.y * scale,
            z: bbox.max.z * scale,
        },
    }
}

/// Resolves the final mass value, using override if present.
///
/// Mass is NOT scaled - it is used as-is from the template, or overridden if
/// `mass_override` is specified.
#[must_use]
pub fn resolve_mass(template_mass: f32, mass_override: Option<f32>) -> f32 {
    mass_override.unwrap_or(template_mass)
}

/// Reads the dimensions of a PNG file from its IHDR chunk.
///
/// # Errors
/// Returns an error if the file cannot be read or is not a valid PNG.
pub fn png_dimensions(path: &str) -> Result<(f32, f32), std::io::Error> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut header = [0u8; 24];
    file.read_exact(&mut header)?;
    // PNG signature: 8 bytes, then IHDR length (4 bytes) + "IHDR" (4 bytes) + width (4 bytes) + height (4 bytes)
    // PNG dimensions are always small (<2^24), so f32 conversion is lossless.
    #[allow(clippy::cast_precision_loss)]
    let width = u32::from_be_bytes([header[16], header[17], header[18], header[19]]) as f32;
    #[allow(clippy::cast_precision_loss)]
    let height = u32::from_be_bytes([header[20], header[21], header[22], header[23]]) as f32;
    Ok((width, height))
}
