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

//! Newtonian physics with gravity, floating origin, and avian3d integration.
//!
//! See ADR-0007 (Floating origin), ADR-0009 (Newtonian physics with gravity),
//! and ADR-0017 (Fixed timestep and determinism).

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

pub mod celestial;
pub mod collision;
pub mod collision_debug;
pub mod constants;
pub mod floating_origin_systems;
pub mod rigid_body;
pub mod spawn;
pub mod systems;

pub use celestial::{Navigable, OrbitalParentId, PendingCelestialMesh, Planet, Sun};
pub use celestial::{attach_celestial_meshes, make_sun_emissive, resolve_orbital_parents};
pub use collision::{
    CollisionDetected, CollisionLayersComponent, CollisionShape, CollisionShapeType, DynamicBody,
    StaticBody, distance_to_surface,
};
pub use constants::{CATCH_UP_TICKS_MAX, FIXED_TIMESTEP_HZ};
pub use delta_v_types::CollisionLayers;
pub use rigid_body::{MassSource, RigidBody};
pub use spawn::{spawn_asteroid, spawn_moon, spawn_planet, spawn_sun};
pub use systems::PhysicsSet;

use bevy::prelude::*;
use delta_v_core::{AppState, FloatingOrigin, FloatingOriginConfig, Health, WorldSpawnSet};
use floating_origin_systems::check_and_recenter_origin_system;
use systems::{
    clear_accumulators_system, gravity_system, integrate_angular_velocity_system,
    integrate_position_system, integrate_velocity_system, moon_orbital_motion_system,
    moon_rotation_system, orbital_motion_system, planet_rotation_system, sun_rotation_system,
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
    #[allow(clippy::too_many_lines)]
    fn build(&self, app: &mut App) {
        info!("PhysicsPlugin initialized");

        // Configure the fixed timestep schedule at 60 Hz.
        // FixedUpdate runs at this rate; render frames run independently at display rate.
        // Per ADR-0017, catch-up is bounded to prevent runaway.
        app.insert_resource(Time::<Fixed>::from_hz(f64::from(FIXED_TIMESTEP_HZ)))
            .add_message::<CollisionDetected>();

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

        // Orbital motion system for planets.
        // Runs in FixedUpdate to update planet positions along their orbital paths.
        app.add_systems(
            FixedUpdate,
            orbital_motion_system
                .in_set(PhysicsSet::IntegratePosition)
                .run_if(in_state(AppState::InGame)),
        );

        // Moon orbital motion system.
        // Runs in FixedUpdate to update moon positions along their orbital paths.
        // Must run AFTER orbital_motion_system so parent planets have updated positions.
        app.add_systems(
            FixedUpdate,
            moon_orbital_motion_system
                .in_set(PhysicsSet::IntegratePosition)
                .run_if(in_state(AppState::InGame))
                .after(orbital_motion_system),
        );

        // Sun rotation system.
        // Runs in FixedUpdate to rotate suns around their Y axis.
        app.add_systems(
            FixedUpdate,
            sun_rotation_system
                .in_set(PhysicsSet::IntegratePosition)
                .run_if(in_state(AppState::InGame)),
        );

        // Planet rotation system.
        // Runs in FixedUpdate to rotate planets around their tilted axis.
        app.add_systems(
            FixedUpdate,
            planet_rotation_system
                .in_set(PhysicsSet::IntegratePosition)
                .run_if(in_state(AppState::InGame)),
        );

        // Moon rotation system.
        // Runs in FixedUpdate to rotate moons around their tilted axis.
        app.add_systems(
            FixedUpdate,
            moon_rotation_system
                .in_set(PhysicsSet::IntegratePosition)
                .run_if(in_state(AppState::InGame)),
        );

        // Collision shape debug visualization
        app.add_systems(
            Update,
            (
                collision_debug::spawn_collision_shape_debug.run_if(in_state(AppState::InGame)),
                collision_debug::update_collision_shape_debug_color
                    .run_if(in_state(AppState::InGame)),
            ),
        );

        // Celestial body spawning systems.
        // Per ADR-0038, these run in SpawningEntities state in dependency order.
        app.add_systems(
            Update,
            spawn_sun
                .in_set(WorldSpawnSet::SpawnSuns)
                .run_if(in_state(AppState::SpawningEntities)),
        );
        app.add_systems(
            Update,
            spawn_planet
                .in_set(WorldSpawnSet::SpawnPlanets)
                .run_if(in_state(AppState::SpawningEntities)),
        );
        app.add_systems(
            Update,
            spawn_moon
                .in_set(WorldSpawnSet::SpawnPlanets)
                .run_if(in_state(AppState::SpawningEntities)),
        );
        app.add_systems(
            Update,
            spawn_asteroid
                .in_set(WorldSpawnSet::SpawnAsteroids)
                .run_if(in_state(AppState::SpawningEntities)),
        );

        // Resolve orbital parent IDs after all entities are spawned.
        // This must run after SpawnSuns and SpawnPlanets.
        app.add_systems(
            Update,
            resolve_orbital_parents
                .after(WorldSpawnSet::SpawnPlanets)
                .run_if(in_state(AppState::SpawningEntities)),
        );

        // Attach celestial body meshes once glTF assets are loaded.
        // Uses the generic attach_meshes system from delta-v-spawn (ADR-0047).
        app.add_systems(
            Update,
            attach_celestial_meshes.run_if(in_state(AppState::InGame)),
        );

        // Make sun meshes emissive after they are loaded.
        // Per ADR-0053, this is a VFX exemption for realistic sun rendering.
        // Must run after attach_celestial_meshes so the meshes exist.
        app.add_systems(
            Update,
            make_sun_emissive
                .run_if(in_state(AppState::InGame))
                .after(attach_celestial_meshes),
        );
    }
}

