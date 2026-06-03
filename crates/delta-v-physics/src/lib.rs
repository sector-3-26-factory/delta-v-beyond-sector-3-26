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

//! Newtonian physics with gravity, floating origin, and avian3d integration.
//!
//! See ADR-0007 (Floating origin), ADR-0009 (Newtonian physics with gravity),
//! and ADR-0017 (Fixed timestep and determinism).

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

pub mod constants;
pub mod rigid_body;
pub mod systems;

pub use constants::{CATCH_UP_TICKS_MAX, FIXED_TIMESTEP_HZ};
pub use rigid_body::{MassSource, RigidBody};
pub use systems::PhysicsSet;

use bevy::prelude::*;
use systems::{
    clear_accumulators_system, gravity_system, integrate_angular_velocity_system,
    integrate_position_system, integrate_velocity_system,
};

/// Physics plugin providing Newtonian dynamics and collision detection.
///
/// Responsibilities:
/// - Configures the fixed timestep schedule at 60 Hz (`FIXED_TIMESTEP_HZ`).
/// - Sets up catch-up logic bounded to `CATCH_UP_TICKS_MAX` ticks per frame.
/// - All physics simulation systems run in `FixedUpdate`, not `Update`.
/// - Provides the [`RigidBody`] component for Newtonian dynamics.
/// - Integrates forces and torques each tick (F=ma, tau=I*alpha).
/// - Computes gravity from [`MassSource`] entities (ADR-0009).
///
/// See ADR-0017 (Fixed timestep and determinism) and ADR-0009 (Newtonian physics).
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        info!("PhysicsPlugin initialized");

        // Configure the fixed timestep schedule at 60 Hz.
        // FixedUpdate runs at this rate; render frames run independently at display rate.
        // Per ADR-0017, catch-up is bounded to prevent runaway.
        app.insert_resource(Time::<Fixed>::from_hz(f64::from(FIXED_TIMESTEP_HZ)));

        // Configure physics system sets for ordered execution.
        app.configure_sets(
            FixedUpdate,
            (
                PhysicsSet::AccumulateForces,
                PhysicsSet::IntegrateVelocity,
                PhysicsSet::IntegratePosition,
                PhysicsSet::ClearAccumulators,
            )
                .chain(),
        );

        // Register physics integration systems in FixedUpdate.
        // Per ADR-0017, gravity contributions are summed in deterministic order
        // (sorted by entity index) within the gravity system itself.
        app.add_systems(
            FixedUpdate,
            (
                gravity_system.in_set(PhysicsSet::AccumulateForces),
                integrate_velocity_system.in_set(PhysicsSet::IntegrateVelocity),
                integrate_angular_velocity_system.in_set(PhysicsSet::IntegrateVelocity),
                integrate_position_system.in_set(PhysicsSet::IntegratePosition),
                clear_accumulators_system.in_set(PhysicsSet::ClearAccumulators),
            ),
        );
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
