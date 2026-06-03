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

//! Ship types and ship-specific systems.
//!
//! Manages the spawning and lifecycle of ship entities. Listens for
//! [`delta_v_world::SpawnEntity`] events and spawns ships from templates.
//!
//! The input → forces pipeline (D3) runs in `FixedUpdate`:
//! 1. Input reader: [`ActiveActions`] → [`ThrustCommand`] + [`TorqueCommand`]
//! 2. Flight-assist toggle: [`LogicalAction::ToggleFlightAssist`] action → flip state
//! 3. Thrust system: [`ThrustCommand`] → `apply_force` on [`RigidBody`]
//! 4. Torque system: [`TorqueCommand`] → `apply_torque` on [`RigidBody`]
//! 5. Flight-assist damping: if enabled, damp velocity
//! 6. Clear commands: zero out command buffers
//!
//! See ADR-0005 (plugin architecture), ADR-0038 (entity template system),
//! and ADR-0017 (Fixed timestep).

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

pub mod spawn;
pub mod systems;

pub use spawn::{setup_scene_lighting, spawn_ship_from_template};

#[cfg(test)]
#[path = "spawn_tests.rs"]
mod spawn_tests;

use bevy::prelude::*;
use delta_v_core::{AppState, InputSet, ThrustCommand, TorqueCommand, WorldSpawnSet};
use delta_v_physics::PhysicsSet;
use systems::{
    clear_commands_system, flight_assist_damping_system, flight_assist_toggle_system,
    input_reader_system, thrust_system, torque_system, ShipInputSet,
};

/// Ships plugin for managing player and NPC vessels.
///
/// Listens for [`delta_v_world::SpawnEntity`] events during
/// [`AppState::SpawningEntities`] and spawns ship entities based on
/// their `entity_type` field.
///
/// The plugin also sets up the input → forces pipeline in `FixedUpdate`
/// during `InGame`, and sets up scene lighting on world load.
pub struct ShipsPlugin;

impl Plugin for ShipsPlugin {
    fn build(&self, app: &mut App) {
        // Per-tick command buffers (cleared each tick by clear_commands_system).
        // allow-default: Bevy requires Default on resources for init_resource.
        // These are per-tick command buffers, not configuration.
        app.init_resource::<ThrustCommand>()
            .init_resource::<TorqueCommand>();

        // Configure WorldSpawnSet ordering (ADR-0038).
        app.configure_sets(
            Update,
            (
                WorldSpawnSet::SpawnSuns,
                WorldSpawnSet::SpawnPlanets,
                WorldSpawnSet::SpawnMoons,
                WorldSpawnSet::SpawnStations,
                WorldSpawnSet::SpawnAsteroids,
                WorldSpawnSet::SpawnShips,
            )
                .chain()
                .run_if(in_state(AppState::SpawningEntities)),
        )
        .add_systems(OnEnter(AppState::SpawningEntities), setup_scene_lighting)
        .add_systems(
            Update,
            spawn_ship_from_template
                .in_set(WorldSpawnSet::SpawnShips)
                .run_if(in_state(AppState::SpawningEntities)),
        )
        .add_systems(
            Update,
            advance_to_in_game
                .after(WorldSpawnSet::SpawnShips)
                .run_if(in_state(AppState::SpawningEntities)),
        )
        // Attach glTF meshes during InGame once assets are loaded.
        .add_systems(
            Update,
            spawn::attach_ship_meshes.run_if(in_state(AppState::InGame)),
        );

        // Input → Forces pipeline in FixedUpdate (ADR-0017).
        // Must run after InputSet::Translate (which populates ActiveActions)
        // and before PhysicsSet::AccumulateForces (which includes gravity).
        app.configure_sets(
            FixedUpdate,
            (
                ShipInputSet::AccumulateCommands,
                ShipInputSet::ToggleFlightAssist,
                ShipInputSet::ApplyThrust,
                ShipInputSet::ApplyTorque,
                ShipInputSet::FlightAssistDamping,
                ShipInputSet::ClearCommands,
            )
                .chain()
                .after(InputSet::Translate)
                .before(PhysicsSet::AccumulateForces)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            FixedUpdate,
            (
                input_reader_system.in_set(ShipInputSet::AccumulateCommands),
                flight_assist_toggle_system.in_set(ShipInputSet::ToggleFlightAssist),
                thrust_system.in_set(ShipInputSet::ApplyThrust),
                torque_system.in_set(ShipInputSet::ApplyTorque),
                flight_assist_damping_system.in_set(ShipInputSet::FlightAssistDamping),
                clear_commands_system.in_set(ShipInputSet::ClearCommands),
            ),
        );
    }
}

/// Advances the state machine to [`AppState::InGame`] after all spawn
/// systems in [`WorldSpawnSet::SpawnShips`] have run.
///
/// This runs in the same `Update` tick as the spawn systems, but after
/// them (via `.after(WorldSpawnSet::SpawnShips)`). Because all
/// [`delta_v_world::SpawnEntity`] events are emitted synchronously on
/// [`AppState::SpawningEntities`] entry, a single `Update` tick is
/// sufficient to process them all before advancing.
fn advance_to_in_game(mut next: ResMut<'_, NextState<AppState>>) {
    next.set(AppState::InGame);
}
