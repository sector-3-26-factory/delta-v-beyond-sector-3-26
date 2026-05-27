// See AGENTS.md
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

//! JSON loading, schema validation, and configuration management.
//!
//! This crate implements the two-layer configuration system (ADR-0010):
//! a default layer shipped under `assets/config/` and an optional user
//! override layer under the platform XDG config directory.
//!
//! Every JSON file is validated against its JSON Schema before use
//! (ADR-0012). Missing or invalid defaults are hard errors (ADR-0013).
//!
//! See also ADR-0011 (Keybindings) and ADR-0035 (Hot-reload in dev builds).

#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![warn(clippy::all, clippy::pedantic)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

pub mod error;
pub mod keybindings;
pub mod loader;
pub mod resources;

#[cfg(test)]
#[path = "loader_tests.rs"]
mod loader_tests;

pub use error::ConfigError;
pub use keybindings::{ActionBindings, Keybindings};
pub use resources::KeybindingsResource;

use bevy::prelude::*;
use delta_v_core::AppState;

use crate::loader::load_keybindings;

/// Configuration plugin: loads and validates all JSON config files.
///
/// Systems run in [`AppState::LoadingDefaults`]. On success the plugin
/// inserts [`KeybindingsResource`] and transitions to
/// [`AppState::LoadingWorld`]. On failure the application panics with a
/// descriptive message (ADR-0013).
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LoadingDefaults), load_configs_system);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Loads all configuration files and inserts their resources.
///
/// Transitions the state machine to [`AppState::LoadingWorld`] when done.
fn load_configs_system(mut commands: Commands<'_, '_>, mut next: ResMut<'_, NextState<AppState>>) {
    // Keybindings (ADR-0011).
    // INVARIANT: a missing or invalid keybindings file is a hard startup
    // error (ADR-0013). The panic is intentional; no recovery is possible.
    #[allow(clippy::panic)]
    let keybindings = load_keybindings().unwrap_or_else(|e| {
        panic!("fatal: failed to load keybindings: {e}");
    });
    log::info!("keybindings loaded ({} actions)", keybindings.actions.len());
    commands.insert_resource(KeybindingsResource(keybindings));

    // Transition to the next phase.
    next.set(AppState::LoadingWorld);
}
