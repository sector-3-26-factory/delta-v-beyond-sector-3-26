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

//! Celestial body components for suns, planets, and asteroids.
//!
//! These components are used by the orbital motion and sun rotation systems.
//! Per ADR-0009, celestial bodies are always gravity sources.
//!
//! Spawn systems are in `spawn.rs` per ADR-0047.

use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_core::WorldEntityId;

/// Component for sun entities.
///
/// Suns are stars that provide light and gravity in the game world.
/// Per ADR-0009, suns are always gravity sources.
///
/// The `rotation_period` is in hours and controls the slow rotation animation.
#[derive(Component, Clone, Debug)]
pub struct Sun {
    /// Rotation period in hours. None means no rotation.
    pub rotation_period: Option<f32>,
}

/// Component for planet entities.
///
/// Planets orbit a parent body (sun or another planet) and are always
/// gravity sources per ADR-0009.
///
/// Orbital parameters are stored in SI units (metres, seconds, radians).
#[derive(Component, Clone, Debug)]
pub struct Planet {
    /// The parent entity this planet orbits.
    pub orbital_parent: Entity,
    /// Orbital distance in metres.
    pub orbital_distance: f32,
    /// Orbital period in seconds.
    pub orbital_period: f32,
    /// Orbital eccentricity (0 = circular, 0.1-0.9 = increasingly elliptical).
    pub orbital_eccentricity: f32,
    /// Orbital inclination in radians.
    pub orbital_inclination: f32,
    /// Initial orbital angle in radians.
    pub initial_orbital_angle: f32,
}

/// Marker component for entities that can be targeted or selected in navigation.
///
/// This includes ships, stations, planets, and other navigable objects.
/// Per ADR-0054, targeting is explicit player control only.
#[derive(Component, Clone, Copy, Debug)]
pub struct Navigable;

/// Pending mesh marker for celestial bodies.
///
/// Used to track when celestial body meshes are loaded and ready to attach.
/// Implements a local version of the `PendingMesh` pattern to avoid the
/// `delta-v-spawn` dependency cycle.
#[derive(Component, Clone, Debug)]
pub struct PendingCelestialMesh {
    /// Handle to the glTF asset being loaded.
    pub gltf_handle: Handle<Gltf>,
}

/// Component storing the parent entity ID string for resolution.
///
/// This is a temporary marker used during spawning to store the parent ID
/// before it can be resolved to an Entity reference.
/// Per ADR-0038, this is resolved in a second pass after all entities are spawned.
#[derive(Component, Clone, Debug)]
pub struct OrbitalParentId(pub String);

/// Attaches loaded glTF scenes to celestial body entities.
///
/// This system runs in `Update` during `AppState::InGame` to attach meshes
/// once the glTF assets are loaded.
#[allow(clippy::needless_pass_by_value)]
pub fn attach_celestial_meshes(
    mut commands: Commands<'_, '_>,
    gltf_assets: Res<'_, Assets<Gltf>>,
    query: Query<'_, '_, (Entity, &PendingCelestialMesh)>,
) {
    for (entity, pending) in &query {
        if let Some(gltf) = gltf_assets.get(&pending.gltf_handle) {
            if gltf.scenes.is_empty() {
                continue;
            }
            for scene_handle in &gltf.scenes {
                let child = commands.spawn(SceneRoot(scene_handle.clone())).id();
                commands.entity(entity).add_child(child);
            }
            commands.entity(entity).remove::<PendingCelestialMesh>();
        }
    }
}

/// Makes sun meshes emissive after they are loaded.
///
/// Per ADR-0053, this is a VFX exemption - the sun's mesh should glow
/// to appear as a light source. This system runs after `attach_celestial_meshes`
/// to modify the material of the sun's mesh to be emissive.
///
/// The glTF scene structure is: Sun -> `SceneRoot` -> Mesh (with material).
/// We need to recursively check all descendants to find meshes with materials.
#[allow(clippy::needless_pass_by_value)]
pub fn make_sun_emissive(
    suns: Query<'_, '_, (Entity, &Children, &Sun)>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
    mesh_query: Query<'_, '_, &MeshMaterial3d<StandardMaterial>>,
    children_query: Query<'_, '_, &Children>,
) {
    for (sun_entity, children, _sun) in &suns {
        // Recursively check all descendants for meshes with materials
        let mut stack: Vec<Entity> = Vec::new();
        for child in children.iter() {
            stack.push(child);
        }
        while let Some(entity) = stack.pop() {
            if let Ok(mat_handle) = mesh_query.get(entity)
                && let Some(material) = materials.get_mut(&mat_handle.0)
            {
                // Make the material emissive with a bright yellow-white color
                // matching the light color (warm white: 1.0, 0.95, 0.8)
                material.emissive = LinearRgba::new(1.0, 0.95, 0.8, 1.0);
                tracing::debug!("Made sun descendant {entity:?} emissive (sun: {sun_entity:?})");
            }
            // Add children to stack for depth-first traversal
            if let Ok(entity_children) = children_query.get(entity) {
                for child in entity_children.iter() {
                    stack.push(child);
                }
            }
        }
    }
}

/// Resolves orbital parent IDs to Entity references.
///
/// This system runs after all entities are spawned to resolve the `OrbitalParentId`
/// component to an actual `Entity` reference.
///
/// Per ADR-0017, this system runs in `FixedUpdate` at 60 Hz.
#[allow(clippy::needless_pass_by_value, clippy::explicit_iter_loop)]
pub fn resolve_orbital_parents(
    mut commands: Commands<'_, '_>,
    mut planets: Query<'_, '_, (Entity, &OrbitalParentId, &mut Planet)>,
    all_entities: Query<'_, '_, (Entity, &WorldEntityId)>,
) {
    for (entity, parent_id, mut planet) in &mut planets {
        // Find the parent entity by WorldEntityId (matches the world definition ID)
        for (potential_parent, world_id) in &all_entities {
            if world_id.0 == parent_id.0 {
                planet.orbital_parent = potential_parent;
                commands.entity(entity).remove::<OrbitalParentId>();
                tracing::debug!(
                    "Resolved orbital parent for {entity:?}: {} -> {potential_parent:?}",
                    parent_id.0
                );
                break;
            }
        }
    }
}

#[cfg(test)]
#[path = "celestial_tests.rs"]
mod tests;