/// System that detects collisions and emits [`CollisionDetected`] events.
///
/// Performs shape-aware collision detection:
/// - Sphere-sphere: uses sum of radii.
/// - Box-box and sphere-box: computes the closest points on the axis-aligned
///   bounding boxes and checks if they overlap.
///
/// For box shapes, the collision check uses the actual box geometry rather than
/// a sphere approximation, preventing false positives in thin axes and false
/// negatives in wide axes.
///
/// Collision layers are checked to filter out non-colliding entity pairs.
#[allow(clippy::needless_pass_by_value)]
fn collision_detection_system(
    mut events: MessageWriter<'_, CollisionDetected>,
    bodies: Query<
        '_,
        '_,
        (
            Entity,
            &RigidBody,
            &Transform,
            &CollisionShape,
            &CollisionLayersComponent,
        ),
    >,
    health_query: Query<'_, '_, &Health>,
) {
    let bodies_vec: Vec<_> = bodies.iter().collect();
    let len = bodies_vec.len();

    for i in 0..len {
        for j in (i + 1)..len {
            // SAFETY: i and j are valid indices from the loop bounds
            #[allow(clippy::indexing_slicing)]
            let (entity_a, body_a, transform_a, shape_a, layers_a) = bodies_vec[i];
            #[allow(clippy::indexing_slicing)]
            let (entity_b, body_b, transform_b, shape_b, layers_b) = bodies_vec[j];

            // Check collision layers: entity A can collide with B if B's layer is in A's mask
            // and A's layer is in B's mask
            let can_collide =
                (layers_b.layers & layers_a.mask) != 0 && (layers_a.layers & layers_b.mask) != 0;
            if !can_collide {
                continue;
            }

            // Apply collision shape offset to get the actual collision center
            let pos_a = transform_a.translation + shape_a.offset;
            let pos_b = transform_b.translation + shape_b.offset;

            let collision = check_collision(pos_a, shape_a, pos_b, shape_b);

            if let Some((normal, penetration_depth)) = collision {
                let relative_velocity = body_b.velocity - body_a.velocity;

                let health_a = health_query.get(entity_a).map_or_else(
                    |_| "no-hp".to_string(),
                    |h| format!("hp={:.0}/{:.0}", h.current, h.max),
                );
                let health_b = health_query.get(entity_b).map_or_else(
                    |_| "no-hp".to_string(),
                    |h| format!("hp={:.0}/{:.0}", h.current, h.max),
                );
                tracing::debug!(
                    "Collision detected: {entity_a:?} ({health_a}) <-> {entity_b:?} ({health_b}), penetration={penetration_depth:.2}, normal={normal:?}"
                );

                events.write(CollisionDetected {
                    target: entity_a,
                    other: entity_b,
                    point: pos_a + normal * penetration_depth * 0.5,
                    normal,
                    relative_velocity,
                    penetration_depth,
                });
            }
        }
    }
}

