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
//! Per ADR-0055, celestial bodies follow Keplerian orbits and act as SOI gravity sources.
//!
//! Spawn systems are in `spawn.rs` per ADR-0047.

use crate::CollisionShape;
use bevy::animation::graph::{AnimationGraph, AnimationGraphHandle};
use bevy::animation::{AnimationClip, AnimationPlayer};
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::window::Window;
use bevy_world_serialization::WorldAssetRoot;
use delta_v_core::WorldEntityId;

/// Component for sun entities.
///
/// Suns are stars that provide light and gravity in the game world.
/// Per ADR-0055, suns are SOI gravity sources (Keplerian orbit root).
///
/// The `rotation_period` is in hours and controls the slow rotation animation.
#[derive(Component, Clone, Debug)]
pub struct Sun {
    /// Rotation period in hours. None means no rotation.
    pub rotation_period: Option<f32>,
}

/// Component for planet entities.
///
/// Planets are SOI gravity sources per ADR-0055 (Keplerian orbits).
/// Orbital parameters are now in the separate `OrbitalBody` component.
#[derive(Component, Clone, Debug)]
pub struct Planet {
    /// Rotation period in seconds. None means no rotation.
    pub rotation_period: Option<f32>,
    /// Axial tilt (obliquity) in radians. Angle between rotation axis and orbital axis.
    pub axial_tilt: f32,
    /// If true, mesh animations (e.g., moving clouds) will be played.
    pub animations_enabled: bool,
}

/// Component for moon entities.
///
/// Moons are SOI gravity sources per ADR-0055 (Keplerian orbits).
/// Orbital parameters are now in the separate `OrbitalBody` component.
#[derive(Component, Clone, Debug)]
pub struct Moon {
    /// Rotation period in seconds. None means no rotation.
    pub rotation_period: Option<f32>,
    /// Axial tilt (obliquity) in radians. Angle between rotation axis and orbital axis.
    pub axial_tilt: f32,
    /// If true, mesh animations (e.g., moving clouds) will be played.
    pub animations_enabled: bool,
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

/// Component to control lazy loading of celestial meshes based on visibility.
///
/// Large bodies (sun, planets) load if visible (≥1 pixel on screen).
/// Small bodies (moons, asteroids) load only when near the player.
/// Suns are always considered visible; the system calculates their screen radius
/// and loads the mesh only when ≥1px, otherwise a 1-pixel fallback should be rendered.
// allow-default: LazyLoadMesh is a runtime Component constructed programmatically during
// entity spawning (see spawn.rs). It is never deserialized from JSON — all fields are
// explicitly initialized from spawn event data. Default provides zero-initialization for
// `loaded` and `current_screen_radius_px` only; `mesh_path` and `min_screen_radius_px`
// are always set explicitly.
#[derive(Component, Clone, Debug, Default)]
pub struct LazyLoadMesh {
    /// Minimum screen-space radius in pixels to trigger mesh loading.
    /// Set to 1.0 for large bodies (always load if visible).
    /// Set to a larger value (e.g., 1000.0) for small bodies to only load when near.
    pub min_screen_radius_px: f32,

    /// Whether the mesh has been loaded.
    pub loaded: bool,

    /// The mesh path to load when visible.
    pub mesh_path: String,

