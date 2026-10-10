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

//! Deterministic asteroid generation for belts and fields.
//!
//! This module implements the procedural generation of asteroids from belt and field
//! templates. Generation is fully deterministic: given the same seed, belt/field ID,
//! sector index, and body index, the same asteroid properties are produced every time.
//!
//! Per the asteroid belt plan (plans/asteroid-belt-plan.md Phase 7):
//! - Each property (mesh, radius, rotation) draws from its own hash stream
//! - Sector identity comes from orbital position, not a running counter
//! - Analytic propagation at sector load, not rigid rotation
//! - Overlap separation at load time
//! - Detail level filtering as a deterministic keep-test

use bevy::prelude::*;
use delta_v_types::{
    AsteroidBeltTemplate, AsteroidFieldTemplate, CollisionShapeData, CollisionShapeType,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A generated asteroid's properties before spawning.
#[derive(Debug, Clone)]
pub struct GeneratedAsteroid {
    /// Stable unique ID for this asteroid (seed, belt/field ID, sector, body index).
    pub stable_id: u64,
    /// World-space position at the current simulation time.
    pub position: Vec3,
    /// World-space rotation.
    pub rotation: Quat,
    /// Radius in metres.
    pub radius: f32,
    /// Mass in kilograms (density * 4/3 * pi * r^3).
    pub mass: f32,
    /// Mesh template name to use.
    pub mesh_name: String,
    /// Collision shape data (scaled to radius).
    pub collision_shape: CollisionShapeData,
    /// Bounding box (scaled to radius).
    pub bounding_box: delta_v_types::BoundingBox,
    /// Orbital parameters for Keplerian motion.
    pub orbital_params: Option<OrbitalParams>,
}

/// Orbital parameters for a generated asteroid.
#[derive(Debug, Clone)]
pub struct OrbitalParams {
    /// Name of the orbital parent body.
    pub orbital_parent: String,
    /// Orbital distance (semi-major axis) in metres.
    pub orbital_distance: f32,
    /// Orbital period in seconds.
    pub orbital_period: f32,
    /// Orbital eccentricity (0 = circular).
    pub orbital_eccentricity: f32,
    /// Orbital inclination in radians.
    pub orbital_inclination: f32,
    /// Initial orbital angle (mean anomaly at epoch) in radians.
    pub initial_orbital_angle: f32,
}

/// Parameters for generating a belt sector.
#[derive(Debug, Clone)]
pub struct BeltSectorParams {
    /// Unique identifier for the belt.
    pub belt_id: String,
    /// Sector index along the belt's orbital ring.
    pub sector_index: i64,
    /// Template with generation parameters (meshes, density).
    pub template: AsteroidBeltTemplate,
    /// Name of the orbital parent body.
    pub orbital_parent: String,
    /// Orbital distance (semi-major axis) in metres.
    pub orbital_distance: f32,
    /// Orbital period in seconds.
    pub orbital_period: f32,
    /// Orbital eccentricity (0 = circular).
    pub orbital_eccentricity: f32,
    /// Orbital inclination in radians.
    pub orbital_inclination: f32,
    /// Angular size of each sector in radians.
    pub sector_size: f32,
    /// Radial extent of the belt in metres.
    pub radial_extent: f32,
    /// Vertical thickness of the belt in metres.
    pub thickness: f32,
    /// Mean spacing between asteroids in metres.
    pub mean_spacing: f32,
    /// Excluded radii (Kirkwood gaps) in the belt.
    pub excluded_radii: Vec<ExcludedRadius>,
    /// Size distribution of asteroids in the belt.
    pub size_distribution: Vec<SizeDistributionEntry>,
    /// Material density of asteroids in kg/m³.
    pub density: f32,
    /// Random seed for deterministic generation.
    pub seed: u64,
    /// Current simulation time in seconds.
    pub simulation_time: f32,
    /// Detail level (0.0 to 1.0) for LOD filtering.
    pub detail_level: f32,
}

/// Parameters for generating a field.
#[derive(Debug, Clone)]
pub struct FieldParams {
    /// Unique identifier for the field.
    pub field_id: String,
    /// Template with generation parameters (meshes, density).
    pub template: AsteroidFieldTemplate,
    /// Name of the orbital parent body (if orbiting).
    pub orbital_parent: Option<String>,
    /// Orbital distance (semi-major axis) in metres (if orbiting).
    pub orbital_distance: Option<f32>,
    /// Orbital period in seconds (if orbiting).
    pub orbital_period: Option<f32>,
    /// Spatial extent of the field in metres (cube side length for box shape).
    pub extent: f32,
    /// Mean spacing between asteroids in metres.
    pub mean_spacing: f32,
    /// Size distribution of asteroids in the field.
    pub size_distribution: Vec<SizeDistributionEntry>,
    /// Material density of asteroids in kg/m³.
    pub density: f32,
    /// Random seed for deterministic generation.
    pub seed: u64,
    /// Current simulation time in seconds.
    pub simulation_time: f32,
    /// Detail level (0.0 to 1.0) for LOD filtering.
    pub detail_level: f32,
}

/// An excluded radius (Kirkwood gap) in a belt.
#[derive(Debug, Clone)]
pub struct ExcludedRadius {
    /// Orbital distance of the gap center in metres.
    pub orbital_distance: f32,
    /// Radius of the gap in metres.
    pub gap_radius: f32,
}

/// A size distribution entry.
#[derive(Debug, Clone)]
pub struct SizeDistributionEntry {
    /// Radius of asteroids in this size class in metres.
    pub radius: f32,
    /// Relative weight/probability of this size class.
    pub weight: f32,
}

/// A stable body ID for delta store tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StableBodyId {
    /// Whether the body belongs to a belt or field.
    pub kind: BodyKind,
    /// Stable hash-based identifier.
    pub id: u64,
}

