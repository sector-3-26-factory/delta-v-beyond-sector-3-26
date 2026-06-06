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

pub mod collision;
pub mod constants;
pub mod floating_origin_systems;
pub mod rigid_body;
pub mod systems;

pub use collision::{
    CollisionDetected, CollisionLayers, CollisionShape, CollisionShapeType, DynamicBody, StaticBody,
};
pub use constants::{CATCH_UP_TICKS_MAX, FIXED_TIMESTEP_HZ};
pub use rigid_body::{MassSource, RigidBody};
pub use systems::PhysicsSet;

use bevy::prelude::*;
use delta_v_core::{AppState, FloatingOrigin, FloatingOriginConfig};
use floating_origin_systems::{check_and_recenter_origin_system, mark_new_entities_system};
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
/// - Manages floating origin recentering (ADR-0007).
/// - Integrates avian3d for collision detection (M3).
///
/// See ADR-0017 (Fixed timestep and determinism) and ADR-0009 (Newtonian physics).
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        info!("PhysicsPlugin initialized");

        // Configure the fixed timestep schedule at 60 Hz.
        // FixedUpdate runs at this rate; render frames run independently at display rate.
        // Per ADR-0017, catch-up is bounded to prevent runaway.
        app.insert_resource(Time::<Fixed>::from_hz(f64::from(FIXED_TIMESTEP_HZ)))
            .add_event::<CollisionDetected>();

        // Initialize floating origin resources
        app.init_resource::<FloatingOrigin>()
            .init_resource::<FloatingOriginConfig>();

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

        // Floating origin recentering runs in FixedUpdate, before physics.
        // This ensures positions are relative to the current origin before forces are applied.
        app.add_systems(
            FixedUpdate,
            check_and_recenter_origin_system.run_if(in_state(AppState::InGame)),
        );

        // Mark new entities as eligible for floating origin translation.
        // Runs in Update during SpawningEntities state, after all domain spawning.
        app.add_systems(
            Update,
            mark_new_entities_system.run_if(in_state(AppState::SpawningEntities)),
        );

        // Collision detection using avian3d.
        // Runs in FixedUpdate after physics integration.
        app.add_systems(
            FixedUpdate,
            collision_detection_system.run_if(in_state(AppState::InGame)),
        );
    }
}

/// System that detects collisions and emits [`CollisionDetected`] events.
///
/// This is a placeholder that will be expanded when avian3d collision
/// is fully integrated. For now, it checks for overlapping bounding boxes
/// as a simple collision test.
#[allow(clippy::needless_pass_by_value)]
fn collision_detection_system(
    mut events: EventWriter<'_, CollisionDetected>,
    bodies: Query<'_, '_, (Entity, &RigidBody, &Transform, &CollisionShape)>,
) {
    let bodies_vec: Vec<_> = bodies.iter().collect();
    let len = bodies_vec.len();

    for i in 0..len {
        for j in (i + 1)..len {
            // SAFETY: i and j are valid indices from the loop bounds
            #[allow(clippy::indexing_slicing)]
            let (entity_a, body_a, transform_a, shape_a) = bodies_vec[i];
            #[allow(clippy::indexing_slicing)]
            let (entity_b, body_b, transform_b, shape_b) = bodies_vec[j];

            let delta = transform_b.translation - transform_a.translation;
            let distance = delta.length();

            // Get radii from collision shapes
            let radius_a = get_collision_radius(shape_a);
            let radius_b = get_collision_radius(shape_b);

            if distance < radius_a + radius_b {
                let normal = delta.normalize_or_zero();
                let relative_velocity = body_b.velocity - body_a.velocity;

                events.send(CollisionDetected {
                    target: entity_a,
                    other: entity_b,
                    point: transform_a.translation + normal * radius_a,
                    normal,
                    relative_velocity,
                });
            }
        }
    }
}

/// Extracts the collision radius from a [`CollisionShape`].
///
/// For sphere shapes, returns the radius directly.
/// For box shapes, returns the maximum half-extent as an approximation.
const fn get_collision_radius(shape: &CollisionShape) -> f32 {
    match shape.shape_type {
        CollisionShapeType::Sphere { radius } => radius,
        CollisionShapeType::Box { half_extents } => {
            half_extents.x.max(half_extents.y).max(half_extents.z)
        }
        CollisionShapeType::ConvexHull => {
            // TODO: Implement convex hull radius calculation
            1.0
        }
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