/// Checks if two collision shapes overlap.
///
/// Returns `Some((normal, penetration_depth))` if colliding, `None` otherwise.
///
/// For sphere-sphere, uses the sum of radii.
/// For box-box and sphere-box, computes overlap along each axis.
#[allow(clippy::too_many_lines)]
fn check_collision(
    pos_a: Vec3,
    shape_a: &CollisionShape,
    pos_b: Vec3,
    shape_b: &CollisionShape,
) -> Option<(Vec3, f32)> {
    match (&shape_a.shape_type, &shape_b.shape_type) {
        // Sphere-sphere: simple distance check
        (
            CollisionShapeType::Sphere { radius: r_a },
            CollisionShapeType::Sphere { radius: r_b },
        ) => {
            let delta = pos_b - pos_a;
            let distance = delta.length();
            let sum_radii = r_a + r_b;
            if distance < sum_radii && distance > f32::EPSILON {
                let normal = delta / distance;
                Some((normal, sum_radii - distance))
            } else if distance < f32::EPSILON {
                // Centers coincide; use arbitrary normal
                Some((Vec3::Z, sum_radii))
            } else {
                None
            }
        }
        // Box-box: compute overlap along each axis
        (
            CollisionShapeType::Box { half_extents: he_a },
            CollisionShapeType::Box { half_extents: he_b },
        ) => {
            // For each axis, compute the gap between the two boxes.
            // If any gap is positive (no overlap), there's no collision.
            let delta = pos_b - pos_a;

            let overlap_x = (he_a.x + he_b.x) - delta.x.abs();
            let overlap_y = (he_a.y + he_b.y) - delta.y.abs();
            let overlap_z = (he_a.z + he_b.z) - delta.z.abs();

            // If any overlap is negative, the boxes are separated along that axis
            if overlap_x <= 0.0 || overlap_y <= 0.0 || overlap_z <= 0.0 {
                return None;
            }

            // Find the axis of minimum penetration (the collision normal)
            // This is the axis where the boxes are least overlapping
            if overlap_x <= overlap_y && overlap_x <= overlap_z {
                let normal = if delta.x > 0.0 { Vec3::X } else { Vec3::NEG_X };
                Some((normal, overlap_x))
            } else if overlap_y <= overlap_x && overlap_y <= overlap_z {
                let normal = if delta.y > 0.0 { Vec3::Y } else { Vec3::NEG_Y };
                Some((normal, overlap_y))
            } else {
                let normal = if delta.z > 0.0 { Vec3::Z } else { Vec3::NEG_Z };
                Some((normal, overlap_z))
            }
        }
        // Sphere-box: check if sphere center is within expanded box
        (CollisionShapeType::Sphere { radius }, CollisionShapeType::Box { half_extents })
        | (CollisionShapeType::Box { half_extents }, CollisionShapeType::Sphere { radius }) => {
            // Ensure sphere is first for uniform handling
            let (sphere_pos, box_pos, he, is_sphere_a) =
                if matches!(shape_a.shape_type, CollisionShapeType::Sphere { .. }) {
                    (pos_a, pos_b, *half_extents, true)
                } else {
                    (pos_b, pos_a, *half_extents, false)
                };

            // Find closest point on box surface to sphere center
            let local = sphere_pos - box_pos;
            let closest = Vec3::new(
                local.x.clamp(-he.x, he.x),
                local.y.clamp(-he.y, he.y),
                local.z.clamp(-he.z, he.z),
            );

            let diff = local - closest;
            let dist_sq = diff.length_squared();

            if dist_sq < radius * radius {
                let dist = dist_sq.sqrt();

                // Calculate penetration depth
                let penetration = if dist > f32::EPSILON {
                    // Sphere is outside the box
                    *radius - dist
                } else {
                    // Sphere center is inside the box
                    // Find the distance to the nearest face
                    let dx = he.x - local.x.abs();
                    let dy = he.y - local.y.abs();
                    let dz = he.z - local.z.abs();
                    let min_dist = dx.min(dy).min(dz);
                    min_dist + *radius
                };

                // Calculate normal direction
                let normal = if dist > f32::EPSILON {
                    // Sphere is outside: normal points from box surface to sphere center
                    diff / dist
                } else {
                    // Sphere center is inside the box
                    // Find which face is closest and point outward
                    let dx = he.x - local.x.abs();
                    let dy = he.y - local.y.abs();
                    let dz = he.z - local.z.abs();

                    if dx <= dy && dx <= dz {
                        // Closest to x face
                        if local.x > 0.0 { Vec3::X } else { Vec3::NEG_X }
                    } else if dy <= dz {
                        // Closest to y face
                        if local.y > 0.0 { Vec3::Y } else { Vec3::NEG_Y }
                    } else {
                        // Closest to z face
                        if local.z > 0.0 { Vec3::Z } else { Vec3::NEG_Z }
                    }
                };

                // Normal should point from target (A) to other (B)
                // The calculated normal points from box to sphere
                // If sphere is A and box is B: normal points from B to A (wrong direction)
                // If box is A and sphere is B: normal points from A to B (correct direction)
                if is_sphere_a {
                    // Sphere is A, Box is B: negate to point from A to B
                    Some((-normal, penetration))
                } else {
                    // Box is A, Sphere is B: keep as-is (already points from A to B)
                    Some((normal, penetration))
                }
            } else {
                None
            }
        }
        // ConvexHull not yet implemented; fall back to sphere approximation
        _ => {
            let delta = pos_b - pos_a;
            let distance = delta.length();
            let radius_a = get_collision_radius(shape_a);
            let radius_b = get_collision_radius(shape_b);
            let sum_radii = radius_a + radius_b;

            if distance < sum_radii && distance > f32::EPSILON {
                let normal = delta / distance;
                Some((normal, sum_radii - distance))
            } else {
                None
            }
        }
    }
}