/// Whether the body belongs to a belt or field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyKind {
    /// Body belongs to an asteroid belt.
    Belt,
    /// Body belongs to an asteroid field.
    Field,
}

/// Delta record for a modified asteroid.
#[derive(Debug, Clone)]
pub enum AsteroidDelta {
    /// Asteroid was destroyed.
    Destroyed {
        /// Stable ID of the destroyed asteroid.
        stable_id: StableBodyId,
    },
    /// Asteroid was displaced (knocked off orbit).
    Displaced {
        /// Stable ID of the displaced asteroid.
        stable_id: StableBodyId,
        /// New position in world space.
        position: Vec3,
        /// New velocity in world space.
        velocity: Vec3,
        /// Simulation time when displacement occurred.
        timestamp: f32,
    },
    /// New asteroid added (wreck, player-built structure).
    Added {
        /// Stable ID of the new asteroid.
        stable_id: StableBodyId,
        /// Initial position in world space.
        position: Vec3,
        /// Initial velocity in world space.
        velocity: Vec3,
        /// Radius in metres.
        radius: f32,
        /// Mesh template name to use.
        mesh_name: String,
        /// Simulation time when added.
        timestamp: f32,
    },
}

/// Delta store for persisting player changes to procedural asteroids.
// allow-default: Bevy Resource trait bound requires Default for init_resource
#[derive(Debug, Default, Resource)]
pub struct AsteroidDeltaStore {
    deltas: Vec<AsteroidDelta>,
}

impl AsteroidDeltaStore {
    /// Records that an asteroid was destroyed.
    pub fn record_destroyed(&mut self, stable_id: StableBodyId) {
        self.deltas.push(AsteroidDelta::Destroyed { stable_id });
    }

    /// Records that an asteroid was displaced.
    pub fn record_displaced(
        &mut self,
        stable_id: StableBodyId,
        position: Vec3,
        velocity: Vec3,
        timestamp: f32,
    ) {
        self.deltas.push(AsteroidDelta::Displaced {
            stable_id,
            position,
            velocity,
            timestamp,
        });
    }

    /// Records a new asteroid added by the player.
    pub fn record_added(
        &mut self,
        stable_id: StableBodyId,
        position: Vec3,
        velocity: Vec3,
        radius: f32,
        mesh_name: String,
        timestamp: f32,
    ) {
        self.deltas.push(AsteroidDelta::Added {
            stable_id,
            position,
            velocity,
            radius,
            mesh_name,
            timestamp,
        });
    }

