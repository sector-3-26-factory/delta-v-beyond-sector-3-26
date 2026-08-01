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

//! Velocity indicator rendering for the velocity vector indicator.
//!
//! ADR-0044 exempts UI rendering infrastructure. This module generates a
//! simple arrow image in memory — no external asset file required.

// Procedural pixel manipulation requires indexing and casts that clippy
// cannot verify at compile time. The buffer size is computed from
// `width * height * 4` and every pixel write is bounds-checked by the
// loop invariants.
#![allow(
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::similar_names
)]

use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Width of the arrow texture in pixels.
pub const ARROW_WIDTH: u32 = 16;

/// Height of the arrow texture in pixels.
pub const ARROW_HEIGHT: u32 = 32;

/// Creates a procedural arrow texture pointing upward.
///
/// The arrow is white with full opacity. Background is transparent.
/// The shape is a triangle (pointing up) with a small rectangular shaft.
///
/// The texture is `ARROW_WIDTH` × `ARROW_HEIGHT` pixels, RGBA8 format.
#[must_use]
#[allow(
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn create_arrow_image() -> Image {
    let w = ARROW_WIDTH;
    let h = ARROW_HEIGHT;
    let mut data = vec![0u8; (w * h) as usize * 4];

    let half_w = w as f32 / 2.0;
    let shaft_start = (h as f32 * 0.5) as u32;
    let shaft_width = (w as f32 * 0.25) as u32;
    let shaft_half = (shaft_width / 2) as f32;

    for y in 0..h {
        for x in 0..w {
            let is_inside = if y < shaft_start {
                let progress = y as f32 / shaft_start as f32;
                let allowed_half_width = half_w * progress;
                let dist_from_center = (x as f32 - half_w).abs();
                dist_from_center <= allowed_half_width
            } else {
                let dist_from_center = (x as f32 - half_w).abs();
                dist_from_center <= shaft_half
            };

            if is_inside {
                let base = (y * w + x) as usize * 4;
                data[base] = 255; // R
                data[base + 1] = 255; // G
                data[base + 2] = 255; // B
                data[base + 3] = 255; // A
            }
        }
    }

    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Speed unit tier thresholds (in m/s).
const THRESHOLD_MS: f32 = 8.333; // 30 km/h
const THRESHOLD_KMH: f32 = 1388.889; // 5000 km/h
const THRESHOLD_KMS: f32 = 300_000.0; // 300 km/s
const THRESHOLD_C: f32 = 299_792_458.0; // c = 299,792,458 m/s

/// Formats a speed value (`m/s`) as a human-readable string with locale-aware separators.
///
/// Uses the following unit tiers:
/// - < 30 km/h: `m/s`
/// - 30 km/h - 5000 km/h: `km/h`
/// - 5000 km/h - 300 km/s: `km/s`
/// - 300 km/s - 1 Mm/s: `Mm/s` (shows as 0.3 - 1.0 Mm/s)
/// - 1 Mm/s - c: `Mm/s` (shows as 1 - 300 Mm/s)
/// - > c: `c` (fraction of light speed)
#[must_use]
#[allow(clippy::too_many_arguments, clippy::uninlined_format_args)]
pub fn format_speed(
    speed_ms: f32,
    decimal_sep: char,
    thousands_sep: char,
    unit_ms: &str,
    unit_kmh: &str,
    unit_kms: &str,
    unit_mms: &str,
    unit_c: &str,
) -> String {
    let value = speed_ms.abs();

    let (converted, unit_str) = if value < THRESHOLD_MS {
        (value, unit_ms)
    } else if value < THRESHOLD_KMH {
        (value * 3.6, unit_kmh)
    } else if value < THRESHOLD_KMS {
        (value / 1000.0, unit_kms)
    } else if value < THRESHOLD_C {
        (value / 1_000_000.0, unit_mms)
    } else {
        (value / THRESHOLD_C, unit_c)
    };

    // Format number with locale-aware separators.
    if converted < 10.0 {
        // 1 decimal place for small numbers.
        let formatted = format!("{:.1}", converted);
        let formatted = formatted.replace('.', &decimal_sep.to_string());
        format!("{formatted} {unit_str}")
    } else if converted < 1000.0 {
        // Integer for medium numbers.
        let formatted = format!("{:.0}", converted);
        format!("{formatted} {unit_str}")
    } else {
        // Thousands separator for large numbers.
        let int_part = converted as u64;
        let formatted = format_thousands(int_part, thousands_sep);
        format!("{formatted} {unit_str}")
    }
}

/// Formats an integer with thousands separators.
fn format_thousands(n: u64, sep: char) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            result.push(sep);
        }
        result.push(c);
    }
    result
}

