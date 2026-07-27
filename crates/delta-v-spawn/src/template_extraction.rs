// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Template field extraction helpers.
//!
//! These functions extract values from `EntityTemplate` enum variants.
//! They are used by domain spawners to get gameplay values from templates.

use delta_v_types::{BoundingBox, EntityTemplate};

/// Extracts a mass value from an `EntityTemplate`.
///
/// # Panics
///
/// Panics if the template is not a ship type. This should never happen because
/// only ships have mass values that need extraction.
#[must_use]
pub const fn extract_mass(template: &EntityTemplate) -> f32 {
    template.mass()
}

/// Extracts a `BoundingBox` from an `EntityTemplate`.
///
/// # Panics
///
/// Panics if the template is not a valid entity type with a bounding box.
#[must_use]
pub fn extract_bounding_box(template: &EntityTemplate) -> &BoundingBox {
    template.bounding_box()
}

/// Extracts a `CollisionShapeData` from an `EntityTemplate`.
///
/// # Panics
///
/// Panics if the template is not a valid entity type with a collision shape.
#[must_use]
pub fn extract_collision_shape(template: &EntityTemplate) -> &delta_v_types::CollisionShapeData {
    template.collision_shape()
}

/// Computes debug axis length from a `BoundingBox` (120% of longest side).
#[must_use]
pub fn compute_debug_axis_length(bbox: &BoundingBox) -> f32 {
    let size = bbox.size();
    let max_dim = size.x.max(size.y).max(size.z);
    max_dim * 1.2
}

/// Scales a `BoundingBox` by the given scale factor.
///
/// Both min and max corners are multiplied by the scale.
#[must_use]
pub fn scale_bounding_box(bbox: &BoundingBox, scale: f32) -> BoundingBox {
    BoundingBox {
        min: bbox.min * scale,
        max: bbox.max * scale,
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