    /// Gets all deltas for a specific belt/field.
    pub fn get_deltas_for(&self, kind: BodyKind, parent_id: u64) -> Vec<&AsteroidDelta> {
        self.deltas
            .iter()
            .filter(|d| match d {
                AsteroidDelta::Destroyed { stable_id }
                | AsteroidDelta::Displaced { stable_id, .. }
                | AsteroidDelta::Added { stable_id, .. } => {
                    stable_id.kind == kind && stable_id.id == parent_id
                }
            })
            .collect()
    }

    /// Gets all deltas.
    pub fn all_deltas(&self) -> &[AsteroidDelta] {
        &self.deltas
    }
}

/// Computes a stable hash for deterministic generation.
fn stable_hash(seed: u64, belt_id: &str, sector_index: i64, body_index: u64, stream: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    belt_id.hash(&mut hasher);
    sector_index.hash(&mut hasher);
    body_index.hash(&mut hasher);
    stream.hash(&mut hasher);
    hasher.finish()
}

/// Computes a stable hash for field generation.
fn stable_hash_field(seed: u64, field_id: &str, body_index: u64, stream: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    field_id.hash(&mut hasher);
    body_index.hash(&mut hasher);
    stream.hash(&mut hasher);
    hasher.finish()
}

/// Picks a mesh from the template's mesh list using a hash.
fn pick_mesh(meshes: &[String], hash: u64) -> String {
    if meshes.is_empty() {
        return "asteroids/shared/mesh_1".to_string();
    }
    let index = usize::try_from(hash).unwrap_or(0) % meshes.len();
    meshes
        .get(index)
        .cloned()
        .unwrap_or_else(|| "asteroids/shared/mesh_1".to_string())
}

/// Picks a radius from the size distribution using a hash.
fn pick_radius(distribution: &[SizeDistributionEntry], hash: u64) -> f32 {
    if distribution.is_empty() {
        return 1000.0; // Default 1km
    }

    let total_weight: f32 = distribution.iter().map(|e| e.weight).sum();
    // Use f64 for intermediate calculation to avoid precision loss
    #[allow(clippy::cast_precision_loss)]
    let target = (hash as f64 / u64::MAX as f64) * f64::from(total_weight);
    #[allow(clippy::cast_possible_truncation)]
    let mut target = target as f32;

    for entry in distribution {
        target -= entry.weight;
        if target <= 0.0 {
            return entry.radius;
        }
    }

    // Fallback to last entry (distribution is non-empty, so this is safe)
    distribution.last().map_or(1000.0, |e| e.radius)
}

/// Generates a random rotation from a hash.
fn generate_rotation(hash: u64) -> Quat {
    // Use hash to generate a random quaternion
    #[allow(clippy::cast_precision_loss)]
    let x = ((hash & 0xFFFF) as f32 / 0xFFFF as f32).mul_add(2.0, -1.0);
    #[allow(clippy::cast_precision_loss)]
    let y = (((hash >> 16) & 0xFFFF) as f32 / 0xFFFF as f32).mul_add(2.0, -1.0);
    #[allow(clippy::cast_precision_loss)]
    let z = (((hash >> 32) & 0xFFFF) as f32 / 0xFFFF as f32).mul_add(2.0, -1.0);
    #[allow(clippy::cast_precision_loss)]
    let w = (((hash >> 48) & 0xFFFF) as f32 / 0xFFFF as f32).mul_add(2.0, -1.0);

    // Normalize to get a valid quaternion
    let len = (x * x + y * y + z * z + w * w).sqrt();
    if len > 0.0 {
        Quat::from_xyzw(x / len, y / len, z / len, w / len)
    } else {
        Quat::IDENTITY
    }
}

/// Checks if a body should be kept based on detail level.
fn detail_keep_test(
    seed: u64,
    belt_id: &str,
    sector_index: i64,
    body_index: u64,
    detail_level: f32,
) -> bool {
    let hash = stable_hash(seed, belt_id, sector_index, body_index, "detail");
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let threshold = (detail_level * u64::MAX as f32) as u64;
    hash < threshold
}

/// Checks if a field body should be kept based on detail level.
fn detail_keep_test_field(seed: u64, field_id: &str, body_index: u64, detail_level: f32) -> bool {
    let hash = stable_hash_field(seed, field_id, body_index, "detail");
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let threshold = (detail_level * u64::MAX as f32) as u64;
    hash < threshold
}

