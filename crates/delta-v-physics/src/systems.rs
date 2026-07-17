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

use crate::celestial::{Planet, Sun};
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
    let delta_time = time.delta().as_secs_f32();

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
    let delta_time = time.delta().as_secs_f32();

    for mut body in &mut bodies {
        body.integrate_angular_velocity(delta_time);
    }
}

/// Integrates position and rotation from velocities.
///
/// Updates the entity's `Transform` based on its `RigidBody` velocity
/// and angular velocity.
///
/// Excludes entities with `Sun` or `Planet` components, as those are
/// handled by `sun_rotation_system` and `orbital_motion_system` respectively.
#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub fn integrate_position_system(
    time: Res<'_, Time<Fixed>>,
    mut bodies: Query<'_, '_, (&RigidBody, &mut Transform), (Without<Sun>, Without<Planet>)>,
) {
    let delta_time = time.delta().as_secs_f32();

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

// ---------------------------------------------------------------------------
// Orbital motion system (ADR-0017)
// ---------------------------------------------------------------------------

/// Updates the position of planets along their orbital paths.
///
/// Per ADR-0017, this system runs in `FixedUpdate` at 60 Hz.
///
/// The orbital motion uses circular orbits with optional inclination.
/// Position is computed relative to the parent body's current position.
///
/// Runs in [`PhysicsSet::IntegratePosition`] each fixed tick.
#[allow(
    clippy::needless_pass_by_value,
    clippy::explicit_iter_loop,
    clippy::suboptimal_flops,
    clippy::type_complexity
)]
pub fn orbital_motion_system(
    time: Res<'_, Time<Fixed>>,
    planets: Query<'_, '_, (Entity, &Planet)>,
    // Use ParamSet to separate immutable and mutable access to Transform.
    // The Sun is always at the origin, so the parents query will fail for Sun parents.
    mut transform_set: ParamSet<
        '_,
        '_,
        (
            Query<'_, '_, &Transform, Without<Sun>>,
            Query<'_, '_, &mut Transform, Without<Sun>>,
        ),
    >,
) {
    let elapsed = time.elapsed().as_secs_f32();

    // First pass: collect all planet data
    let planet_data: Vec<(Entity, Entity, f32, f32, f32, f32, f32)> = planets
        .iter()
        .map(|(entity, planet)| {
            (
                entity,
                planet.orbital_parent,
                planet.orbital_distance,
                planet.orbital_period,
                planet.orbital_inclination,
                planet.initial_orbital_angle,
                planet.orbital_eccentricity,
            )
        })
        .collect();

    // Get immutable access to parent positions first
    {
        let parents = transform_set.p0();

        // Pre-compute all parent positions to avoid holding the immutable borrow
        // while we need mutable access later
        let mut parent_positions: Vec<(Entity, Vec3)> = Vec::new();
        for (entity, parent_id, distance, period, inclination, initial_angle, _eccentricity) in
            &planet_data
        {
            // Get parent position. The Sun is always at the origin, so for Sun parents
            // the query will fail and we use Vec3::ZERO. For other parents, query their position.
            let parent_pos = parents
                .get(*parent_id)
                .map_or(Vec3::ZERO, |parent_transform| parent_transform.translation);

            // Compute current angle: initial + (elapsed / period) * TAU
            // This gives us the angle in radians around the orbital circle
            let angle = (elapsed / *period).mul_add(std::f32::consts::TAU, *initial_angle);

            // Compute position in the orbital plane (X-Z plane, Y=0)
            // Then apply inclination: y_offset = distance * sin(inclination) * sin(angle)
            let x_offset = distance * angle.cos();
            let z_offset = distance * angle.sin();
            let y_offset = distance * inclination.sin() * angle.sin();

            let new_pos = parent_pos + Vec3::new(x_offset, y_offset, z_offset);
            parent_positions.push((*entity, new_pos));
        }

        // Now get mutable access and apply updates
        let mut transforms = transform_set.p1();
        for (entity, new_pos) in parent_positions {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = new_pos;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Sun rotation system (ADR-0017)
// ---------------------------------------------------------------------------

/// Rotates suns around their Y axis.
///
/// Per ADR-0017, this system runs in `FixedUpdate` at 60 Hz.
///
/// The rotation period is in hours, converted to seconds for the calculation.
///
/// Runs in [`PhysicsSet::IntegratePosition`] each fixed tick.
#[allow(clippy::needless_pass_by_value, clippy::explicit_iter_loop)]
pub fn sun_rotation_system(
    mut suns: Query<'_, '_, (Entity, &Sun, &mut Transform)>,
    time: Res<'_, Time<Fixed>>,
) {
    let elapsed = time.elapsed().as_secs_f32();

    for (entity, sun, mut transform) in &mut suns {
        let Some(rotation_period_hours) = sun.rotation_period else {
            continue;
        };

        // Convert hours to seconds
        let rotation_period_seconds = rotation_period_hours * 3600.0;

        // Compute rotation angle: (elapsed / period) * TAU
        let angle = (elapsed / rotation_period_seconds) * std::f32::consts::TAU;

        // Set rotation around Y axis
        transform.rotation = Quat::from_axis_angle(Vec3::Y, angle);

        tracing::trace!(
            "Sun {entity:?} rotation: angle={angle:.4} rad, period={rotation_period_hours:.1} h"
        );
    }
}
