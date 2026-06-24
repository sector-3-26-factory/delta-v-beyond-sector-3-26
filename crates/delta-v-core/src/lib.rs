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

//! Core ECS fundamentals, shared components, and plugin traits.

#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
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
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

// Subdirectory modules (ADR-0048)
pub mod boundary;
pub mod camera;
pub mod debug;
pub mod diagnostics;
pub mod events;
pub mod flight_assist;
pub mod floating_origin;
pub mod health;
pub mod i18n;
pub mod input;
pub mod spawn;
pub mod state;

// Re-exports from subdirectories
pub use boundary::{
    BoundaryBehavior, SectorBoundary, SectorBoundaryResource, check_sector_boundary_system,
};
pub use camera::{
    ActiveMainCamera, CameraDefinition, PlayerShipEntity, RenderLayer, ShipCamerasTemplate,
    spawn_menu_camera, spawn_ui_camera,
};
pub use debug::{
    AxisLabel, DebugAxes, DebugAxesEligible, DebugConfig, mark_debug_axes, render_debug_axes,
    spawn_debug_axis_labels, update_debug_axis_labels, update_gizmo_render_layers,
};
pub use diagnostics::{DiagnosticsConfig, DiagnosticsPlugin};
pub use events::{FireWeapon, ProjectileHit, SpawnEntity};
pub use flight_assist::{FlightAssist, FlightAssistConfig, FlightAssistState};
pub use floating_origin::{
    FloatingOrigin, FloatingOriginConfig, FloatingOriginEligible, OriginThreshold,
};
pub use health::{Health, Weapon};
pub use i18n::{I18n, KeybindingsMenuTranslations, MenuTranslations, UiTranslations};
pub use spawn::WorldSpawnSet;
pub use state::AppState;

// Re-exports from delta-v-types for shared types (ADR-0046)
pub use delta_v_types::{
    BoundingBoxJson, CollisionShapeJson, PhysicalQuantityJson, PlayerSettings, Vec3Json,
};

#[cfg(test)]
#[path = "state/tests.rs"]
mod state_tests;

#[cfg(test)]
#[path = "diagnostics/tests.rs"]
mod diagnostics_tests;

#[cfg(test)]
#[path = "boundary/tests.rs"]
mod boundary_tests;

use bevy::prelude::*;

/// The core plugin that initialises fundamental ECS infrastructure.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        info!(version = env!("CARGO_PKG_VERSION"), "Delta-V starting");

        app.init_state::<AppState>()
            .add_plugins(DiagnosticsPlugin)
            .add_plugins(input::InputManagerPlugin::<delta_v_types::LogicalAction>::default());

        // Initialize gizmo config with default render layer (will be updated dynamically).
        // The update_gizmo_render_layers system will set the correct layer based on active camera.
        app.init_gizmo_group::<bevy::gizmos::config::DefaultGizmoConfigGroup>();

        // Log every state entry at INFO level (ADR-0015, ADR-0018).
        app.add_systems(OnEnter(AppState::Boot), log_boot);
        app.add_systems(OnEnter(AppState::LoadingDefaults), log_loading_defaults);
        app.add_systems(OnEnter(AppState::LoadingWorld), log_loading_world);
        app.add_systems(OnEnter(AppState::SpawningEntities), log_spawning_entities);
        app.add_systems(
            OnEnter(AppState::InGame),
            (log_in_game, spawn_ui_camera, spawn_menu_camera),
        );
        app.add_systems(OnEnter(AppState::SkirmishOver), log_skirmish_over);

        // Build InputMap from KeybindingsResource after config is loaded.
        // This must run after ConfigPlugin inserts KeybindingsResource
        // and before any FixedUpdate systems that read ActionState.
        app.add_systems(OnEnter(AppState::InGame), build_input_map_system);

        // Configure WorldSpawnSet ordering.
        app.configure_sets(
            Update,
            (
                WorldSpawnSet::SpawnSuns,
                WorldSpawnSet::SpawnPlanets,
                WorldSpawnSet::SpawnMoons,
                WorldSpawnSet::SpawnStations,
                WorldSpawnSet::SpawnAsteroids,
                WorldSpawnSet::SpawnShips,
                WorldSpawnSet::SpawnNpcs,
                WorldSpawnSet::MarkDebugAxes,
            )
                .chain()
                .run_if(in_state(AppState::SpawningEntities)),
        );

        // Mark eligible entities with debug axes.
        app.add_systems(
            Update,
            (debug::mark_debug_axes, debug::spawn_debug_axis_labels)
                .chain()
                .in_set(WorldSpawnSet::MarkDebugAxes),
        );

        // Add gameplay render layers to all entities that have Transform but no RenderLayers.
        // Runs every frame to catch dynamically spawned entities.
        app.add_systems(Update, apply_gameplay_render_layers);

        // Debug axis labels: update UI text positions via viewport projection.
        app.add_systems(
            Update,
            debug::update_debug_axis_labels.run_if(in_state(AppState::InGame)),
        );

        // Debug axes rendering: runs in Update to draw gizmos.
        app.add_systems(
            Update,
            debug::render_debug_axes.run_if(in_state(AppState::InGame)),
        );

        // Update gizmo render layers to match the active camera.
        // Runs when active camera changes to ensure gizmos render only on active camera's layer.
        app.add_systems(
            Update,
            debug::update_gizmo_render_layers.run_if(in_state(AppState::InGame)),
        );

        // Immediately leave Boot.
        app.add_systems(OnEnter(AppState::Boot), advance_from_boot);
    }
}

/// Adds `RenderLayer::Gameplay` render layers to all entities that have a
/// `Transform` but no `RenderLayers` component, excluding cameras.
/// This ensures gameplay objects are visible to all ship cameras (all on layer 0).
// INVARIANT: Query uses multiple With/Without clauses for precise entity filtering.
#[allow(clippy::type_complexity)]
fn apply_gameplay_render_layers(
    mut commands: Commands<'_, '_>,
    query: Query<
        '_,
        '_,
        Entity,
        (
            With<Transform>,
            Without<bevy::camera::visibility::RenderLayers>,
            Without<Camera>,
        ),
    >,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(RenderLayer::Gameplay.render_layers());
    }
}

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

fn log_skirmish_over() {
    info!("AppState -> SkirmishOver");
}

fn advance_from_boot(mut next: ResMut<'_, NextState<AppState>>) {
    next.set(AppState::LoadingDefaults);
}

/// Builds the `InputMap<LogicalAction>` from the loaded `KeybindingsResource`
/// and registers it as a Bevy resource.
///
/// This enables the leafwing-input-manager `InputState` system to populate
/// `ActionState<LogicalAction>` from keyboard/gamepad input.
// Bevy systems require `Res<T>` by value, not by reference.
#[allow(clippy::needless_pass_by_value)]
fn build_input_map_system(
    keybindings: Res<'_, input::KeybindingsResource>,
    mut commands: Commands<'_, '_>,
) {
    let input_map = input::build_input_map(&keybindings);
    commands.insert_resource(input_map);
    commands.init_resource::<input::ActionState<delta_v_types::LogicalAction>>();
}
