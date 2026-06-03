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

//! Core ECS fundamentals, shared components, and plugin traits.
//!
//! This crate provides the foundation that all domain crates build upon.
//! It owns the top-level [`AppState`] state machine, the logical input
//! action pipeline (ADR-0011), and registers the state-transition logging
//! systems that every other plugin relies on.
//!
//! [`KeybindingsResource`] is defined here (not in `delta-v-config`) so
//! that [`input::input_translation_system`] can read it without creating
//! a crate-dependency cycle.  `ConfigPlugin` in `delta-v-config` inserts
//! the resource; `delta-v-config` already depends on `delta-v-core`.
//!
//! See ADR-0005 (plugin architecture) and ADR-0018 (state management).

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

pub mod camera;
pub mod debug_axes;
pub mod debug_config;
pub mod diagnostics;
pub mod flight_assist;
pub mod input;
pub mod keybindings_resource;
pub mod spawn_sets;
pub mod state;

pub use camera::{spawn_chase_camera, CameraFollow, ChaseCameraOffset, PlayerShipEntity};
pub use debug_axes::{
    mark_debug_axes, spawn_debug_axes, update_debug_axes_positions, DebugAxes, DebugAxesEligible,
    DebugAxisTarget,
};
pub use debug_config::DebugConfig;
pub use diagnostics::{DiagnosticsConfig, DiagnosticsPlugin};
pub use flight_assist::{
    FlightAssist, FlightAssistConfig, FlightAssistState, MainThrusterTemplate, PhysicalQuantity,
    ShipPropulsionConfig, ShipPropulsionTemplate, ShipTemplate, ThrustCommand, TorqueCommand,
};
pub use input::{ActiveActions, InputSet, LogicalAction};
pub use keybindings_resource::KeybindingsResource;
pub use spawn_sets::WorldSpawnSet;
pub use state::AppState;

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;

#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod diagnostics_tests;

#[cfg(test)]
#[path = "input_tests.rs"]
mod input_tests;

#[cfg(test)]
#[path = "camera_tests.rs"]
mod camera_tests;

use bevy::prelude::*;

use crate::input::{input_log_system, input_translation_system};

/// The core plugin that initialises fundamental ECS infrastructure.
///
/// Responsibilities:
/// - Registers the [`AppState`] state machine.
/// - Logs an `INFO` line on every state transition (ADR-0015, ADR-0018).
/// - Immediately transitions from [`AppState::Boot`] to
///   [`AppState::LoadingDefaults`] so that config and world loaders can
///   start their work.
/// - Initialises [`ActiveActions`] and registers the input translation
///   pipeline (ADR-0011, ADR-0017).
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        info!(version = env!("CARGO_PKG_VERSION"), "Delta-V starting");

        app.init_state::<AppState>().add_plugins(DiagnosticsPlugin);

        // Log every state entry at INFO level (ADR-0015, ADR-0018).
        app.add_systems(OnEnter(AppState::Boot), log_boot);
        app.add_systems(OnEnter(AppState::LoadingDefaults), log_loading_defaults);
        app.add_systems(OnEnter(AppState::LoadingWorld), log_loading_world);
        app.add_systems(OnEnter(AppState::SpawningEntities), log_spawning_entities);
        app.add_systems(OnEnter(AppState::InGame), (log_in_game, spawn_chase_camera));

        // Configure WorldSpawnSet ordering: MarkDebugAxes runs after all domain spawning.
        // Per ADR-0005 (plugin architecture), debug visualization is decoupled from domains.
        // Sets must be configured in Update schedule, not OnEnter (per Bevy system scheduling).
        app.configure_sets(
            Update,
            WorldSpawnSet::MarkDebugAxes
                .after(WorldSpawnSet::SpawnShips)
                .run_if(in_state(AppState::SpawningEntities)),
        );

        // Mark eligible entities with debug axes, then spawn axis meshes (ADR-0022, ADR-0005).
        // mark_debug_axes converts DebugAxesEligible → DebugAxes based on config.
        // spawn_debug_axes renders the axes for marked entities.
        // Both run in Update during SpawningEntities, after all domain spawn systems.
        app.add_systems(
            Update,
            (debug_axes::mark_debug_axes, spawn_debug_axes)
                .chain()
                .in_set(WorldSpawnSet::MarkDebugAxes),
        );

        // Chase camera follows the ship every frame during InGame.
        app.add_systems(
            Update,
            camera::chase_camera_system.run_if(in_state(AppState::InGame)),
        );

        // Debug axes position update: keeps axis roots at the target's position
        // while maintaining world-aligned (identity) rotation.
        app.add_systems(
            Update,
            debug_axes::update_debug_axes_positions.run_if(in_state(AppState::InGame)),
        );

        // Input pipeline (ADR-0011, ADR-0017).
        // Runs in FixedUpdate during InGame; Translate always before Log.
        app.init_resource::<ActiveActions>()
            .configure_sets(
                FixedUpdate,
                (InputSet::Translate, InputSet::Log)
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                (
                    input_translation_system.in_set(InputSet::Translate),
                    input_log_system.in_set(InputSet::Log),
                )
                    .run_if(in_state(AppState::InGame)),
            );

        // Immediately leave Boot: transition to LoadingDefaults so that
        // ConfigPlugin can begin loading on the same frame.
        app.add_systems(OnEnter(AppState::Boot), advance_from_boot);
    }
}

// ---------------------------------------------------------------------------
// State-transition logging systems (ADR-0015, ADR-0018)
// ---------------------------------------------------------------------------

fn log_boot() {
    info!("AppState -> Boot");
}

fn log_loading_defaults() {
    info!("AppState -> LoadingDefaults");
}

fn log_loading_world() {
    info!("AppState -> LoadingWorld");
}

fn log_spawning_entities() {
    info!("AppState -> SpawningEntities");
}

fn log_in_game() {
    info!("AppState -> InGame");
}

// ---------------------------------------------------------------------------
// Boot transition
// ---------------------------------------------------------------------------

/// Advances the state machine from [`AppState::Boot`] to
/// [`AppState::LoadingDefaults`].
///
/// This runs in `OnEnter(Boot)` so the transition happens on the very
/// first frame, before any rendering occurs.
fn advance_from_boot(mut next: ResMut<'_, NextState<AppState>>) {
    next.set(AppState::LoadingDefaults);
}