/// Generates asteroids for a belt sector.
#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn generate_belt_sector(params: &BeltSectorParams) -> Vec<GeneratedAsteroid> {
    let mut asteroids = Vec::new();

    // Calculate number of asteroids in this sector based on mean spacing
    // Sector is a torus segment: volume ≈ 2π * R * (radial_extent * 2) * (thickness * 2)
    let sector_volume = 2.0
        * std::f32::consts::PI
        * params.orbital_distance
        * (params.radial_extent * 2.0)
        * (params.thickness * 2.0)
        * (params.sector_size / (2.0 * std::f32::consts::PI * params.orbital_distance));

    let spacing_cubed = params.mean_spacing * params.mean_spacing * params.mean_spacing;
    let estimated_count = (sector_volume / spacing_cubed) as u64;

    // Cap at a reasonable maximum for performance
    let max_asteroids = 2000;
    let count = estimated_count.min(max_asteroids);

    for body_index in 0..count {
        // Generate stable ID
        let stable_id = stable_hash(
            params.seed,
            &params.belt_id,
            params.sector_index,
            body_index,
            "id",
        );

        // Detail level keep test
        if !detail_keep_test(
            params.seed,
            &params.belt_id,
            params.sector_index,
            body_index,
            params.detail_level,
        ) {
            continue;
        }

        // Pick mesh (independent stream)
        let mesh_hash = stable_hash(
            params.seed,
            &params.belt_id,
            params.sector_index,
            body_index,
            "mesh",
        );
        let mesh_name = pick_mesh(&params.template.meshes, mesh_hash);

        // Pick radius (independent stream)
        let radius_hash = stable_hash(
            params.seed,
            &params.belt_id,
            params.sector_index,
            body_index,
            "radius",
        );
        let radius = pick_radius(&params.size_distribution, radius_hash);

        // Generate rotation (independent stream)
        let rotation_hash = stable_hash(
            params.seed,
            &params.belt_id,
            params.sector_index,
            body_index,
            "rotation",
        );
        let rotation = generate_rotation(rotation_hash);

        // Generate orbital parameters for this asteroid
        // Semi-major axis varies within the belt's radial extent
        let a_hash = stable_hash(
            params.seed,
            &params.belt_id,
            params.sector_index,
            body_index,
            "semi_major_axis",
        );
        let a_offset = ((a_hash as f32 / u64::MAX as f32) - 0.5) * 2.0 * params.radial_extent;
        let orbital_distance = params.orbital_distance + a_offset;

        // Skip if in excluded radius (Kirkwood gap)
        let mut in_gap = false;
        for gap in &params.excluded_radii {
            let dist_from_gap = (orbital_distance - gap.orbital_distance).abs();
            if dist_from_gap < gap.gap_radius {
                in_gap = true;
                break;
            }
        }
        if in_gap {
            continue;
        }

        // Eccentricity and inclination vary around the belt's mean
        let e_hash = stable_hash(
            params.seed,
            &params.belt_id,
            params.sector_index,
            body_index,
            "eccentricity",
        );
        let orbital_eccentricity =
            params.orbital_eccentricity * (0.5 + (e_hash as f32 / u64::MAX as f32));

        let i_hash = stable_hash(
            params.seed,
            &params.belt_id,
            params.sector_index,
            body_index,
            "inclination",
        );
        let orbital_inclination =
            params.orbital_inclination * (0.5 + (i_hash as f32 / u64::MAX as f32));

        // Initial orbital angle
        let angle_hash = stable_hash(
            params.seed,
            &params.belt_id,
            params.sector_index,
            body_index,
            "initial_angle",
        );
        let initial_orbital_angle =
            (angle_hash as f32 / u64::MAX as f32) * 2.0 * std::f32::consts::PI;

        // Compute position at current simulation time using Keplerian propagation
        // Mean anomaly = initial_angle + 2π * t / T
        let mean_anomaly = initial_orbital_angle
            + 2.0 * std::f32::consts::PI * params.simulation_time / params.orbital_period;

        // Solve Kepler's equation for eccentric anomaly (simplified for small e)
        let eccentric_anomaly = mean_anomaly + orbital_eccentricity * mean_anomaly.sin();

        // True anomaly
        let true_anomaly = 2.0
            * ((1.0 + orbital_eccentricity) / (1.0 - orbital_eccentricity))
                .sqrt()
                .atan()
            * ((eccentric_anomaly / 2.0).tan());

        // Position in orbital plane
        let r = orbital_distance * (1.0 - orbital_eccentricity * eccentric_anomaly.cos());
        let x_orbital = r * true_anomaly.cos();
        let y_orbital = r * true_anomaly.sin();
        let z_orbital = 0.0;

        // Rotate by inclination
        let cos_i = orbital_inclination.cos();
        let sin_i = orbital_inclination.sin();
        let x = x_orbital;
        let y = y_orbital * cos_i - z_orbital * sin_i;
        let z = y_orbital * sin_i + z_orbital * cos_i;

        let position = Vec3::new(x, y, z);

        // Mass from density and radius
        let mass = params.density * (4.0 / 3.0) * std::f32::consts::PI * radius * radius * radius;

        // Collision shape (sphere scaled to radius)
        let collision_shape = CollisionShapeData {
            shape_type: CollisionShapeType::Sphere { radius },
            offset: Vec3::ZERO,
        };

        // Bounding box
        let bounding_box = delta_v_types::BoundingBox {
            min: Vec3::new(-radius, -radius, -radius),
            max: Vec3::new(radius, radius, radius),
        };

        asteroids.push(GeneratedAsteroid {
            stable_id,
            position,
            rotation,
            radius,
            mass,
            mesh_name,
            collision_shape,
            bounding_box,
            orbital_params: Some(OrbitalParams {
                orbital_parent: params.orbital_parent.clone(),
                orbital_distance,
                orbital_period: params.orbital_period,
                orbital_eccentricity,
                orbital_inclination,
                initial_orbital_angle,
            }),
        });
    }

    asteroids
}

