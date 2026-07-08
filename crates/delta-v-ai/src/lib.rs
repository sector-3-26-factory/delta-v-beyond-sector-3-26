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

//! AI state machines and behavior systems for driven entities.
//!
//! Provides AI-driven behavior for ships, stations, and other entities.
//! The AI plugin listens for [`delta_v_core::SpawnEntity`] events with
//! `entity_type == "ai_controlled_ship"` and spawns NPC entities with
//! AI state machine components.
//!
//! See ADR-0005 (plugin architecture) and ADR-0017 (Fixed timestep).

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

pub mod components;
pub mod error;
pub mod resources;
pub mod spawn;
pub mod systems;

pub use components::{AiConfig, AiState, AiTask, NpcShip};
pub use error::AiError;
pub use resources::SkirmishState;
pub use systems::AiSet;

use bevy::prelude::*;
use delta_v_core::{AppState, WorldSpawnSet};
use delta_v_physics::PhysicsSet;

/// AI plugin for managing NPC behavior and skirmish tracking.
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkirmishState>()
            .configure_sets(
                Update,
                WorldSpawnSet::SpawnNpcs
                    .after(WorldSpawnSet::SpawnShips)
                    .before(WorldSpawnSet::MarkDebugAxes)
                    .run_if(in_state(AppState::SpawningEntities)),
            )
            .configure_sets(
                FixedUpdate,
                (
                    AiSet::StateMachine,
                    AiSet::FireWeapons,
                    AiSet::SkirmishCheck,
                )
                    .chain()
                    .before(PhysicsSet::AccumulateForces)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    spawn::spawn_npc_ship
                        .in_set(WorldSpawnSet::SpawnNpcs)
                        .run_if(in_state(AppState::SpawningEntities)),
                    delta_v_spawn::mesh_attachment::attach_meshes::<delta_v_spawn::PendingShipMesh>
                        .run_if(in_state(AppState::InGame)),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    systems::ai_state_machine_system.in_set(AiSet::StateMachine),
                    systems::skirmish_check_system.in_set(AiSet::SkirmishCheck),
                ),
            );
    }
}
