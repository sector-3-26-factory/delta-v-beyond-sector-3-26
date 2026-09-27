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

use crate::celestial::{Moon, OrbitalBody, Planet, Sun};
use crate::constants::GRAVITATIONAL_CONSTANT;
use crate::rigid_body::{MassSource, RigidBody};
use bevy::prelude::*;
use delta_v_core::{FloatingOrigin, PlayerShipEntity, WorldEntityId};

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
// Gravity system (ADR-0055: Keplerian orbits + SOI gravity)
// ---------------------------------------------------------------------------

/// Computes gravitational forces using Sphere of Influence (SOI) model.
///
/// Per ADR-0055:
/// - Celestial bodies (Sun, Planet, Moon) follow Keplerian orbits — NO N-body gravity between them.
/// - Dynamic entities (ships, asteroids, debris) experience gravity ONLY from their current SOI parent.
/// - SOI radius: `r_soi = a * (m/M)^(2/5)` where `a` = orbital distance, `m` = body mass, `M` = parent mass.
/// - Special objects (black holes, gravity bombs) may act as temporary dynamic point-attractors via `GravityAttractor` component.
///
/// Runs in [`PhysicsSet::AccumulateForces`] each fixed tick.
#[allow(clippy::needless_pass_by_value)]
pub fn gravity_system(
    // Celestial bodies that are gravity sources (have SOI)
    celestial_sources: Query<
        '_,
        '_,
        (Entity, &RigidBody, &Transform, &OrbitalBody),
        (With<MassSource>, Or<(With<Sun>, With<Planet>, With<Moon>)>),
    >,
    // Dynamic entities that receive gravity (ships, asteroids, debris) - NO MassSource, NO GravityAttractor
    mut dynamic_bodies: Query<
        '_,
        '_,
        (Entity, &mut RigidBody, &Transform),
        (
            Without<MassSource>,
            Without<Sun>,
            Without<Planet>,
            Without<Moon>,
            Without<GravityAttractor>,
        ),
    >,
    // Optional: temporary dynamic gravity attractors (black holes, gravity bombs, etc.)
    attractors: Query<
        '_,
        '_,
        (Entity, &RigidBody, &Transform, &GravityAttractor),
        Without<MassSource>,
    >,
) {
    // Collect celestial sources with their SOI radii
    let mut sources: Vec<(Entity, Vec3, f32, f32)> = Vec::new(); // (entity, position, mass, soi_radius)

    for (entity, body, transform, orbital) in &celestial_sources {
        let source_pos = transform.translation;
        let source_mass = body.mass;

        // Compute SOI radius: r_soi = a * (m/M)^(2/5)
        // We need the parent's mass. For the Sun, SOI is effectively infinite (or very large).
        // For planets/moons, we need to find the parent's mass.
        // For now, use a simplified approach: SOI = orbital_distance * (mass / parent_mass)^(2/5)
        // Since we don't have parent mass easily accessible, use a configurable default or compute from orbital params.
        // Simplified: SOI radius proportional to orbital distance and mass ratio.
        // For the Sun (no parent), use a very large SOI.
        // For planets/moons, we'll compute it if we can access parent mass.

        // TODO: Proper SOI computation requires parent mass. For now, use a large default for all.
        // In practice, we can compute SOI when we have the orbital hierarchy resolved.
        let soi_radius = compute_soi_radius(entity, source_mass, orbital, &celestial_sources);

        sources.push((entity, source_pos, source_mass, soi_radius));
    }

    // Sort sources for deterministic ordering (ADR-0017)
    sources.sort_by_key(|(e, _, _, _)| e.index());

    // Collect dynamic body IDs and sort for deterministic ordering
    let mut body_ids: Vec<Entity> = dynamic_bodies.iter().map(|(e, _, _)| e).collect();
    body_ids.sort_by_key(|e| e.index());

    // For each dynamic body, find its SOI parent and apply gravity from that single source
    for body_id in &body_ids {
        let Ok((_, mut body, transform)) = dynamic_bodies.get_mut(*body_id) else {
            continue;
        };
        let body_pos = transform.translation;

        // Find which SOI this body is in
        let mut soi_parent: Option<(Vec3, f32)> = None; // (parent_position, parent_mass)
        let mut min_soi_ratio = f32::INFINITY;

        for (_source_entity, source_pos, source_mass, soi_radius) in &sources {
            let delta = *source_pos - body_pos;
            let dist_sq = delta.length_squared();
            let dist = dist_sq.sqrt();

            // Check if body is within this source's SOI
            if dist < *soi_radius {
                // Use distance/SOI ratio to find the "most dominant" SOI (closest to center of its SOI)
                let soi_ratio = dist / *soi_radius;
                if soi_ratio < min_soi_ratio {
                    min_soi_ratio = soi_ratio;
                    soi_parent = Some((*source_pos, *source_mass));
                }
            }
        }

        // Also check temporary attractors
        for (_, attractor_body, attractor_transform, attractor) in &attractors {
            let delta = attractor_transform.translation - body_pos;
            let dist_sq = delta.length_squared();
            let dist = dist_sq.sqrt();

            if dist < attractor.radius && dist > f32::EPSILON {
                // Attractor overrides or adds to SOI gravity
                let force_magnitude =
                    GRAVITATIONAL_CONSTANT * attractor_body.mass * body.mass / dist_sq;
                let force_direction = delta / dist;
                let force = force_direction * force_magnitude * attractor.strength_multiplier;
                body.apply_force(force);
            }
        }

        // Apply gravity from SOI parent (single source)
        if let Some((parent_pos, parent_mass)) = soi_parent {
            let delta = parent_pos - body_pos;
            let dist_sq = delta.length_squared();

            if dist_sq > f32::EPSILON {
                let dist = dist_sq.sqrt();
                let force_magnitude = GRAVITATIONAL_CONSTANT * parent_mass * body.mass / dist_sq;
                let force_direction = delta / dist;
                let force = force_direction * force_magnitude;
                body.apply_force(force);
            }
        }
    }
}