/// Collision response data collected from events.
///
/// Stored for deferred processing to avoid double mutable borrow of the query.
struct CollisionResponse {
    /// The target entity (first body).
    target: Entity,
    /// The other entity (second body).
    other: Entity,
    /// Collision normal pointing from target to other.
    normal: Vec3,
    /// Penetration depth of the collision.
    penetration_depth: f32,
    /// Velocity of the target body along the collision normal.
    vel_target: f32,
    /// Velocity of the other body along the collision normal.
    vel_other: f32,
    /// Whether the target is a static body.
    target_is_static: bool,
    /// Whether the other is a static body.
    other_is_static: bool,
}

/// System that responds to collision events by applying impulses and position correction.
///
/// Handles three collision scenarios:
/// - **Dynamic vs Static**: Applies impulse to the dynamic body only (e.g., ship hitting asteroid).
/// - **Dynamic vs Dynamic**: Applies impulse to both bodies, split by mass (e.g., ship-to-ship).
///
/// Position correction is applied to both bodies to prevent overlap. The correction
/// is proportional to the inverse mass of each body (lighter bodies move more).
///
/// Runs in `FixedUpdate` BEFORE velocity integration, so impulses affect current frame.
#[allow(clippy::needless_pass_by_value)]
fn collision_response_system(
    mut events: MessageReader<'_, '_, CollisionDetected>,
    mut all_bodies: Query<'_, '_, (&mut RigidBody, &mut Transform)>,
    static_markers: Query<'_, '_, Entity, With<StaticBody>>,
    shapes: Query<'_, '_, &CollisionShape>,
) {
    const RESTITUTION: f32 = 0.5;
    const POSITION_CORRECTION_PERCENT: f32 = 0.8;
    const POSITION_CORRECTION_SLOP: f32 = 0.01;

    // Collect collision data first to avoid double mutable borrow
    let mut responses: Vec<CollisionResponse> = Vec::new();

    // First pass: read events and collect body data
    for collision in events.read() {
        let target_is_static = static_markers.get(collision.target).is_ok();
        let other_is_static = static_markers.get(collision.other).is_ok();

        // Skip static-static collisions (neither should move)
        if target_is_static && other_is_static {
            continue;
        }

        // Read body data (immutable borrow is fine here)
        let Ok((body_a, _)) = all_bodies.get(collision.target) else {
            continue;
        };
        let Ok((body_b, _)) = all_bodies.get(collision.other) else {
            continue;
        };

        let normal = collision.normal;
        let vel_target = body_a.velocity.dot(normal);
        let vel_other = body_b.velocity.dot(normal);

        responses.push(CollisionResponse {
            target: collision.target,
            other: collision.other,
            normal,
            penetration_depth: collision.penetration_depth,
            vel_target,
            vel_other,
            target_is_static,
            other_is_static,
        });
    }

    // Second pass: apply impulses and position correction
    for response in &responses {
        let normal = response.normal;

        // Relative velocity of other with respect to target along normal
        let vel_along_normal = response.vel_other - response.vel_target;

        // Get inverse masses
        let inv_mass_a = if response.target_is_static {
            0.0
        } else {
            // SAFETY: We already validated the entity exists in the first pass
            all_bodies
                .get(response.target)
                .map(|(body, _)| 1.0 / body.mass)
                .unwrap_or(0.0)
        };
        let inv_mass_b = if response.other_is_static {
            0.0
        } else {
            all_bodies
                .get(response.other)
                .map(|(body, _)| 1.0 / body.mass)
                .unwrap_or(0.0)
        };
        let total_inv_mass = inv_mass_a + inv_mass_b;

        if total_inv_mass < f32::EPSILON {
            continue;
        }

        // Apply position correction first to separate overlapping bodies
        if response.penetration_depth > POSITION_CORRECTION_SLOP {
            let correction_magnitude = (response.penetration_depth - POSITION_CORRECTION_SLOP)
                * POSITION_CORRECTION_PERCENT
                / total_inv_mass;

            if let Ok(shape_a) = shapes.get(response.target)
                && !response.target_is_static
                && let Ok((_, mut transform_a)) = all_bodies.get_mut(response.target)
            {
                let pos_a = transform_a.translation + shape_a.offset;
                // Normal points from A to B, so move A in opposite direction (away from B)
                let corrected_pos = pos_a - normal * correction_magnitude * inv_mass_a;
                transform_a.translation = corrected_pos - shape_a.offset;
            }
            if let Ok(shape_b) = shapes.get(response.other)
                && !response.other_is_static
                && let Ok((_, mut transform_b)) = all_bodies.get_mut(response.other)
            {
                let pos_b = transform_b.translation + shape_b.offset;
                // Move B in the direction of the normal (away from A)
                let corrected_pos = pos_b + normal * correction_magnitude * inv_mass_b;
                transform_b.translation = corrected_pos - shape_b.offset;
            }
        }

        // Don't resolve impulse if velocities are separating
        if vel_along_normal > 0.0 {
            continue;
        }

        // Compute impulse magnitude using the standard collision response formula
        // j = -(1 + e) * v_rel · n / (1/m_a + 1/m_b)
        let impulse_magnitude = -(1.0 + RESTITUTION) * vel_along_normal / total_inv_mass;
        let impulse = normal * impulse_magnitude;

        // Apply impulse to both bodies (Newton's third law)
        if let Ok((mut body_a, _)) = all_bodies.get_mut(response.target)
            && !response.target_is_static
        {
            body_a.apply_impulse(-impulse);
        }
        if let Ok((mut body_b, _)) = all_bodies.get_mut(response.other)
            && !response.other_is_static
        {
            body_b.apply_impulse(impulse);
        }

        tracing::debug!(
            "Collision response: target={:?}, other={:?}, impulse_mag={:.2}, static={}/{}",
            response.target,
            response.other,
            impulse_magnitude,
            response.target_is_static,
            response.other_is_static,
        );
    }
}

/// Extracts the collision radius from a [`CollisionShape`].
///
/// For sphere shapes, returns the radius directly.
/// For box shapes, returns the maximum half-extent as an approximation.
fn get_collision_radius(shape: &CollisionShape) -> f32 {
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

#[cfg(test)]
#[path = "collision_tests.rs"]
mod collision_tests;

#[cfg(test)]
#[path = "integration_tests.rs"]
mod integration_tests;

#[cfg(test)]
#[path = "rigid_body_panic_tests.rs"]
mod rigid_body_panic_tests;
