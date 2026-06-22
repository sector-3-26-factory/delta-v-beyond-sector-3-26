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

//! UI theme — single source of truth for all visual properties.
//!
//! [`UiTheme`] is a Bevy resource that holds the font handle and all
//! visual constants (colors, font sizes, spacing) for windows and overlays.
//! Every consumer references this resource.

use bevy::prelude::*;

/// UI theme resource.
///
/// Inserted once at startup by [`load_ui_theme`].  All UI systems that need
/// the font or visual constants receive `Res<UiTheme>`.
#[derive(Resource, Debug)]
pub struct UiTheme {
    /// Handle to the font used for all UI text.
    pub font: Handle<Font>,
}

impl UiTheme {
    // --- Window ---

    /// Window background color (semi-transparent black).
    ///
    /// Used for the panel background and as the fade target color for
    /// scroll fade zones.
    pub const BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);

    /// Height of the fade-out zones in pixels.
    ///
    /// Controls both the height of the gradient overlay sprites and the
    /// padding lines added at the top and bottom of the scroll content.
    pub const FADE_ZONE_HEIGHT: f32 = 30.0;

    /// Default window size in pixels (width, height).
    pub const DEFAULT_WINDOW_SIZE: Vec2 = Vec2::new(300.0, 200.0);

    // --- Header ---

    /// Header row height in pixels.
    pub const HEADER_HEIGHT: f32 = 20.0;

    /// Horizontal padding inside the header row in pixels.
    pub const HEADER_PADDING: f32 = 5.0;

    /// Title text font size.
    pub const TITLE_FONT_SIZE: f32 = 20.0;

    /// Title text color.
    pub const TITLE_COLOR: Color = Color::WHITE;

    /// Hint text font size.
    pub const HINT_FONT_SIZE: f32 = 14.0;

    /// Hint text color.
    pub const HINT_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);

    // --- Content ---

    /// Group header font size.
    pub const GROUP_HEADER_FONT_SIZE: f32 = 14.0;

    /// Group header text color.
    pub const GROUP_HEADER_COLOR: Color = Color::srgb(1.0, 0.85, 0.0);

    /// General content text font size.
    pub const TEXT_FONT_SIZE: f32 = 12.0;

    /// Label text color (e.g. action names, primary text).
    pub const LABEL_COLOR: Color = Color::WHITE;

    /// Value text color (e.g. key bindings, secondary text).
    pub const VALUE_COLOR: Color = Color::srgb(0.7, 1.0, 0.7);
}

/// Startup system that loads the UI font and inserts the [`UiTheme`] resource.
///
/// Loads `fonts/DejaVuSansMono.ttf` and inserts it as a resource so that
/// all UI systems can access the font handle.
#[allow(clippy::needless_pass_by_value)]
pub fn load_ui_theme(mut commands: Commands<'_, '_>, asset_server: Res<'_, AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/DejaVuSansMono.ttf");
    commands.insert_resource(UiTheme { font });
    tracing::info!("loaded UI theme font (DejaVu Sans Mono)");
}
