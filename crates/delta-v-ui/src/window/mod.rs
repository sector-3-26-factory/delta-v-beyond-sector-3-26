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

//! Reusable window UI module.
//!
//! Provides a generic window frame with a title bar, hint text, and content area.
//! Uses pure Bevy UI (`Node` + `Text` + `ScrollPosition`) for layout and clipping.
//! The content area is populated via a callback function, making this module
//! reusable for different window types (keybindings menu, settings, etc.).

pub mod components;
pub mod fade_images;
pub mod spawn;
pub mod systems;
pub mod theme;

pub use spawn::{WindowConfig, spawn_window};
pub use theme::{UiTheme, load_ui_theme};
