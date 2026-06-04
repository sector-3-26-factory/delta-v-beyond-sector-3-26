// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Debug axis indicators: RGB arrows from entity centers.
//!
//! Per ADR-0022 (performance instrumentation), debug axes help visualize
//! entity orientations and alignment during development.
//!
//! The spawning process is decoupled per ADR-0005 (plugin architecture):
//! - Domain plugins (`ShipsPlugin`, etc.) mark entities with `DebugAxesEligible`.
//! - `CorePlugin`'s `mark_debug_axes` system converts eligible entities to `DebugAxes`.
//! - `CorePlugin`'s `spawn_debug_axes` system renders the axes and labels.
//!
//! Axes are rendered as independent world-space entities (not children of the
//! tracked entity) so they translate with the entity but do not inherit its
//! rotation. A separate `update_debug_axes_positions` system syncs their
//! position each frame.
//!
//! - X axis: red line with "X" label
//! - Y axis: green line with "Y" label
//! - Z axis: blue line with "Z" label
//!
//! Length is calculated as 2× the entity's longest expansion along any axis.
//! When the axis length changes (e.g. after a glTF mesh finishes loading and
//! the bounding box is computed), the old axis root is despawned and a new one
//! is created with the updated length.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy_mod_billboard::prelude::*;

use crate::debug_config::DebugConfig;

#[cfg(test)]
#[path = "debug_axes_tests.rs"]
mod tests;