/// Generates asteroids for a field.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn generate_field(params: &FieldParams) -> Vec<GeneratedAsteroid> {
    let mut asteroids = Vec::new();

    // Field is a box of size extent^3
    let field_volume = params.extent * params.extent * params.extent;
    let spacing_cubed = params.mean_spacing * params.mean_spacing * params.mean_spacing;
    let estimated_count = (field_volume / spacing_cubed) as u64;

    // Cap at a reasonable maximum for performance
    let max_asteroids = 2000;
    let count = estimated_count.min(max_asteroids);

    for body_index in 0..count {
        // Generate stable ID
        let stable_id = stable_hash_field(params.seed, &params.field_id, body_index, "id");

        // Detail level keep test
        if !detail_keep_test_field(
            params.seed,
            &params.field_id,
            body_index,
            params.detail_level,
        ) {
            continue;
        }

        // Pick mesh (independent stream)
        let mesh_hash = stable_hash_field(params.seed, &params.field_id, body_index, "mesh");
        let mesh_name = pick_mesh(&params.template.meshes, mesh_hash);

        // Pick radius (independent stream)
        let radius_hash = stable_hash_field(params.seed, &params.field_id, body_index, "radius");
        let radius = pick_radius(&params.size_distribution, radius_hash);

        // Generate rotation (independent stream)
        let rotation_hash =
            stable_hash_field(params.seed, &params.field_id, body_index, "rotation");
        let rotation = generate_rotation(rotation_hash);

        // Generate position within the field box
        let pos_hash_x = stable_hash_field(params.seed, &params.field_id, body_index, "pos_x");
        let pos_hash_y = stable_hash_field(params.seed, &params.field_id, body_index, "pos_y");
        let pos_hash_z = stable_hash_field(params.seed, &params.field_id, body_index, "pos_z");

        let x = ((pos_hash_x as f32 / u64::MAX as f32) - 0.5) * params.extent;
        let y = ((pos_hash_y as f32 / u64::MAX as f32) - 0.5) * params.extent;
        let z = ((pos_hash_z as f32 / u64::MAX as f32) - 0.5) * params.extent;

        let position = Vec3::new(x, y, z);

        // Mass from density and radius
        let mass = params.density * (4.0 / 3.0) * std::f32::consts::PI * radius * radius * radius;

        // Collision shape (sphere scaled to radius)
        let collision_shape = CollisionShapeData {
            shape_type: CollisionShapeType::Sphere { radius },
            offset: Vec3::ZERO,
        };

        // Bounding box
        let bounding_box = delta_v_types::BoundingBox {
            min: Vec3::new(-radius, -radius, -radius),
            max: Vec3::new(radius, radius, radius),
        };

        // Orbital parameters if the field orbits a parent
        let orbital_params = if let (Some(parent), Some(distance), Some(period)) = (
            params.orbital_parent.clone(),
            params.orbital_distance,
            params.orbital_period,
        ) {
            // Field orbits as a rigid body - all asteroids share the field's orbital motion
            // plus a small random offset
            let angle_hash =
                stable_hash_field(params.seed, &params.field_id, body_index, "initial_angle");
            let initial_orbital_angle =
                (angle_hash as f32 / u64::MAX as f32) * 2.0 * std::f32::consts::PI;

            Some(OrbitalParams {
                orbital_parent: parent,
                orbital_distance: distance,
                orbital_period: period,
                orbital_eccentricity: 0.0,
                orbital_inclination: 0.0,
                initial_orbital_angle,
            })
        } else {
            None
        };

        asteroids.push(GeneratedAsteroid {
            stable_id,
            position,
            rotation,
            radius,
            mass,
            mesh_name,
            collision_shape,
            bounding_box,
            orbital_params,
        });
    }

    asteroids
}

