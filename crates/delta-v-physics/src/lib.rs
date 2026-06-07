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
        // Runs in FixedUpdate BEFORE physics integration, so we can respond to collisions.
        app.add_systems(
            FixedUpdate,
            collision_detection_system
                .in_set(PhysicsSet::AccumulateForces)
                .run_if(in_state(AppState::InGame)),
        );

        // Collision response applies impulses to dynamic bodies.
        // Runs in FixedUpdate BEFORE velocity integration, so impulses affect current frame.
        app.add_systems(
            FixedUpdate,
            collision_response_system
                .in_set(PhysicsSet::AccumulateForces)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/// System that detects collisions and emits [`CollisionDetected`] events.
///
/// Uses sphere-based collision detection. When two entities with collision
/// shapes overlap, emits a `CollisionDetected` event.
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

                log::debug!(
                    "Collision detected: entities {:?} and {:?}, distance={:.2}, radii={:.2}+{:.2}={:.2}",
                    entity_a,
                    entity_b,
                    distance,
                    radius_a,
                    radius_b,
                    radius_a + radius_b
                );

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

/// System that responds to collision events by applying impulses.
///
/// When a dynamic body (ship) collides with a static body (asteroid),
/// applies an impulse to the dynamic body to simulate a bounce.
///
/// Runs in `FixedUpdate` BEFORE velocity integration, so impulses affect current frame.
#[allow(clippy::needless_pass_by_value)]
fn collision_response_system(
    mut events: EventReader<'_, '_, CollisionDetected>,
    mut dynamic_bodies: Query<'_, '_, (&mut RigidBody, &Transform), Without<StaticBody>>,
    static_bodies: Query<'_, '_, &RigidBody, With<StaticBody>>,
    static_markers: Query<'_, '_, Entity, With<StaticBody>>,
) {
    for collision in events.read() {
        // Check if one body is static (asteroid) and the other is dynamic (ship)
        let target_is_static = static_markers.get(collision.target).is_ok();
        let other_is_static = static_markers.get(collision.other).is_ok();

        if !target_is_static && !other_is_static {
            // Both are dynamic - skip (or could implement for NPC ships)
            continue;
        }

        // Determine which is the dynamic body
        let dynamic_entity = if target_is_static {
            collision.other
        } else {
            collision.target
        };

        // Get the static body's velocity
        let static_entity = if target_is_static {
            collision.target
        } else {
            collision.other
        };
        let Ok(static_body) = static_bodies.get(static_entity) else {
            continue;
        };
        let static_velocity = static_body.velocity;

        // Get the dynamic body (mutable for applying impulse)
        let Ok((mut dynamic_body, _)) = dynamic_bodies.get_mut(dynamic_entity) else {
            continue;
        };

        // Compute impulse based on relative velocity and restitution.
        // Impulse formula: j = -(1 + e) * m * (v_rel · n)
        // where e = coefficient of restitution, m = mass of dynamic body,
        // v_rel = v_dynamic - v_static, n = normal (pointing from static to dynamic)
        // The velocity change is: delta_v = j / m = -(1 + e) * (v_rel · n)
        //
        // The normal in the collision event points from target to other.
        // We need the normal pointing from static body to dynamic body.
        let restitution = 0.5;

        // Relative velocity of dynamic body with respect to static body
        let relative_velocity = dynamic_body.velocity - static_velocity;

        // Normal points from target to other. We need the normal pointing from
        // static body to dynamic body to push the dynamic body away.
        let normal = if target_is_static {
            collision.normal // normal points from static (target) to dynamic (other)
        } else {
            -collision.normal // normal points from dynamic (target) to static (other)
        };

        // Impulse magnitude includes mass for correct physics
        let impulse =
            normal * (-(1.0 + restitution) * dynamic_body.mass * relative_velocity.dot(normal));

        log::debug!(
            "Collision response: entity={:?}, impulse={:?}, mass={}",
            dynamic_entity,
            impulse,
            dynamic_body.mass
        );

        dynamic_body.apply_impulse(impulse);
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