    /// Current screen-space radius in pixels (updated each frame by lazy_load_celestial_meshes).
    /// For suns: always updated, even when <1px (for 1-pixel fallback rendering).
    /// For other bodies: only updated when on-screen.
    pub current_screen_radius_px: f32,
}

/// Component for the 1-pixel sun fallback VFX.
///
/// When a sun's mesh is not loaded (screen radius < 1px) but the sun is visible
/// (current_screen_radius_px > 0), this component marks an entity that renders
/// a single bright pixel at the sun's position.
///
/// Per ADR-0053, this is a VFX exemption - procedural rendering of a sun flare
/// when the mesh is not loaded.
#[derive(Component, Clone, Debug)]
pub struct SunFallbackVfx {
    /// The sun entity this fallback belongs to.
    pub sun_entity: Entity,
}

/// Component for entities that orbit a parent body.
///
/// This unified component replaces the orbital fields that were previously
/// duplicated in `Planet` and `Moon` components. Any entity with this component
/// will have its position updated by the `orbital_motion_system`.
#[derive(Component, Clone, Debug)]
pub struct OrbitalBody {
    /// The parent entity this body orbits.
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
    mut animation_graphs: ResMut<'_, Assets<AnimationGraph>>,
    query: Query<'_, '_, (Entity, &PendingCelestialMesh, &Transform, Option<&Planet>)>,
) {
    for (entity, pending, parent_transform, planet) in &query {
        if let Some(gltf) = gltf_assets.get(&pending.gltf_handle) {
            if gltf.scenes.is_empty() {
                continue;
            }
            for scene_handle in &gltf.scenes {
                let child = commands.spawn((WorldAssetRoot(scene_handle.clone()),)).id();
                commands.entity(entity).add_child(child);

                // Debug: log the parent and child transforms
                tracing::debug!(
                    "Attached celestial mesh to entity {entity:?}, child: {child:?}, parent_scale={:?}, parent_translation={:?}",
                    parent_transform.scale,
                    parent_transform.translation
                );
            }

            // If this is a planet with animations enabled and the glTF has animations,
            // add an AnimationPlayer and play the animations.
            if let Some(planet) = planet
                && planet.animations_enabled
                && !gltf.animations.is_empty()
            {
                // Create an AnimationGraph from the animation clips
                let clips: Vec<Handle<AnimationClip>> = gltf.animations.clone();
                let (animation_graph, node_indices) = AnimationGraph::from_clips(clips);

                // Add the AnimationGraph as an asset and get a handle to it
                let animation_graph_handle = animation_graphs.add(animation_graph);

                // Add AnimationPlayer and AnimationGraphHandle to the planet entity
                let mut animation_player = AnimationPlayer::default();
                for node_index in node_indices {
                    animation_player.play(node_index).repeat();
                }

                commands.entity(entity).insert((
                    animation_player,
                    AnimationGraphHandle(animation_graph_handle),
                ));
                tracing::info!(
                    "Started {} animations for planet entity {entity:?}",
                    gltf.animations.len()
                );
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
                && let Some(ref mut material) = materials.get_mut(&mat_handle.0)
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

/// Lazily loads celestial body meshes based on screen-space pixel size.
///
/// This system prevents OOM on startup by only loading meshes when they are
/// visible on screen (for large bodies) or near the player (for small bodies).
/// Uses true screen-space projection to calculate the apparent size in pixels.
///
/// Per ADR-0055, this is part of the performance optimization for the full
/// solar system with 289 moons.
///
/// Runs in `Update` during `AppState::InGame`.
#[allow(clippy::needless_pass_by_value)]
pub fn lazy_load_celestial_meshes(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    gltf_assets: Res<'_, Assets<Gltf>>,
    mut animation_graphs: ResMut<'_, Assets<AnimationGraph>>,
    cameras: Query<'_, '_, (&Camera, &GlobalTransform)>,
    windows: Query<'_, '_, &Window>,
    mut query: Query<
        '_,
        '_,
        (
            Entity,
            &mut LazyLoadMesh,
            &GlobalTransform,
            &CollisionShape,
            Option<&Planet>,
            Option<&Sun>,
        ),
        Without<PendingCelestialMesh>,
    >,
) {
    // Get the main camera (first one found)
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };

    // Get the primary window for viewport dimensions
    let Ok(window) = windows.single() else {
        return;
    };

    for (entity, mut lazy_load, global_transform, collision_shape, planet, sun) in &mut query {
        if lazy_load.loaded {
            continue;
        }

        let world_pos = global_transform.translation();

        // Project world position to screen space (viewport coordinates)
        let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_pos) else {
            // Behind camera or projection failed
            // For suns, still calculate a minimal screen radius for 1-pixel fallback
            if sun.is_some() {
                lazy_load.current_screen_radius_px = 0.0;
            }
            continue;
        };

        // Calculate screen-space radius in pixels using the collision shape
        // The collision shape is already scaled by the world.json scale factor during spawning
        // and represents the actual mesh bounds from the template JSON files.
        let estimated_radius_m = match collision_shape.shape_type {
            delta_v_types::CollisionShapeType::Sphere { radius } => radius,
            delta_v_types::CollisionShapeType::Box { half_extents } => {
                // Use the maximum half-extent as the effective radius
                half_extents.x.max(half_extents.y).max(half_extents.z)
            }
            delta_v_types::CollisionShapeType::ConvexHull => {
                // Fallback for convex hull - use a reasonable default
                1000.0
            }
        };

        // Project a point at radius distance along the view direction
        // to get the screen-space radius
        let view_dir = (world_pos - camera_transform.translation()).normalize();
        let radius_world_pos = world_pos + view_dir * estimated_radius_m;

        let Ok(radius_viewport_pos) = camera.world_to_viewport(camera_transform, radius_world_pos)
        else {
            // For suns, still track screen radius even if radius projection fails
            if sun.is_some() {
                lazy_load.current_screen_radius_px = 0.0;
            }
            continue;
        };

        let screen_radius_px = (viewport_pos - radius_viewport_pos).length();

        // Always update current_screen_radius_px for suns (for 1-pixel fallback rendering)
        // For other bodies, only update when on-screen
        let is_sun = sun.is_some();
        let on_screen = viewport_pos.x >= -100.0
            && viewport_pos.x <= window.width() + 100.0
            && viewport_pos.y >= -100.0
            && viewport_pos.y <= window.height() + 100.0;

        if is_sun || on_screen {
            lazy_load.current_screen_radius_px = screen_radius_px;
        }

        // Check if screen-space radius meets the threshold
        if screen_radius_px >= lazy_load.min_screen_radius_px {
            // Load the mesh
            let gltf_handle = asset_server.load::<Gltf>(&lazy_load.mesh_path);

            // Check if already loaded
            if let Some(gltf) = gltf_assets.get(&gltf_handle) {
                if !gltf.scenes.is_empty() {
                    for scene_handle in &gltf.scenes {
                        let child = commands.spawn((WorldAssetRoot(scene_handle.clone()),)).id();
                        commands.entity(entity).add_child(child);
                    }

                    // If this is a planet with animations enabled and the glTF has animations,
                    // add an AnimationPlayer and play the animations.
                    if let Some(planet) = planet
                        && planet.animations_enabled
                        && !gltf.animations.is_empty()
                    {
                        let clips: Vec<Handle<AnimationClip>> = gltf.animations.clone();
                        let (animation_graph, node_indices) = AnimationGraph::from_clips(clips);
                        let animation_graph_handle = animation_graphs.add(animation_graph);
                        let mut animation_player = AnimationPlayer::default();
                        for node_index in node_indices {
                            animation_player.play(node_index).repeat();
                        }
                        commands.entity(entity).insert((
                            animation_player,
                            AnimationGraphHandle(animation_graph_handle),
                        ));
                        tracing::info!(
                            "Started {} animations for planet entity {entity:?}",
                            gltf.animations.len()
                        );
                    }

                    // Make sun emissive
                    if sun.is_some() {
                        // We'll handle this in make_sun_emissive system
                    }

                    lazy_load.loaded = true;
                    tracing::info!(
                        "Lazy loaded mesh for entity {entity:?} (screen_radius={screen_radius_px:.1}px, threshold={:.1}px)",
                        lazy_load.min_screen_radius_px
                    );
                }
            } else {
                // Not loaded yet, add PendingCelestialMesh to track it
                commands
                    .entity(entity)
                    .insert(PendingCelestialMesh { gltf_handle });
                lazy_load.loaded = true; // Mark as "loading initiated"
            }
        }
    }
}

/// Spawns a 1-pixel fallback VFX for suns that are visible but too small for mesh loading.
///
/// Runs in `Update` during `AppState::InGame`.
/// Per ADR-0053, this is a VFX exemption - procedural rendering of a sun flare
/// when the mesh is not loaded (screen radius < 1px but > 0).
#[allow(clippy::needless_pass_by_value)]
pub fn sun_fallback_vfx_system(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    suns: Query<'_, '_, (Entity, &Sun, &LazyLoadMesh, &GlobalTransform), Without<SunFallbackVfx>>,
) {
    for (entity, _sun, lazy_load, global_transform) in &suns {
        // Only spawn fallback if:
        // - Mesh is not loaded
        // - Sun is visible (current_screen_radius_px > 0)
        // - Sun is too small for mesh loading (current_screen_radius_px < 1.0)
        if !lazy_load.loaded
            && lazy_load.current_screen_radius_px > 0.0
            && lazy_load.current_screen_radius_px < 1.0
        {
            // Create a 1x1 white pixel image for the fallback
            let fallback_image = Image::new_fill(
                bevy::render::render_resource::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                bevy::render::render_resource::TextureDimension::D2,
                &[255, 255, 255, 255], // Bright white
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                bevy::asset::RenderAssetUsages::default(),
            );
            let image_handle = asset_server.add(fallback_image);

            // Spawn a child entity with a sprite at the sun's position
            // Use a billboard that always faces the camera
            let fallback_entity = commands
                .spawn((
                    Name::new("Sun Fallback VFX"),
                    Sprite {
                        image: image_handle,
                        color: Color::srgb(1.0, 0.95, 0.8), // Warm white like the sun
                        ..default()
                    },
                    Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(100.0)),
                    Visibility::Visible,
                    SunFallbackVfx { sun_entity: entity },
                    // Render on gameplay layer
                    RenderLayers::layer(0),
                ))
                .id();

            // Add as child of the sun so it follows the sun's position
            commands.entity(entity).add_child(fallback_entity);

            tracing::debug!("Spawned 1-pixel sun fallback for {entity:?}");
        }
    }
}

/// Updates sun fallback VFX - removes when mesh loads or sun becomes invisible.
///
/// Runs in `Update` during `AppState::InGame`.
#[allow(clippy::needless_pass_by_value)]
pub fn update_sun_fallback_vfx_system(
    mut commands: Commands<'_, '_>,
    suns: Query<'_, '_, (Entity, &Sun, &LazyLoadMesh)>,
    fallback_query: Query<'_, '_, (Entity, &SunFallbackVfx)>,
) {
    for (sun_entity, _sun, lazy_load) in &suns {
        // Check if fallback should be removed:
        // - Mesh is now loaded, OR
        // - Sun is no longer visible (current_screen_radius_px == 0)
        let should_remove = lazy_load.loaded || lazy_load.current_screen_radius_px == 0.0;

        if should_remove {
            for (fallback_entity, fallback) in &fallback_query {
                if fallback.sun_entity == sun_entity {
                    commands.entity(fallback_entity).despawn();
                    tracing::debug!("Despawned sun fallback for {sun_entity:?}");
                }
            }
        }
    }
}

/// Resolves orbital parent IDs to Entity references.
///
/// This system runs after all entities are spawned to resolve the `OrbitalParentId`
/// component to an actual `Entity` reference for both `Planet`/`Moon` and `OrbitalBody` components.
///
/// Runs once during `AppState::SpawningEntities` in `Update` schedule (not `FixedUpdate`).
#[allow(clippy::needless_pass_by_value, clippy::explicit_iter_loop)]
pub fn resolve_orbital_parents(
    mut commands: Commands<'_, '_>,
    mut planets: Query<'_, '_, (Entity, &OrbitalParentId, &mut Planet)>,
    mut moons: Query<'_, '_, (Entity, &OrbitalParentId, &mut Moon)>,
    mut orbital_bodies: Query<'_, '_, (Entity, &OrbitalParentId, &mut OrbitalBody)>,
    all_entities: Query<'_, '_, (Entity, &WorldEntityId)>,
) {
    for (entity, parent_id, _planet) in &mut planets {
        // Find the parent entity by WorldEntityId (matches the world definition ID)
        for (potential_parent, world_id) in &all_entities {
            if world_id.0 == parent_id.0 {
                // Planet component no longer has orbital_parent, but we still need to remove OrbitalParentId
                commands.entity(entity).remove::<OrbitalParentId>();
                tracing::debug!(
                    "Resolved orbital parent for {entity:?}: {} -> {potential_parent:?}",
                    parent_id.0
                );
                break;
            }
        }
    }

    for (entity, parent_id, _moon) in &mut moons {
        // Find the parent entity by WorldEntityId (matches the world definition ID)
        for (potential_parent, world_id) in &all_entities {
            if world_id.0 == parent_id.0 {
                // Moon component no longer has orbital_parent, but we still need to remove OrbitalParentId
                commands.entity(entity).remove::<OrbitalParentId>();
                tracing::debug!(
                    "Resolved orbital parent for {entity:?}: {} -> {potential_parent:?}",
                    parent_id.0
                );
                break;
            }
        }
    }

    for (entity, parent_id, mut orbital_body) in &mut orbital_bodies {
        // Find the parent entity by WorldEntityId (matches the world definition ID)
        for (potential_parent, world_id) in &all_entities {
            if world_id.0 == parent_id.0 {
                orbital_body.orbital_parent = potential_parent;
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
