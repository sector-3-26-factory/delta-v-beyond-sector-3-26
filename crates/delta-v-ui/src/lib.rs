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

//! UI overlay systems for HUD and menus.
//!
//! This crate contains UI-related systems that are not ship-specific:
//! - Keybindings reference menu (F1)
//! - Future: inventory, price lists, settings menu

#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
// Multiple crate versions are unavoidable due to Bevy ecosystem dependencies
// pulling in transitive duplicates that cannot be unified without upstream fixes.
#![allow(clippy::multiple_crate_versions)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]

use bevy::prelude::*;

pub mod overlays;
pub mod window;

/// Plugin for UI overlay systems.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(overlays::OverlaysPlugin);
        app.add_plugins(window::WindowBorderPlugin);
        app.add_systems(Startup, window::load_ui_theme);
        app.add_systems(Update, window::systems::window_scroll_system);
    }
}
