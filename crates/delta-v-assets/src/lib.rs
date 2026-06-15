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

//! Asset loading and template management.
//!
//! This crate owns:
//! - Asset path resolution
//! - Template loading
//! - Template merging
//!
//! See ADR-0049 for the template system reorganization.

#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::cargo
)]
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

use bevy::app::Plugin;

pub mod error;
pub mod paths;
pub mod template;

#[cfg(test)]
#[path = "template_tests.rs"]
mod template_tests;

// Re-exports for convenience
pub use error::AssetError;
pub use paths::{get_workspace_root, resolve_template_path};

/// Plugin for asset loading and template management.
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, _app: &mut bevy::app::App) {
        // Asset plugin initialization
        // Template loading is done on demand via the template module functions
    }
}
