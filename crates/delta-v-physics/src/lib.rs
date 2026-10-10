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
//! See ADR-0007 (Floating origin), ADR-0055 (Keplerian orbits + SOI gravity),
//! and ADR-0017 (Fixed timestep and determinism).

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

pub mod belt_field_generation;
pub mod belt_field_spawn;
pub mod belt_field_streaming;
pub mod celestial;
pub mod collision;
pub mod collision_debug;
pub mod constants;
pub mod floating_origin_systems;
pub mod rigid_body;
pub mod spawn;
pub mod systems;

pub use belt_field_generation::{
    AsteroidDelta, AsteroidDeltaStore, BeltSectorParams, FieldParams, GeneratedAsteroid,
    OrbitalParams, SizeDistributionEntry, StableBodyId, generate_belt_sector, generate_field,
    propagate_displaced_asteroid, separate_overlaps,
};
pub use belt_field_spawn::{
    AsteroidBelt, AsteroidField, spawn_asteroid_belt, spawn_asteroid_field,
};
pub use celestial::{
    LazyLoadMesh, Navigable, OrbitalBody, OrbitalParentId, PendingCelestialMesh, Planet, Sun,
    SunFallbackVfx,
};
pub use celestial::{
    attach_celestial_meshes, lazy_load_celestial_meshes, make_sun_emissive,
    resolve_orbital_parents, sun_fallback_vfx_system, update_sun_fallback_vfx_system,
};
pub use collision::{
    CollisionDetected, CollisionLayersComponent, CollisionShape, CollisionShapeType, DynamicBody,
    StaticBody, distance_to_surface,
};
pub use constants::{CATCH_UP_TICKS_MAX, COLLISION_RELEVANCE_PX, FIXED_TIMESTEP_HZ};
pub use delta_v_types::CollisionLayers;
pub use delta_v_types::collision::layers::ASTEROID_LAYER;
pub use rigid_body::{MassSource, RigidBody};
pub use spawn::{spawn_asteroid, spawn_moon, spawn_planet, spawn_sun};
pub use systems::PhysicsSet;

use bevy::prelude::*;
use delta_v_core::{AppState, FloatingOrigin, FloatingOriginConfig, Health, WorldSpawnSet};
use floating_origin_systems::check_and_recenter_origin_system;
use systems::{
    clear_accumulators_system, gravity_system, integrate_angular_velocity_system,
    integrate_position_system, integrate_velocity_system, moon_rotation_system,
    orbital_motion_system, planet_rotation_system, sun_rotation_system,
};