/// Creates a procedural arrow texture with thrust-based fill.
///
/// The arrow shape points upward. The bottom portion is filled with color
/// (green for forward thrust, red for backward thrust) proportional to
/// `thrust_ratio` (0.0 = no fill, 1.0 = fully filled). The top portion
/// remains white.
///
/// # Arguments
/// * `thrust_ratio` – 0.0 to 1.0, fraction of max thrust currently applied.
/// * `is_forward` – true for forward thrust (green), false for backward (red).
#[must_use]
#[allow(
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::branches_sharing_code
)]
pub fn create_arrow_image_with_thrust(thrust_ratio: f32, is_forward: bool) -> Image {
    let w = ARROW_WIDTH;
    let h = ARROW_HEIGHT;
    let mut data = vec![0u8; (w * h) as usize * 4];

    let half_w = w as f32 / 2.0;
    let shaft_start = (h as f32 * 0.5) as u32;
    let shaft_width = (w as f32 * 0.25) as u32;
    let shaft_half = (shaft_width / 2) as f32;

    // Fill level: 0.0 = no fill, 1.0 = full arrow filled.
    let fill_level = thrust_ratio.clamp(0.0, 1.0);
    let fill_y = (h as f32 * (1.0 - fill_level)) as u32;

    // Color for thrust fill.
    let (r, g, b) = if is_forward {
        (0, 255, 0) // Green for forward
    } else {
        (255, 0, 0) // Red for backward
    };

    for y in 0..h {
        for x in 0..w {
            let is_inside = if y < shaft_start {
                let progress = y as f32 / shaft_start as f32;
                let allowed_half_width = half_w * progress;
                let dist_from_center = (x as f32 - half_w).abs();
                dist_from_center <= allowed_half_width
            } else {
                let dist_from_center = (x as f32 - half_w).abs();
                dist_from_center <= shaft_half
            };

            if is_inside {
                let base = (y * w + x) as usize * 4;
                if y >= fill_y {
                    // Filled portion: colored.
                    data[base] = r;
                    data[base + 1] = g;
                    data[base + 2] = b;
                    data[base + 3] = 255;
                } else {
                    // Unfilled portion: white.
                    data[base] = 255;
                    data[base + 1] = 255;
                    data[base + 2] = 255;
                    data[base + 3] = 255;
                }
            }
        }
    }

    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Updates an existing arrow image in place with thrust-based fill.
///
/// Modifies the image data directly without creating a new image.
/// The bottom portion is filled with color (green for forward, red for backward)
/// proportional to `thrust_ratio`. The top portion remains white.
#[allow(clippy::branches_sharing_code)]
pub fn update_arrow_image_in_place(image: &mut Image, thrust_ratio: f32, is_forward: bool) {
    let w = image.texture_descriptor.size.width;
    let h = image.texture_descriptor.size.height;
    let half_w = w as f32 / 2.0;
    let shaft_start = (h as f32 * 0.5) as u32;
    let shaft_width = (w as f32 * 0.25) as u32;
    let shaft_half = (shaft_width / 2) as f32;

    let fill_level = thrust_ratio.clamp(0.0, 1.0);
    let fill_y = (h as f32 * (1.0 - fill_level)) as u32;

    let (r, g, b) = if is_forward {
        (0, 255, 0) // Green for forward
    } else {
        (255, 0, 0) // Red for backward
    };

    // Ensure data buffer is large enough.
    let expected_len = (w * h) as usize * 4;
    let data = image.data.get_or_insert_with(|| vec![0u8; expected_len]);
    if data.len() != expected_len {
        data.resize(expected_len, 0);
    }

    for y in 0..h {
        for x in 0..w {
            let is_inside = if y < shaft_start {
                let progress = y as f32 / shaft_start as f32;
                let allowed_half_width = half_w * progress;
                let dist_from_center = (x as f32 - half_w).abs();
                dist_from_center <= allowed_half_width
            } else {
                let dist_from_center = (x as f32 - half_w).abs();
                dist_from_center <= shaft_half
            };

            if is_inside {
                let base = (y * w + x) as usize * 4;
                if y >= fill_y {
                    data[base] = r;
                    data[base + 1] = g;
                    data[base + 2] = b;
                    data[base + 3] = 255;
                } else {
                    data[base] = 255;
                    data[base + 1] = 255;
                    data[base + 2] = 255;
                    data[base + 3] = 255;
                }
            }
        }
    }
}

/// Number of preset thrust fill levels.
const THRUST_LEVELS: u32 = 10;

/// Creates a set of preset arrow textures for different thrust levels.
///
/// Returns a vector of images, where index 0 is no fill (all white),
/// index 1 is 10% fill, ..., index `THRUST_LEVELS` is 100% fill.
pub fn create_thrust_arrow_presets(color: [u8; 3]) -> Vec<Image> {
    let mut images = Vec::new();
    for level in 0..=THRUST_LEVELS {
        let thrust_ratio = level as f32 / THRUST_LEVELS as f32;
        let image = create_arrow_image_with_thrust_color(thrust_ratio, color);
        images.push(image);
    }
    images
}

/// Creates an arrow image with a specific thrust fill color.
#[allow(clippy::branches_sharing_code)]
fn create_arrow_image_with_thrust_color(thrust_ratio: f32, color: [u8; 3]) -> Image {
    let w = ARROW_WIDTH;
    let h = ARROW_HEIGHT;
    let mut data = vec![0u8; (w * h) as usize * 4];

    let half_w = w as f32 / 2.0;
    let shaft_start = (h as f32 * 0.5) as u32;
    let shaft_width = (w as f32 * 0.25) as u32;
    let shaft_half = (shaft_width / 2) as f32;

    let fill_level = thrust_ratio.clamp(0.0, 1.0);
    let fill_y = (h as f32 * (1.0 - fill_level)) as u32;

    for y in 0..h {
        for x in 0..w {
            let is_inside = if y < shaft_start {
                let progress = y as f32 / shaft_start as f32;
                let allowed_half_width = half_w * progress;
                let dist_from_center = (x as f32 - half_w).abs();
                dist_from_center <= allowed_half_width
            } else {
                let dist_from_center = (x as f32 - half_w).abs();
                dist_from_center <= shaft_half
            };

            if is_inside {
                let base = (y * w + x) as usize * 4;
                if y >= fill_y {
                    data[base] = color[0];
                    data[base + 1] = color[1];
                    data[base + 2] = color[2];
                    data[base + 3] = 255;
                } else {
                    data[base] = 255;
                    data[base + 1] = 255;
                    data[base + 2] = 255;
                    data[base + 3] = 255;
                }
            }
        }
    }

    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