/// Computes the Sphere of Influence radius for a celestial body.
///
/// SOI formula: r_soi = a * (m/M)^(2/5)
/// where a = orbital distance (semi-major axis), m = body mass, M = parent mass.
///
/// For the Sun (no parent), returns a very large radius (effectively infinite).
/// For planets/moons, computes based on parent mass if available.
fn compute_soi_radius(
    _entity: Entity,
    mass: f32,
    orbital: &OrbitalBody,
    celestial_sources: &Query<
        '_,
        '_,
        (Entity, &RigidBody, &Transform, &OrbitalBody),
        (With<MassSource>, Or<(With<Sun>, With<Planet>, With<Moon>)>),
    >,
) -> f32 {
    // If this body has no orbital parent (e.g., the Sun), SOI is effectively infinite
    if orbital.orbital_parent == Entity::PLACEHOLDER || orbital.orbital_distance <= 0.0 {
        return f32::MAX; // Sun's SOI encompasses everything
    }

    // Try to find parent mass
    if let Ok((_, parent_body, _, _)) = celestial_sources.get(orbital.orbital_parent) {
        let parent_mass = parent_body.mass;
        if parent_mass > 0.0 && mass > 0.0 {
            let mass_ratio = mass / parent_mass;
            let soi_radius = orbital.orbital_distance * mass_ratio.powf(0.4); // (2/5) = 0.4
            return soi_radius.max(1000.0); // Minimum 1km SOI
        }
    }

    // Fallback: use a fraction of orbital distance
    (orbital.orbital_distance * 0.1).max(1000.0)
}