/// Separates overlapping asteroids at sector load.
///
/// Per the plan (4.11): walk every pair in stable ID order, move both along
/// the line joining their centres by half the overlap each.
///
/// # Panics
///
/// Panics if `asteroids` is empty (though the loop handles this gracefully).
/// The internal `get`/`get_mut` calls are guarded by loop bounds `i < j < len`,
/// so they cannot panic in practice.
pub fn separate_overlaps(asteroids: &mut [GeneratedAsteroid]) {
    // Sort by stable ID for deterministic order
    asteroids.sort_by_key(|a| a.stable_id);

    let len = asteroids.len();
    for i in 0..len {
        for j in (i + 1)..len {
            // Safe because i < j < len - loop bounds guarantee valid indices
            #[allow(clippy::indexing_slicing)]
            let (pos_i, radius_i) = (asteroids[i].position, asteroids[i].radius);
            #[allow(clippy::indexing_slicing)]
            let (pos_j, radius_j) = (asteroids[j].position, asteroids[j].radius);

            let delta = pos_j - pos_i;
            let distance = delta.length();
            let min_distance = radius_i + radius_j;

            if distance < min_distance && distance > f32::EPSILON {
                // Move both by half the overlap
                let overlap = min_distance - distance;
                let correction = delta.normalize() * (overlap * 0.5);
                #[allow(clippy::indexing_slicing)]
                {
                    asteroids[i].position -= correction;
                }
                #[allow(clippy::indexing_slicing)]
                {
                    asteroids[j].position += correction;
                }
            } else if distance <= f32::EPSILON {
                // Centers coincide - use direction from hash stream
                #[allow(clippy::cast_possible_wrap)]
                let hash = stable_hash(0, "separation", i as i64, j as u64, "direction");
                let dir = generate_rotation(hash) * Vec3::Z;
                let correction = dir * (min_distance * 0.5);
                #[allow(clippy::indexing_slicing)]
                {
                    asteroids[i].position -= correction;
                }
                #[allow(clippy::indexing_slicing)]
                {
                    asteroids[j].position += correction;
                }
            }
        }
    }
}

/// Propagates a displaced asteroid from its recorded timestamp to current time.
pub fn propagate_displaced_asteroid(
    position: Vec3,
    velocity: Vec3,
    timestamp: f32,
    current_time: f32,
    _orbital_params: &OrbitalParams,
) -> (Vec3, Vec3) {
    let dt = current_time - timestamp;
    if dt <= 0.0 {
        return (position, velocity);
    }

    // For simplicity, use linear propagation for displaced asteroids
    // In a full implementation, this would use Keplerian propagation
    let new_position = position + velocity * dt;
    (new_position, velocity)
}

#[cfg(test)]
#[path = "belt_field_generation_tests.rs"]
mod tests;
