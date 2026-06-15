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

//! Weapons systems and projectiles.
//!
//! Provides projectile weapons that inherit the firing ship's velocity
//! and a damage model on rigid bodies (M4).
//!
//! See ADR-0005 (plugin architecture) and ADR-0017 (Fixed timestep).

#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![warn(clippy::all, clippy::pedantic)]
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
pub mod resources;
pub mod spawn;
pub mod systems;

pub use components::Projectile;
pub use resources::WeaponState;
pub use systems::WeaponsSet;

use bevy::prelude::*;
use delta_v_core::{AppState, FireWeapon, InputSet, ProjectileHit};
use delta_v_physics::PhysicsSet;

/// Weapons plugin for managing projectiles and damage.
///
/// Responsibilities:
/// - Registers weapon-related events ([`FireWeapon`], [`ProjectileHit`]).
/// - Configures the [`WeaponsSet`] system sets in `FixedUpdate`.
/// - Spawns projectiles that inherit the source entity's velocity.
/// - Applies damage to entities on projectile collision.
/// - Despawns projectiles after their lifetime expires.
pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeaponState>()
            .add_event::<FireWeapon>()
            .add_event::<ProjectileHit>()
            .configure_sets(
                FixedUpdate,
                (
                    WeaponsSet::ProcessFireCommands,
                    WeaponsSet::ProjectileCollision,
                    WeaponsSet::UpdateProjectiles,
                )
                    .chain()
                    .after(InputSet::Log)
                    .before(PhysicsSet::AccumulateForces)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                (
                    systems::fire_input_system.in_set(WeaponsSet::ProcessFireCommands),
                    systems::process_fire_commands.in_set(WeaponsSet::ProcessFireCommands),
                    systems::projectile_collision_system.in_set(WeaponsSet::ProjectileCollision),
                    systems::update_projectiles.in_set(WeaponsSet::UpdateProjectiles),
                ),
            );
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
