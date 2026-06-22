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

//! Fade gradient image generation for window scroll areas.

use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::components::{BACKGROUND_COLOR, FADE_ZONE_HEIGHT};

/// Creates the top fade gradient image: window background (top) → transparent (bottom).
///
/// Uses a cubic ease-out curve: 1 - (1-t)³ where t goes from 1 at top to 0 at bottom.
/// This keeps the background color strong for most of the zone, then fades quickly at the edge.
/// The image is 1 pixel wide and `FADE_ZONE_HEIGHT` pixels tall, in RGBA8 format.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn create_fade_top_image() -> Image {
    let height = FADE_ZONE_HEIGHT as u32;
    let mut data = Vec::with_capacity((height * 4) as usize);

    for y in 0..height {
        // t = 1 at top, 0 at bottom
        let t = (height - y) as f32 / height as f32;
        // Cubic ease-out: strong color for most of the zone, quick fade at edge
        let alpha = ((1.0 - (1.0 - t).powi(3)) * 255.0).round() as u8;
        data.push((BACKGROUND_COLOR.to_linear().red * 255.0) as u8); // R
        data.push((BACKGROUND_COLOR.to_linear().green * 255.0) as u8); // G
        data.push((BACKGROUND_COLOR.to_linear().blue * 255.0) as u8); // B
        data.push(alpha); // A
    }

    Image::new(
        Extent3d {
            width: 1,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Creates the bottom fade gradient image: transparent (top) → window background (bottom).
///
/// Uses a cubic ease-out curve: 1 - (1-t)³ where t goes from 0 at top to 1 at bottom.
/// This keeps the background color strong for most of the zone, then fades quickly at the edge.
/// The image is 1 pixel wide and `FADE_ZONE_HEIGHT` pixels tall, in RGBA8 format.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn create_fade_bottom_image() -> Image {
    let height = FADE_ZONE_HEIGHT as u32;
    let mut data = Vec::with_capacity((height * 4) as usize);

    for y in 0..height {
        // t = 0 at top, 1 at bottom
        let t = y as f32 / height as f32;
        // Cubic ease-out: quick fade from transparent, then strong color for most of the zone
        let alpha = ((1.0 - (1.0 - t).powi(3)) * 255.0).round() as u8;
        data.push((BACKGROUND_COLOR.to_linear().red * 255.0) as u8); // R
        data.push((BACKGROUND_COLOR.to_linear().green * 255.0) as u8); // G
        data.push((BACKGROUND_COLOR.to_linear().blue * 255.0) as u8); // B
        data.push(alpha); // A
    }

    Image::new(
        Extent3d {
            width: 1,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
