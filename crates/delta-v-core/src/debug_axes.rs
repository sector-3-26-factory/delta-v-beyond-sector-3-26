// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Debug axis indicators: RGB arrows from entity centers.
//!
//! Per ADR-0022 (performance instrumentation), debug axes help visualize
//! entity orientations and alignment during development.
//!
//! The spawning process is decoupled per ADR-0005 (plugin architecture):
//! - Domain plugins (ShipsPlugin, etc.) mark entities with `DebugAxesEligible`.
//! - CorePlugin's `mark_debug_axes` system converts eligible entities to `DebugAxes`.
//! - CorePlugin's `spawn_debug_axes` system renders the axes.
//!
//! Axes are rendered as children with:
//! - X axis: red line
//! - Y axis: green line
//! - Z axis: blue line
//!
//! Length is calculated as 2× the entity's longest expansion along any axis.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::debug_config::DebugConfig;

#[cfg(test)]
#[path = "debug_axes_tests.rs"]
mod debug_axes_tests;

#[cfg(test)]
#[path = "debug_axes_tests.rs"]
mod tests;

/// Marker component for entities eligible to have debug axes rendered.
///
/// Domain plugins (e.g., ShipsPlugin) spawn entities and mark them with this component.
/// The `mark_debug_axes` system (in CorePlugin) reads this marker and adds `DebugAxes`
/// if debug config enables visualization. Per ADR-0005, this decouples debug
/// visualization from domain spawn logic.
#[derive(Component, Debug, Clone)]
pub struct DebugAxesEligible {
    /// Entity type or name for debug filtering (e.g., "local_player_ship").
    pub entity_id: String,
    /// Axis length in metres.
    pub axis_length: f32,
}

impl DebugAxesEligible {
    /// Creates a new debug axes eligibility marker.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(entity_id: String, axis_length: f32) -> Self {
        Self {
            entity_id,
            axis_length,
        }
    }
}

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

/// Marks eligible entities with `DebugAxes` if debug config enables visualization.
///
/// Runs during `SpawningEntities` state after all domain spawn systems have run.
/// Reads `DebugAxesEligible` markers and adds `DebugAxes` if `show_axis_indicators` is true.
/// Per ADR-0005, this decouples debug visualization from domain plugins.
/// Per ADR-0013, if DebugConfig is missing it is a hard error (checked at startup).
#[allow(clippy::needless_pass_by_value)]
pub fn mark_debug_axes(
    mut commands: Commands<'_, '_>,
    debug_config: Res<'_, DebugConfig>,
    query: Query<'_, '_, (Entity, &DebugAxesEligible), Added<DebugAxesEligible>>,
) {
    if !debug_config.show_axis_indicators {
        log::debug!("mark_debug_axes: show_axis_indicators is false, skipping");
        return;
    }

    let eligible_count = query.iter().count();
    if eligible_count == 0 {
        log::debug!("mark_debug_axes: no eligible entities found");
        return;
    }

    log::info!(
        "mark_debug_axes: {} eligible entities found, show_axis_indicators={}, axis_indicator_entities={:?}",
        eligible_count, debug_config.show_axis_indicators, debug_config.axis_indicator_entities
    );

    for (entity, eligible) in query.iter() {
        if debug_config.should_show_axes_for(&eligible.entity_id) {
            log::debug!(
                "mark_debug_axes: adding DebugAxes to entity {:?} (id: {})",
                entity,
                eligible.entity_id
            );
            commands.entity(entity).insert(DebugAxes::new(
                eligible.entity_id.clone(),
                eligible.axis_length,
            ));
        } else {
            log::debug!(
                "mark_debug_axes: filtering out entity {:?} (id: {}) - not in axis_indicator_entities",
                entity, eligible.entity_id
            );
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
/// Per ADR-0006, axes follow the right-handed coordinate system: +X right, +Y up, -Z forward.
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_debug_axes(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
    debug_config: Res<'_, DebugConfig>,
    query: Query<'_, '_, (Entity, &DebugAxes), Added<DebugAxes>>,
) {
    let axes_count = query.iter().count();
    if axes_count == 0 {
        log::debug!("spawn_debug_axes: no entities with DebugAxes component");
        return;
    }

    log::info!(
        "spawn_debug_axes: spawning axes for {} entities",
        axes_count
    );

    for (entity, axes) in query.iter() {
        // Check if this entity should have axes shown based on config.
        if !debug_config.should_show_axes_for(&axes.entity_id) {
            log::debug!(
                "spawn_debug_axes: skipping entity {:?} (id: {}) - filtered by config",
                entity,
                axes.entity_id
            );
            continue;
        }

        log::debug!(
            "spawn_debug_axes: rendering axes for entity {:?} (id: {}, length: {})",
            entity,
            axes.entity_id,
            axes.axis_length
        );

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