/// Physics plugin providing Newtonian dynamics and collision detection.
///
/// Responsibilities:
/// - Configures the fixed timestep schedule at 60 Hz (`FIXED_TIMESTEP_HZ`).
/// - Sets up catch-up logic bounded to `CATCH_UP_TICKS_MAX` ticks per frame.
/// - All physics simulation systems run in `FixedUpdate`, not `Update`.
/// - Provides the [`RigidBody`] component for Newtonian dynamics.
/// - Integrates forces and torques each tick (F=ma, tau=I*alpha).
/// - Computes gravity using Sphere of Influence (SOI) model (ADR-0055):
///   * Celestial bodies (Sun, Planet, Moon) follow Keplerian orbits — NO N-body gravity between them.
///   * Dynamic entities (ships, asteroids, debris) experience gravity only from their current SOI parent.
///   * Temporary dynamic attractors (black holes, gravity bombs) via `GravityAttractor` component.
/// - Manages floating origin recentering (ADR-0007).
/// - Integrates avian3d for collision detection (M3).
///
/// See ADR-0017 (Fixed timestep and determinism) and ADR-0055 (Keplerian orbits + SOI gravity).
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
        // Must run BEFORE PhysicsSet::AccumulateForces so that orbital_motion_system (which runs
        // in PhysicsSet::IntegratePosition) doesn't get overwritten by recentering.
        app.add_systems(
            FixedUpdate,
            check_and_recenter_origin_system
                .run_if(in_state(AppState::InGame))
                .before(PhysicsSet::AccumulateForces),
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

        // Unified orbital motion system for all orbiting bodies (planets, moons, etc.).
        // Runs in FixedUpdate to update positions along their orbital paths.
        app.add_systems(
            FixedUpdate,
            orbital_motion_system
                .in_set(PhysicsSet::IntegratePosition)
                .run_if(in_state(AppState::InGame)),
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
                .in_set(WorldSpawnSet::SpawnMoons)
                .run_if(in_state(AppState::SpawningEntities)),
        );
        app.add_systems(
            Update,
            spawn_asteroid
                .in_set(WorldSpawnSet::SpawnAsteroids)
                .run_if(in_state(AppState::SpawningEntities)),
        );
        app.add_systems(
            Update,
            spawn_asteroid_belt
                .in_set(WorldSpawnSet::SpawnAsteroids)
                .run_if(in_state(AppState::SpawningEntities)),
        );
        app.add_systems(
            Update,
            spawn_asteroid_field
                .in_set(WorldSpawnSet::SpawnAsteroids)
                .run_if(in_state(AppState::SpawningEntities)),
        );

        // Resolve orbital parent IDs after all entities are spawned.
        // This must run after the whole spawn chain, not just SpawnSuns and SpawnPlanets:
        // moons spawn in SpawnMoons and asteroids in SpawnAsteroids, and both also carry
        // OrbitalParentId. MarkDebugAxes is the chain terminus, so ordering after it covers
        // every spawn set (ADR-0038 second pass).
        app.add_systems(
            Update,
            resolve_orbital_parents
                .after(WorldSpawnSet::MarkDebugAxes)
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

        // Lazy load celestial body meshes based on distance from camera.
        // This prevents OOM on startup by only loading meshes when near the camera.
        // Per ADR-0055, this is part of the performance optimization for the full
        // solar system with 289 moons.
        app.add_systems(
            Update,
            lazy_load_celestial_meshes.run_if(in_state(AppState::InGame)),
        );

        // 1-pixel sun fallback VFX for when mesh is not loaded but sun is visible.
        // Per ADR-0053, this is a VFX exemption.
        app.add_systems(
            Update,
            (sun_fallback_vfx_system, update_sun_fallback_vfx_system)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/// Whether a body counts as collision-relevant for the pair loop (ADR-0057).
///
/// `None` means the body carries no `LazyLoadMesh` and therefore no relevance value.
/// Ships, projectiles and stations fall in this case and are **always** tested: a body
/// without a number is never treated as irrelevant, so weapons, player flight and
/// station docking cannot regress.
///
/// This is a distance test. It reads `current_screen_radius_px`, which
/// `lazy_load_celestial_meshes` derives from distance alone, and never consults a
/// camera, a frustum or the player's view direction.
fn is_collision_relevant(lazy_load: Option<&LazyLoadMesh>) -> bool {
    lazy_load.is_none_or(|lazy| lazy.current_screen_radius_px >= lazy.collision_relevance_px)
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
///
/// Per ADR-0057 the pair loop is pre-filtered by collision relevance. This is a
/// distance test and not a visibility test: no camera, frustum or view direction
/// takes part in it. Bodies without a `LazyLoadMesh` have no relevance value and
/// are always tested, so ships, projectiles and stations are unaffected.
// `type_complexity` is allowed here only because adding `Option<&LazyLoadMesh>`
// for ADR-0057 pushed this query past the lint's element threshold. Narrower than
// introducing a type alias that would exist for a single signature.
#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
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
            Option<&LazyLoadMesh>,
        ),
    >,
    health_query: Query<'_, '_, &Health>,
) {
    let _span = tracing::info_span!("delta_v_physics::collision_detection_system").entered();

    let bodies_vec: Vec<_> = bodies.iter().collect();

    // ADR-0057: partition ONCE, before any pair is examined. Deciding relevance
    // inside the pair loop was measured at only ~3.4x, because the inner loop still
    // ran n^2/2 times and skipped only the shape test. Partitioning up front means an
    // irrelevant asteroid is never walked against another irrelevant asteroid at all.
    //
    // A body with no LazyLoadMesh has no relevance value and counts as relevant, so it
    // belongs in `relevant_asteroids` or `others` on its own terms, never skipped.
    let mut relevant_asteroids: Vec<BodyRef<'_>> = Vec::new();
    let mut irrelevant_asteroids: Vec<BodyRef<'_>> = Vec::new();
    let mut others: Vec<BodyRef<'_>> = Vec::new();

    for body in &bodies_vec {
        // Destructure the collected query tuple to reach the layers and the optional
        // relevance component. `*body` copies only the references.
        let (_, _, _, _, layers, lazy_load) = *body;
        let is_asteroid = (layers.layers & ASTEROID_LAYER) != 0;
        if is_asteroid {
            if is_collision_relevant(lazy_load) {
                relevant_asteroids.push(*body);
            } else {
                irrelevant_asteroids.push(*body);
            }
        } else {
            others.push(*body);
        }
    }

    // Four tested groups. The two skipped groups — relevant x irrelevant and
    // irrelevant x irrelevant — are simply never formed.
    test_pairs(
        &relevant_asteroids,
        &relevant_asteroids,
        &mut events,
        &health_query,
    );
    test_pairs(&relevant_asteroids, &others, &mut events, &health_query);
    test_pairs(&irrelevant_asteroids, &others, &mut events, &health_query);
    test_pairs(&others, &others, &mut events, &health_query);
}

/// One body's data as borrowed from the collision query.
type BodyRef<'w> = (
    Entity,
    &'w RigidBody,
    &'w Transform,
    &'w CollisionShape,
    &'w CollisionLayersComponent,
    Option<&'w LazyLoadMesh>,
);

/// Tests every unordered pair drawn from two lists and emits a message per overlap.
///
/// `a` and `b` may be the same list, in which case each unordered pair is tested once.
/// Pairs whose collision layers do not permit them are skipped without a shape test.
fn test_pairs(
    a: &[BodyRef<'_>],
    b: &[BodyRef<'_>],
    events: &mut MessageWriter<'_, CollisionDetected>,
    health_query: &Query<'_, '_, &Health>,
) {
    for (index, body_a) in a.iter().enumerate() {
        // When both lists are the same one, start past `index` so each pair is
        // visited once rather than twice. `index` comes from `enumerate` over `a`,
        // and the slice is the same one, so `index + 1` is in bounds; `get` is
        // used anyway because ADR-0023 denies `indexing_slicing`.
        let rest = if std::ptr::eq(a.as_ptr(), b.as_ptr()) {
            b.get(index + 1..).unwrap_or_default()
        } else {
            b
        };

        for body_b in rest {
            let (entity_a, body_a, transform_a, shape_a, layers_a, _) = *body_a;
            let (entity_b, body_b, transform_b, shape_b, layers_b, _) = *body_b;

            // Collision layers: entity A can collide with B if B's layer is in A's
            // mask and A's layer is in B's mask.
            let can_collide =
                (layers_b.layers & layers_a.mask) != 0 && (layers_a.layers & layers_b.mask) != 0;
            if !can_collide {
                continue;
            }

            // Apply collision shape offset to get the actual collision center. The
            // offset is expressed in the body's local frame, so it has to be rotated
            // by the body's rotation before it is added to the translation.
            let pos_a = transform_a.translation + transform_a.rotation * shape_a.offset;
            let pos_b = transform_b.translation + transform_b.rotation * shape_b.offset;

            if let Some((normal, penetration_depth)) = check_collision(
                pos_a,
                transform_a.rotation,
                shape_a,
                pos_b,
                transform_b.rotation,
                shape_b,
            ) {
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
/// The returned normal always points from body A towards body B.
///
/// For sphere-sphere, uses the sum of radii.
/// For sphere-box, the sphere centre is taken into the box's local frame, where the
/// clamp test is exact.
/// For box-box, the 15-axis separating axis theorem is used.
///
/// Both rotations are REQUIRED. A box is an oriented shape: testing it against the
/// world axes while ignoring `rotation_a` / `rotation_b` makes a pitched ship stop
/// colliding with bodies that its drawn collision shape visibly overlaps.
#[allow(clippy::too_many_lines)]
fn check_collision(
    pos_a: Vec3,
    rotation_a: Quat,
    shape_a: &CollisionShape,
    pos_b: Vec3,
    rotation_b: Quat,
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
        // Box-box: exact oriented test via the 15-axis separating axis theorem
        (
            CollisionShapeType::Box { half_extents: he_a },
            CollisionShapeType::Box { half_extents: he_b },
        ) => oriented_box_overlap(pos_a, rotation_a, *he_a, pos_b, rotation_b, *he_b),
        // Sphere-box: exact test in the box's local frame
        (CollisionShapeType::Sphere { radius }, CollisionShapeType::Box { half_extents })
        | (CollisionShapeType::Box { half_extents }, CollisionShapeType::Sphere { radius }) => {
            // Ensure sphere is first for uniform handling
            let (sphere_pos, box_pos, box_rotation, is_sphere_a) =
                if matches!(shape_a.shape_type, CollisionShapeType::Sphere { .. }) {
                    (pos_a, pos_b, rotation_b, true)
                } else {
                    (pos_b, pos_a, rotation_a, false)
                };

            let he = *half_extents;

            // Rotate the sphere centre into the box's own frame. There the box is
            // axis-aligned, so the clamp below measures the true distance from the
            // sphere centre to the oriented box surface.
            let local = box_rotation.inverse() * (sphere_pos - box_pos);
            let closest = Vec3::new(
                local.x.clamp(-he.x, he.x),
                local.y.clamp(-he.y, he.y),
                local.z.clamp(-he.z, he.z),
            );

            let diff = local - closest;
            let dist_sq = diff.length_squared();

            if dist_sq >= radius * radius {
                return None;
            }

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

            // Calculate normal direction, in the box's local frame
            let local_normal = if dist > f32::EPSILON {
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

            // Rotating the local normal back to world space preserves its meaning:
            // it still points from the box surface towards the sphere centre.
            let box_to_sphere = box_rotation * local_normal;

            // Normal should point from target (A) to other (B)
            // The calculated normal points from box to sphere
            // If sphere is A and box is B: normal points from B to A (wrong direction)
            // If box is A and sphere is B: normal points from A to B (correct direction)
            if is_sphere_a {
                // Sphere is A, Box is B: negate to point from A to B
                Some((-box_to_sphere, penetration))
            } else {
                // Box is A, Sphere is B: keep as-is (already points from A to B)
                Some((box_to_sphere, penetration))
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

/// The three local axes of an oriented box, expressed in world space.
struct BoxAxes {
    /// World-space direction of the box's local X axis.
    x: Vec3,
    /// World-space direction of the box's local Y axis.
    y: Vec3,
    /// World-space direction of the box's local Z axis.
    z: Vec3,
}

impl BoxAxes {
    /// Builds the axis set from a body's rotation.
    fn from_rotation(rotation: Quat) -> Self {
        Self {
            x: rotation * Vec3::X,
            y: rotation * Vec3::Y,
            z: rotation * Vec3::Z,
        }
    }

    /// Radius of the box's shadow on a unit `axis`.
    fn projection_radius(&self, half_extents: Vec3, axis: Vec3) -> f32 {
        let along_y = half_extents.y * self.y.dot(axis).abs();
        let along_x = half_extents.x.mul_add(self.x.dot(axis).abs(), along_y);
        half_extents.z.mul_add(self.z.dot(axis).abs(), along_x)
    }
}

/// Cross products of near-parallel edges collapse towards zero and say nothing
/// about separation, so they are dropped instead of being normalised.
const PARALLEL_EDGE_EPSILON: f32 = 1.0e-6;

/// Exact oriented-box overlap test using the 15-axis separating axis theorem.
///
/// Six face normals (three per box) plus nine edge cross products are the complete
/// candidate set for two convex boxes. The deepest overlap among them is the minimum
/// translation distance, and its axis is the collision normal.
///
/// Returns `None` as soon as any candidate axis separates the boxes, which is the
/// common case and ends the search early.
fn oriented_box_overlap(
    pos_a: Vec3,
    rotation_a: Quat,
    half_extents_a: Vec3,
    pos_b: Vec3,
    rotation_b: Quat,
    half_extents_b: Vec3,
) -> Option<(Vec3, f32)> {
    let axes_a = BoxAxes::from_rotation(rotation_a);
    let axes_b = BoxAxes::from_rotation(rotation_b);
    let delta = pos_b - pos_a;

    // Deepest overlap seen so far, as (depth, axis).
    let mut deepest: Option<(f32, Vec3)> = None;

    // Six face normals.
    for axis in [axes_a.x, axes_a.y, axes_a.z, axes_b.x, axes_b.y, axes_b.z] {
        if !consider_separating_axis(
            axis,
            &axes_a,
            half_extents_a,
            &axes_b,
            half_extents_b,
            delta,
            &mut deepest,
        ) {
            return None;
        }
    }

    // Nine edge cross products.
    for edge_a in [axes_a.x, axes_a.y, axes_a.z] {
        for edge_b in [axes_b.x, axes_b.y, axes_b.z] {
            let edge_cross = edge_a.cross(edge_b);
            if edge_cross.length_squared() <= PARALLEL_EDGE_EPSILON {
                continue;
            }
            if !consider_separating_axis(
                edge_cross.normalize(),
                &axes_a,
                half_extents_a,
                &axes_b,
                half_extents_b,
                delta,
                &mut deepest,
            ) {
                return None;
            }
        }
    }

    let (depth, axis) = deepest?;
    // `delta` runs from A to B, so orient the axis the same way.
    let normal = if delta.dot(axis) < 0.0 { -axis } else { axis };
    Some((normal, depth))
}

/// Tests one candidate separating axis and records it when it is the deepest yet.
///
/// Returns `false` when the axis separates the boxes, which ends the search.
fn consider_separating_axis(
    axis: Vec3,
    axes_a: &BoxAxes,
    half_extents_a: Vec3,
    axes_b: &BoxAxes,
    half_extents_b: Vec3,
    delta: Vec3,
    deepest: &mut Option<(f32, Vec3)>,
) -> bool {
    let reach = axes_a.projection_radius(half_extents_a, axis)
        + axes_b.projection_radius(half_extents_b, axis);
    let overlap = reach - delta.dot(axis).abs();

    if overlap <= 0.0 {
        return false;
    }

    match *deepest {
        Some((best, _)) if overlap >= best => {}
        _ => *deepest = Some((overlap, axis)),
    }
    true
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
                .map_or(0.0, |(body, _)| 1.0 / body.mass)
        };
        let inv_mass_b = if response.other_is_static {
            0.0
        } else {
            all_bodies
                .get(response.other)
                .map_or(0.0, |(body, _)| 1.0 / body.mass)
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
                // The offset is local to the body: rotate it into world space before
                // adding it to the translation, and rotate it back afterwards.
                let local_offset = transform_a.rotation * shape_a.offset;
                let pos_a = transform_a.translation + local_offset;
                // Normal points from A to B, so move A in opposite direction (away from B)
                let corrected_pos = pos_a - normal * correction_magnitude * inv_mass_a;
                transform_a.translation = corrected_pos - local_offset;
            }
            if let Ok(shape_b) = shapes.get(response.other)
                && !response.other_is_static
                && let Ok((_, mut transform_b)) = all_bodies.get_mut(response.other)
            {
                let local_offset = transform_b.rotation * shape_b.offset;
                let pos_b = transform_b.translation + local_offset;
                // Move B in the direction of the normal (away from A)
                let corrected_pos = pos_b + normal * correction_magnitude * inv_mass_b;
                transform_b.translation = corrected_pos - local_offset;
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

#[cfg(all(test, feature = "bench"))]
mod approach_comparison;