/// Component for temporary dynamic gravity attractors (black holes, gravity bombs, tractor beams).
///
/// Per ADR-0055, these are gameplay objects that temporarily act as point-attractors.
/// They do NOT re-introduce celestial N-body physics.
// allow-default: GravityAttractor is a runtime Component for procedural VFX/gameplay objects
// (black holes, gravity bombs, tractor beams) spawned via code, not deserialized from JSON.
// Defaults are sensible fallbacks for programmatic spawning, not silent fallbacks for missing JSON.
#[derive(Component, Clone, Debug, Default)]
pub struct GravityAttractor {
    /// Radius of influence in metres.
    pub radius: f32,
    /// Strength multiplier (1.0 = normal gravity, >1.0 = stronger, <1.0 = weaker).
    pub strength_multiplier: f32,
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
/// Excludes entities with `Sun`, `Planet`, or `Moon` components, as those are
/// handled by `sun_rotation_system`, `orbital_motion_system`, and `moon_rotation_system` respectively.
#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub fn integrate_position_system(
    time: Res<'_, Time<Fixed>>,
    mut bodies: Query<
        '_,
        '_,
        (&RigidBody, &mut Transform),
        (Without<Sun>, Without<Planet>, Without<Moon>),
    >,
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
// Unified orbital motion system (ADR-0017)
// ---------------------------------------------------------------------------

/// Updates the position of all orbiting bodies along their orbital paths.
///
/// Per ADR-0017, this system runs in `FixedUpdate` at 60 Hz.
///
/// The orbital motion uses circular orbits with optional inclination.
/// Position is computed relative to the parent body's current position.
/// This single system handles both planets and moons (and any other orbiting bodies)
/// via the unified `OrbitalBody` component.
///
/// Runs in [`PhysicsSet::IntegratePosition`] each fixed tick.
#[allow(
     clippy::needless_pass_by_value, // Bevy system parameters require pass-by-value
     clippy::explicit_iter_loop, // Manual iteration needed for ParamSet borrow splitting
     clippy::suboptimal_flops, // Trigonometric operations are inherent to orbital mechanics
     clippy::type_complexity // ParamSet with two queries is required for borrow separation
 )]
