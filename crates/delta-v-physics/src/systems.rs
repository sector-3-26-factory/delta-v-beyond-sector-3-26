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

//! Physics integration systems for fixed-timestep Newtonian simulation.
//!
//! These systems run in the `FixedUpdate` schedule at a fixed 60 Hz rate.
//! They implement F=ma and tau=I*alpha integration (ADR-0017).

use crate::constants::{GRAVITATIONAL_CONSTANT, GRAVITY_CUTOFF_RADIUS_M};
use crate::rigid_body::{MassSource, RigidBody};
use bevy::prelude::*;

/// System set for physics integration, allowing ordered execution.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsSet {
    /// Accumulate forces and torques (gravity, thrust, input).
    AccumulateForces,
    /// Integrate velocities from accumulated forces.
    IntegrateVelocity,
    /// Integrate positions and rotations from velocities.
    IntegratePosition,
    /// Clear accumulators for the next tick.
    ClearAccumulators,
}

// ---------------------------------------------------------------------------
// Gravity system (ADR-0009, ADR-0017)
// ---------------------------------------------------------------------------

/// Computes gravitational forces from [`MassSource`] entities and accumulates
/// them on all rigid bodies within the cutoff radius.
///
/// Per ADR-0009, gravity sources generate `F = G * M * m / r²` on all bodies
/// within [`GRAVITY_CUTOFF_RADIUS_M`].
///
/// Per ADR-0017, gravity contributions are summed in a fixed, stable order
/// (sorted by entity index) to ensure determinism.
///
/// Runs in [`PhysicsSet::AccumulateForces`] each fixed tick.
#[allow(clippy::needless_pass_by_value)]
pub fn gravity_system(
    sources: Query<'_, '_, (Entity, &RigidBody, &Transform), With<MassSource>>,
    mut bodies: Query<'_, '_, (Entity, &mut RigidBody, &Transform), Without<MassSource>>,
) {
    // Collect source entity IDs and sort for deterministic ordering (ADR-0017).
    let mut source_ids: Vec<Entity> = sources.iter().map(|(e, _, _)| e).collect();
    source_ids.sort_by_key(|e| e.index());

    // Collect body entity IDs and sort for deterministic ordering (ADR-0017).
    let mut body_ids: Vec<Entity> = bodies.iter().map(|(e, _, _)| e).collect();
    body_ids.sort_by_key(|e| e.index());

    let cutoff_sq = GRAVITY_CUTOFF_RADIUS_M * GRAVITY_CUTOFF_RADIUS_M;

    for source_id in &source_ids {
        let Ok((_, source_body, source_transform)) = sources.get(*source_id) else {
            continue;
        };
        let source_pos = source_transform.translation;
        let source_mass = source_body.mass;

        for body_id in &body_ids {
            let Ok((_, mut body, transform)) = bodies.get_mut(*body_id) else {
                continue;
            };

            let delta = source_pos - transform.translation;
            let dist_sq = delta.length_squared();

            // Skip self-interaction and bodies beyond cutoff.
            if dist_sq < f32::EPSILON || dist_sq > cutoff_sq {
                continue;
            }

            // F = G * M * m / r^2, direction: toward source
            let dist = dist_sq.sqrt();
            let force_magnitude = GRAVITATIONAL_CONSTANT * source_mass * body.mass / dist_sq;
            let force_direction = delta / dist;
            let force = force_direction * force_magnitude;

            body.apply_force(force);
        }
    }
}

// ---------------------------------------------------------------------------
// Integration systems
// ---------------------------------------------------------------------------

/// Integrates linear velocity from accumulated forces using F = ma.
///
/// Runs each fixed timestep. Per ADR-0009, every rigid body MUST have a
/// positive mass; [`RigidBody::new`] panics on non-positive mass, so this
/// system only ever sees valid bodies.
#[allow(clippy::needless_pass_by_value)]
pub fn integrate_velocity_system(
    time: Res<'_, Time<Fixed>>,
    mut bodies: Query<'_, '_, &mut RigidBody>,
) {
    let delta_time = time.delta_seconds();

    for mut body in &mut bodies {
        body.integrate_velocity(delta_time);
    }
}

/// Integrates angular velocity from accumulated torques using tau = I * alpha.
///
/// Runs each fixed timestep.
#[allow(clippy::needless_pass_by_value)]
pub fn integrate_angular_velocity_system(
    time: Res<'_, Time<Fixed>>,
    mut bodies: Query<'_, '_, &mut RigidBody>,
) {
    let delta_time = time.delta_seconds();

    for mut body in &mut bodies {
        body.integrate_angular_velocity(delta_time);
    }
}

/// Integrates position and rotation from velocities.
///
/// Updates the entity's `Transform` based on its `RigidBody` velocity
/// and angular velocity.
#[allow(clippy::needless_pass_by_value)]
pub fn integrate_position_system(
    mut bodies: Query<'_, '_, (&RigidBody, &mut Transform)>,
    time: Res<'_, Time<Fixed>>,
) {
    let delta_time = time.delta_seconds();

    for (body, mut transform) in &mut bodies {
        // Linear integration: p += v * dt
        transform.translation += body.velocity * delta_time;

        // Angular integration: rotate by omega * dt (using axis-angle representation)
        let rotation_angle = body.angular_velocity.length() * delta_time;
        if rotation_angle > 0.0 {
            let rotation_axis = body.angular_velocity.normalize();
            let delta_rotation = Quat::from_axis_angle(rotation_axis, rotation_angle);
            transform.rotation = delta_rotation * transform.rotation;
        }
    }
}

/// Clears force and torque accumulators after integration.
///
/// Runs last each tick to prepare for the next tick's input.
pub fn clear_accumulators_system(mut bodies: Query<'_, '_, &mut RigidBody>) {
    for mut body in &mut bodies {
        body.clear_accumulators();
    }
}