/// Marker component for entities eligible to have debug axes rendered.
///
/// Domain plugins (e.g., `ShipsPlugin`) spawn entities and mark them with this component.
/// The `mark_debug_axes` system (in `CorePlugin`) reads this marker and adds `DebugAxes`
/// if debug config enables visualization. Per ADR-0005, this decouples debug
/// visualization from domain spawn logic.
#[derive(Component, Debug, Clone)]
pub struct DebugAxesEligible {
    /// Entity type or name for debug filtering (e.g., `"player_controlled_ship"`).
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
/// The axes are spawned as independent world-space entities (not children of the
/// target) so they translate with the target but do not inherit its rotation.
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

/// Component on a debug axis root entity linking it to the target entity whose
/// position it should follow.
///
/// The axis root is spawned as an independent world-space entity (not a child of
/// the target). The `update_debug_axes_positions` system updates its
/// `Transform::translation` each frame to match the target's position, while
/// keeping rotation at identity so the axes remain world-aligned.
#[derive(Component, Debug, Clone, Copy)]
pub struct DebugAxisTarget {
    /// The entity whose position this axis root should track.
    pub target: Entity,
}

/// Component on a target entity that tracks its spawned debug axis root entity.
///
/// When the [`DebugAxes`] component changes (e.g. axis length updated after mesh
/// load), the old axis root is despawned and a new one is spawned. This component
/// stores a reference to the current axis root so it can be cleaned up.
#[derive(Component, Debug, Clone, Copy)]
pub struct DebugAxisRoot {
    /// The entity that renders the debug axes for this target.
    pub root: Entity,
}

/// Marks eligible entities with `DebugAxes` if debug config enables visualization.
///
/// Runs during `SpawningEntities` state after all domain spawn systems have run.
/// Reads `DebugAxesEligible` markers and adds `DebugAxes` if `show_axis_indicators` is true.
/// Per ADR-0005, this decouples debug visualization from domain plugins.
/// Per ADR-0013, if `DebugConfig` is missing it is a hard error (checked at startup).
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

/// Spawns debug axis line meshes and labels for entities with `DebugAxes` component.
///
/// Runs during `SpawningEntities` state and also during `InGame` (for re-spawns
/// when axis length changes). For each marked entity, spawns an independent
/// world-space entity (NOT a child) at the same position, with identity rotation
/// so axes are world-aligned:
/// - X axis: red line with "X" label
/// - Y axis: green line with "Y" label
/// - Z axis: blue line with "Z" label
///
/// Each axis line is rendered as a line mesh via Bevy's line rendering.
/// Text labels are positioned at the end of each axis line (at coordinates
/// (length, 0, 0), (0, length, 0), (0, 0, length) respectively) and rendered
/// as Text2d with Billboard so they always face the camera.
/// Per ADR-0006, axes follow the right-handed coordinate system: +X right, +Y up, -Z forward.
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_debug_axes(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
    debug_config: Res<'_, DebugConfig>,
    query: Query<'_, '_, (Entity, &DebugAxes, &Transform), Added<DebugAxes>>,
) {
    let axes_count = query.iter().count();
    if axes_count == 0 {
        log::debug!("spawn_debug_axes: no entities with DebugAxes component");
        return;
    }

    log::info!("spawn_debug_axes: spawning axes for {axes_count} entities");

    for (entity, axes, transform) in query.iter() {
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

        let root_entity = spawn_axis_root(
            &mut commands,
            &mut meshes,
            &mut materials,
            axes,
            transform,
            entity,
        );

        // Store a reference to the axis root on the target entity so it can
        // be despawned when the axis length changes.
        commands
            .entity(entity)
            .insert(DebugAxisRoot { root: root_entity });
    }
}

/// Updates debug axes when the [`DebugAxes`] component changes (e.g. axis length
/// updated after glTF mesh load).
///
/// Runs during `InGame`. When the axis length changes, despawns the old axis root
/// entity and spawns a new one with the correct length.
#[allow(clippy::needless_pass_by_value)]
pub fn update_debug_axes_on_change(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
    debug_config: Res<'_, DebugConfig>,
    query: Query<'_, '_, (Entity, &DebugAxes, &Transform, &DebugAxisRoot), Changed<DebugAxes>>,
) {
    for (entity, axes, transform, axis_root) in query.iter() {
        if !debug_config.should_show_axes_for(&axes.entity_id) {
            continue;
        }

        log::info!(
            "update_debug_axes_on_change: axis length changed for entity {:?} (id: {}), despawning old root {:?}",
            entity,
            axes.entity_id,
            axis_root.root
        );

        // Despawn the old axis root entity.
        commands.entity(axis_root.root).despawn_recursive();

        // Spawn a new axis root with the updated length.
        let new_root = spawn_axis_root(
            &mut commands,
            &mut meshes,
            &mut materials,
            axes,
            transform,
            entity,
        );

        // Update the root reference.
        commands
            .entity(entity)
            .insert(DebugAxisRoot { root: new_root });
    }
}

/// Spawns a debug axis root entity with line meshes and labels.
///
/// # Arguments
///
/// * `commands` - Bevy commands buffer.
/// * `meshes` - Mesh asset storage.
/// * `materials` - Material asset storage.
/// * `axes` - The `DebugAxes` component with entity ID and length.
/// * `transform` - The target entity's transform (for initial position).
/// * `target` - The entity whose position the axis root should track.
///
/// Returns the entity ID of the spawned root.
fn spawn_axis_root(
    commands: &mut Commands<'_, '_>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    axes: &DebugAxes,
    transform: &Transform,
    target: Entity,
) -> Entity {
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

    // Spawn axis root as an independent world-space entity at the target's
    // current position, with identity rotation (world-aligned).
    // Child entities (lines + labels) inherit this identity rotation, so
    // they always show world X/Y/Z regardless of the target's orientation.
    commands
        .spawn((
            Transform {
                translation: transform.translation,
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            DebugAxisTarget { target },
        ))
        .with_children(|parent| {
            // X axis (red) with "X" label
            parent.spawn(PbrBundle {
                mesh: meshes.add(x_axis_mesh),
                material: x_material,
                ..default()
            });
            spawn_axis_label(
                parent,
                "X",
                length * LABEL_POSITION_FRACTION,
                0.0,
                0.0,
                Color::srgb(1.0, 0.0, 0.0),
                length,
            );

            // Y axis (green) with "Y" label
            parent.spawn(PbrBundle {
                mesh: meshes.add(y_axis_mesh),
                material: y_material,
                ..default()
            });
            spawn_axis_label(
                parent,
                "Y",
                0.0,
                length * LABEL_POSITION_FRACTION,
                0.0,
                Color::srgb(0.0, 1.0, 0.0),
                length,
            );

            // Z axis (blue) with "Z" label
            parent.spawn(PbrBundle {
                mesh: meshes.add(z_axis_mesh),
                material: z_material,
                ..default()
            });
            spawn_axis_label(
                parent,
                "Z",
                0.0,
                0.0,
                length * LABEL_POSITION_FRACTION,
                Color::srgb(0.0, 0.0, 1.0),
                length,
            );
        })
        .id()
}

/// Updates the position of all debug axis root entities to match their target.
///
/// Runs every frame in `Update` when `AppState::InGame`. Keeps the axis root's
/// rotation at identity so axes remain world-aligned while translating with
/// the target entity.
#[allow(clippy::needless_pass_by_value)]
pub fn update_debug_axes_positions(
    mut axis_query: Query<'_, '_, (&mut Transform, &DebugAxisTarget)>,
    target_query: Query<'_, '_, &Transform, Without<DebugAxisTarget>>,
) {
    for (mut axis_transform, axis_target) in &mut axis_query {
        let Ok(target_transform) = target_query.get(axis_target.target) else {
            continue;
        };
        // Match position only; keep rotation at identity (world-aligned).
        axis_transform.translation = target_transform.translation;
    }
}

/// Base font size for axis labels at the reference axis length.
const LABEL_BASE_FONT_SIZE: f32 = 50.0;

/// Base transform scale for axis labels at the reference axis length.
const LABEL_BASE_SCALE: f32 = 0.01;

/// Reference axis length for label sizing.
///
/// comfortably readable labels. For other lengths, labels are scaled
/// with the square root of the ratio to avoid enormous labels on
/// large ships while keeping them visible.
/// comfortably readable labels. For other lengths, labels are scaled
/// with the square root of the ratio to avoid enormous labels on
/// large ships while keeping them visible.
const LABEL_REFERENCE_LENGTH: f32 = 2.0;

/// Fraction of axis length where labels are placed.
///
/// Labels are positioned at this fraction of the axis length along
/// their respective axis, measured from the origin. A value of 1.1
/// places labels just past the tip of the axis line.
const LABEL_POSITION_FRACTION: f32 = 1.1;

/// Helper function to spawn an axis label with size adapted to the ship scale.
///
/// The label is rendered as `BillboardTextBundle` so it always faces the camera.
/// Font size and transform scale grow with the square root of the axis length,
/// providing readable labels on both small and large ships without overwhelming
/// the screen.
///
/// # Arguments
///
/// * `parent` - The child builder to spawn the label into.
/// * `label` - The text to display (e.g. "X").
/// * `x` - X position in local space.
/// * `y` - Y position in local space.
/// * `z` - Z position in local space.
/// * `color` - Text color.
/// * `axis_length` - The axis length used to scale the label size.
fn spawn_axis_label(
    parent: &mut ChildBuilder<'_>,
    label: &str,
    x: f32,
    y: f32,
    z: f32,
    color: Color,
    axis_length: f32,
) {
    let scale_factor = (axis_length / LABEL_REFERENCE_LENGTH).sqrt();
    let font_size = LABEL_BASE_FONT_SIZE * scale_factor;
    let transform_scale = LABEL_BASE_SCALE * scale_factor;
    log::info!(
        "spawn_axis_label: spawning label '{label}' at ({x}, {y}, {z}) font_size={font_size:.1} scale={transform_scale:.4}"
    );
    parent.spawn(BillboardTextBundle {
        transform: Transform::from_xyz(x, y, z).with_scale(Vec3::splat(transform_scale)),
        text: Text::from_section(
            label,
            TextStyle {
                font_size,
                color,
                ..default()
            },
        ),
        ..default()
    });
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
