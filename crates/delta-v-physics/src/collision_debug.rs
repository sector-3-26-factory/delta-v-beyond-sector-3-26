// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Collision shape debug visualization (ADR-0044 exempt).

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::primitives::Aabb;

use crate::collision::CollisionShapeType;
use crate::CollisionDetected;

/// Marker for entities whose collision shape should be visualized.
#[derive(Component, Debug)]
pub struct CollisionShapeDebug {
    /// Half-extents of the collision shape box.
    pub half_extents: Vec3,
}

/// Marker for the wireframe mesh child entity.
#[derive(Component, Debug)]
pub struct CollisionShapeDebugMesh;

/// Spawns wireframe debug meshes for entities with collision shapes.
#[allow(clippy::needless_pass_by_value, clippy::type_complexity, deprecated)]
pub fn spawn_collision_shape_debug(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
    debug_config: Res<'_, delta_v_core::debug::debug_config::DebugConfig>,
    query: Query<
        '_,
        '_,
        (Entity, &'static crate::CollisionShape),
        (Without<CollisionShapeDebug>, With<crate::CollisionShape>),
    >,
) {
    if !debug_config.show_collision_shapes {
        return;
    }

    for (entity, shape) in query.iter() {
        let (mesh, offset) = match &shape.shape_type {
            CollisionShapeType::Box { half_extents } => {
                (create_wireframe_box_mesh(*half_extents), shape.offset)
            }
            CollisionShapeType::Sphere { radius } => {
                (create_wireframe_sphere_mesh(*radius), shape.offset)
            }
            CollisionShapeType::ConvexHull => (create_wireframe_box_mesh(Vec3::ONE), shape.offset),
        };

        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 0.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        let mesh_handle: bevy::asset::Handle<Mesh> = meshes.add(mesh);
        let mat_handle: bevy::asset::Handle<StandardMaterial> = material;
        let debug_mesh = commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(mat_handle),
                Transform::from_translation(offset),
                Visibility::default(),
                CollisionShapeDebugMesh,
            ))
            .id();

        let debug_half_extents = match &shape.shape_type {
            CollisionShapeType::Box { half_extents } => *half_extents,
            CollisionShapeType::Sphere { radius } => Vec3::splat(*radius),
            CollisionShapeType::ConvexHull => Vec3::ONE,
        };
        commands
            .entity(entity)
            .insert(CollisionShapeDebug {
                half_extents: debug_half_extents,
            })
            .add_child(debug_mesh);
    }
}

/// Updates debug mesh colors and logs AABB bounds for troubleshooting.
#[allow(clippy::needless_pass_by_value)]
pub fn update_collision_shape_debug_color(
    mut events: EventReader<'_, '_, CollisionDetected>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
    debug_config: Res<'_, delta_v_core::debug::debug_config::DebugConfig>,
    mesh_query: Query<
        '_,
        '_,
        (&MeshMaterial3d<StandardMaterial>, &Parent),
        With<CollisionShapeDebugMesh>,
    >,
    aabb_query: Query<'_, '_, (Entity, &Aabb, &GlobalTransform)>,
    collision_debug_query: Query<'_, '_, (Entity, &CollisionShapeDebug)>,
) {
    if !debug_config.show_collision_shapes {
        return;
    }

    let mut colliding = std::collections::HashSet::new();
    for event in events.read() {
        colliding.insert(event.target);
        colliding.insert(event.other);
    }

    for (mat_handle, parent) in mesh_query.iter() {
        let is_colliding = colliding.contains(&parent.get());
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color = if is_colliding {
                Color::srgb(1.0, 0.0, 0.0)
            } else {
                Color::srgb(0.0, 1.0, 0.0)
            };
        }
    }

    // Log AABB bounds for entities with collision shapes
    for (entity, _debug) in collision_debug_query.iter() {
        if let Ok((_, aabb, gt)) = aabb_query.get(entity) {
            let center: Vec3 = aabb.center.into();
            let half: Vec3 = aabb.half_extents.into();
            let world_center = gt.transform_point(center);
            log::debug!(
                "Entity {:?} AABB: center=({:.2},{:.2},{:.2}) half=({:.2},{:.2},{:.2}) world_center=({:.2},{:.2},{:.2})",
                entity, center.x, center.y, center.z, half.x, half.y, half.z, world_center.x, world_center.y, world_center.z
            );
        }
    }
}

/// Edge connections for wireframe box mesh.
const BOX_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

/// Creates a wireframe box mesh.
#[allow(clippy::indexing_slicing, clippy::cast_possible_truncation)]
fn create_wireframe_box_mesh(he: Vec3) -> Mesh {
    let c = [
        Vec3::new(-he.x, -he.y, -he.z),
        Vec3::new(he.x, -he.y, -he.z),
        Vec3::new(he.x, he.y, -he.z),
        Vec3::new(-he.x, he.y, -he.z),
        Vec3::new(-he.x, -he.y, he.z),
        Vec3::new(he.x, -he.y, he.z),
        Vec3::new(he.x, he.y, he.z),
        Vec3::new(-he.x, he.y, he.z),
    ];
    let mut v = Vec::with_capacity(24);
    let mut idx = Vec::with_capacity(24);
    for (i, &(a, b)) in BOX_EDGES.iter().enumerate() {
        v.push(c[a]);
        v.push(c[b]);
        idx.push((i * 2) as u32);
        idx.push((i * 2 + 1) as u32);
    }
    let mut m = Mesh::new(PrimitiveTopology::LineList, default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, v);
    m.insert_indices(Indices::U32(idx));
    m
}

/// Creates a wireframe sphere mesh.
#[allow(
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::unnecessary_cast
)]
fn create_wireframe_sphere_mesh(radius: f32) -> Mesh {
    let mut v = Vec::new();
    let mut idx = Vec::new();
    let num_segments = 32u32;

    // Latitude circles
    for i in 0..=num_segments {
        let i_f32 = i as f32;
        let num_segments_f32 = num_segments as f32;
        let lat =
            std::f32::consts::PI.mul_add(i_f32 / num_segments_f32, -std::f32::consts::PI / 2.0);
        let circle_radius = lat.cos();
        let z = radius * lat.sin();

        for j in 0..num_segments {
            let j_f32 = j as f32;
            let lon = 2.0 * std::f32::consts::PI * (j_f32 / num_segments_f32);
            let x = radius * circle_radius * lon.cos();
            let y = radius * circle_radius * lon.sin();
            v.push(Vec3::new(x, y, z));
        }
    }

    // Create line list from latitude circles
    for i in 0..num_segments {
        for j in 0..num_segments {
            let curr = i * num_segments + j;
            let next = i * num_segments + (j + 1) % num_segments;
            idx.push(curr);
            idx.push(next);
        }
    }

    // Longitude circles
    for i in 0..num_segments {
        for j in 0..num_segments {
            let curr = i * num_segments + j;
            let next = ((i + 1) % (num_segments + 1)) * num_segments + j;
            idx.push(curr);
            idx.push(next);
        }
    }

    let mut m = Mesh::new(PrimitiveTopology::LineList, default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, v);
    m.insert_indices(Indices::U32(idx));
    m
}