pub fn orbital_motion_system(
    time: Res<'_, Time<Fixed>>,
    orbital_bodies: Query<'_, '_, (Entity, &OrbitalBody)>,
    // Use ParamSet to separate immutable and mutable access to Transform.
    // Include all potential parents (Sun, Planet, Moon) in the parent query.
    mut transform_set: ParamSet<
        '_,
        '_,
        (
            Query<'_, '_, &Transform>,
            Query<'_, '_, &mut Transform, Without<Sun>>,
        ),
    >,
) {
    let elapsed = time.elapsed().as_secs_f32();

    // First pass: collect all orbital body data
    let orbital_data: Vec<(Entity, Entity, f32, f32, f32, f32)> = orbital_bodies
        .iter()
        .map(|(entity, orbital_body)| {
            (
                entity,
                orbital_body.orbital_parent,
                orbital_body.orbital_distance,
                orbital_body.orbital_period,
                orbital_body.orbital_inclination,
                orbital_body.initial_orbital_angle,
            )
        })
        .collect();

    // Get immutable access to parent positions first
    let parents = transform_set.p0();

    // Pre-compute all parent positions to avoid holding the immutable borrow
    // while we need mutable access later
    let mut parent_positions: Vec<(Entity, Vec3)> = Vec::new();
    for (entity, parent_id, distance, period, inclination, initial_angle) in &orbital_data {
        // Get parent position. All potential parents (Sun, Planet, Moon) are included in the query,
        // so we get their actual position after floating origin recentering and orbital motion.
        // If parent is not found, skip this entity (it has no valid orbital parent).
        let Ok(parent_transform) = parents.get(*parent_id) else {
            continue;
        };
        let parent_pos = parent_transform.translation;

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

/// Rotates planets around their tilted axis, accounting for orbital inclination.
///
/// Per ADR-0017, this system runs in `FixedUpdate` at 60 Hz.
///
/// The rotation period is in seconds (stored in the Planet component).
/// The axial tilt is in radians (angle between rotation axis and orbital axis).
/// The orbital inclination is in radians (angle between orbital plane and ecliptic).
///
/// The planet's rotation axis is computed as:
/// 1. Start with orbital axis (Y, perpendicular to ecliptic)
/// 2. Tilt by `orbital_inclination` around X axis (orbital plane inclination)
/// 3. Tilt by `axial_tilt` around the line of nodes (X axis, intersection of orbital plane with ecliptic)
///    This assumes longitude of ascending node = 0 for simplicity.
///
/// Runs in [`PhysicsSet::IntegratePosition`] each fixed tick.
#[allow(clippy::needless_pass_by_value, clippy::explicit_iter_loop)]
pub fn planet_rotation_system(
    mut planets: Query<'_, '_, (Entity, &Planet, &OrbitalBody, &mut Transform)>,
    time: Res<'_, Time<Fixed>>,
) {
    let elapsed = time.elapsed().as_secs_f32();

    for (entity, planet, orbital_body, mut transform) in &mut planets {
        let Some(rotation_period_seconds) = planet.rotation_period else {
            continue;
        };

        // Compute rotation angle: (elapsed / period) * TAU
        let angle = (elapsed / rotation_period_seconds) * std::f32::consts::TAU;

        // Orbital axis: start with Y (ecliptic normal), tilt by orbital_inclination around X
        let orbital_axis =
            Quat::from_axis_angle(Vec3::X, orbital_body.orbital_inclination) * Vec3::Y;

        // Planet's rotation axis: tilt orbital axis by axial_tilt around the line of nodes.
        // The line of nodes is the intersection of the orbital plane with the ecliptic plane.
        // Since we incline the orbital plane around the X axis, the X axis lies in both planes
        // and is the line of nodes (assuming longitude of ascending node = 0).
        // This gives a physically consistent tilt direction independent of orbital inclination.
        let tilt_axis = Vec3::X;
        let rotation_axis = Quat::from_axis_angle(tilt_axis, planet.axial_tilt) * orbital_axis;

        // Set rotation around tilted axis
        transform.rotation = Quat::from_axis_angle(rotation_axis, angle);

        tracing::trace!(
            "Planet {entity:?} rotation: angle={angle:.4} rad, period={rotation_period_seconds:.1} s, axial_tilt={:.4} rad, orbital_inclination={:.4} rad",
            planet.axial_tilt,
            orbital_body.orbital_inclination
        );
    }
}

/// Rotates moons around their tilted axis, accounting for orbital inclination.
///
/// Per ADR-0017, this system runs in `FixedUpdate` at 60 Hz.
///
/// The rotation period is in seconds (stored in the Moon component).
/// The axial tilt is in radians (angle between rotation axis and orbital axis).
/// The orbital inclination is in radians (angle between orbital plane and ecliptic).
///
/// The moon's rotation axis is computed as:
/// 1. Start with orbital axis (Y, perpendicular to ecliptic)
/// 2. Tilt by `orbital_inclination` around X axis (orbital plane inclination)
/// 3. Tilt by `axial_tilt` around the line of nodes (X axis, intersection of orbital plane with ecliptic)
///    This assumes longitude of ascending node = 0 for simplicity.
///
/// Runs in [`PhysicsSet::IntegratePosition`] each fixed tick.
#[allow(clippy::needless_pass_by_value, clippy::explicit_iter_loop)]
pub fn moon_rotation_system(
    mut moons: Query<'_, '_, (Entity, &Moon, &OrbitalBody, &mut Transform)>,
    time: Res<'_, Time<Fixed>>,
) {
    let elapsed = time.elapsed().as_secs_f32();

    for (entity, moon, orbital_body, mut transform) in &mut moons {
        let Some(rotation_period_seconds) = moon.rotation_period else {
            continue;
        };

        // Compute rotation angle: (elapsed / period) * TAU
        let angle = (elapsed / rotation_period_seconds) * std::f32::consts::TAU;

        // Orbital axis: start with Y (ecliptic normal), tilt by orbital_inclination around X
        let orbital_axis =
            Quat::from_axis_angle(Vec3::X, orbital_body.orbital_inclination) * Vec3::Y;

        // Moon's rotation axis: tilt orbital axis by axial_tilt around the line of nodes.
        // The line of nodes is the intersection of the orbital plane with the ecliptic plane.
        // Since we incline the orbital plane around the X axis, the X axis lies in both planes
        // and is the line of nodes (assuming longitude of ascending node = 0).
        // This gives a physically consistent tilt direction independent of orbital inclination.
        let tilt_axis = Vec3::X;
        let rotation_axis = Quat::from_axis_angle(tilt_axis, moon.axial_tilt) * orbital_axis;

        // Set rotation around tilted axis
        transform.rotation = Quat::from_axis_angle(rotation_axis, angle);

        tracing::trace!(
            "Moon {entity:?} rotation: angle={angle:.4} rad, period={rotation_period_seconds:.1} s, axial_tilt={:.4} rad, orbital_inclination={:.4} rad",
            moon.axial_tilt,
            orbital_body.orbital_inclination
        );
    }
}
