// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Debug axis indicators: RGB arrows from entity centers.
//!
//! Per ADR-0022 (performance instrumentation), debug axes help visualize
//! entity orientations and alignment during development.
//!
//! The `spawn_debug_axes` system creates line meshes for entities marked
//! with the `DebugAxes` component. Axes are rendered as children with:
//! - X axis: red line
//! - Y axis: green line
//! - Z axis: blue line
//!
//! Length is calculated as 2× the entity's longest expansion along any axis.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::debug_config::DebugConfig;

/// Component marking an entity that should have debug axes rendered.
///
/// The axes are spawned as child entities with line meshes.
/// Length is calculated as 2× the entity's longest expansion along any axis.
#[derive(Component, Debug, Clone)]
pub struct DebugAxes {
    /// Entity ID for selective axis targeting (from world definition).
    pub entity_id: String,
    /// Length of each axis line in metres.
    pub axis_length: f32,
}

impl DebugAxes {
    /// Creates a new debug axes marker with the given entity ID and axis length.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(entity_id: String, axis_length: f32) -> Self {
        Self {
            entity_id,
            axis_length,
        }
    }
}

/// Spawns debug axis line meshes for entities with `DebugAxes` component.
///
/// Runs during `SpawningEntities` state. Creates three child entities per
/// marked entity:
/// - X axis: red line from origin to (`axis_length`, 0, 0)
/// - Y axis: green line from origin to (0, `axis_length`, 0)
/// - Z axis: blue line from origin to (0, 0, `axis_length`)
///
/// Each axis is rendered as a line mesh via Bevy's line rendering.
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_debug_axes(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
    debug_config: Res<'_, DebugConfig>,
    query: Query<'_, '_, (Entity, &DebugAxes), Added<DebugAxes>>,
) {
    for (entity, axes) in query.iter() {
        // Check if this entity should have axes shown based on config.
        if !debug_config.should_show_axes_for(&axes.entity_id) {
            continue;
        }

        let length = axes.axis_length;

        // Create line meshes for each axis
        let x_axis_mesh = create_line_mesh(Vec3::ZERO, Vec3::new(length, 0.0, 0.0));
        let y_axis_mesh = create_line_mesh(Vec3::ZERO, Vec3::new(0.0, length, 0.0));
        let z_axis_mesh = create_line_mesh(Vec3::ZERO, Vec3::new(0.0, 0.0, length));

        // Create materials for each axis (red, green, blue)
        let x_material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),
            emissive: Color::srgb(1.0, 0.0, 0.0).into(),
            ..default()
        });
        let y_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 0.0),
            emissive: Color::srgb(0.0, 1.0, 0.0).into(),
            ..default()
        });
        let z_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.0, 1.0),
            emissive: Color::srgb(0.0, 0.0, 1.0).into(),
            ..default()
        });

        // Spawn axis lines as child entities
        commands.entity(entity).with_children(|parent| {
            // X axis (red)
            parent.spawn(PbrBundle {
                mesh: meshes.add(x_axis_mesh),
                material: x_material,
                ..default()
            });

            // Y axis (green)
            parent.spawn(PbrBundle {
                mesh: meshes.add(y_axis_mesh),
                material: y_material,
                ..default()
            });

            // Z axis (blue)
            parent.spawn(PbrBundle {
                mesh: meshes.add(z_axis_mesh),
                material: z_material,
                ..default()
            });
        });
    }
}

/// Creates a line mesh from start to end point.
///
/// Returns a mesh with two vertices and one line segment.
#[allow(clippy::default_trait_access)]
fn create_line_mesh(start: Vec3, end: Vec3) -> Mesh {
    let vertices = vec![start, end];
    let indices = Indices::U32(vec![0, 1]);

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, default());

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(indices);

    mesh
}
