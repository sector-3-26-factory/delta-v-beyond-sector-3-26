// AGENTS: before modifying this file, read AGENTS.md at the repository root.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy_mod_billboard::prelude::*;

use super::axes::{DebugAxes, DebugAxesEligible, DebugAxisRootMarker};
use super::debug_config::DebugConfig;

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
/// when axis length changes). For each marked entity, adds axis root as a CHILD
/// of the target entity, with axis meshes and labels as children of the root.
/// The axis root's rotation is set to the inverse of the target's rotation each
/// frame by `update_debug_axes_rotation`, keeping axes world-aligned.
///
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

    log::info!("spawn_debug_axes: spawning axes for {axes_count} entities");

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

        add_debug_axes_to_entity(&mut commands, &mut meshes, &mut materials, axes, entity);
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
    query: Query<'_, '_, (Entity, &DebugAxes, &Children), Changed<DebugAxes>>,
    axis_root_query: Query<'_, '_, Entity, With<DebugAxisRootMarker>>,
) {
    for (entity, axes, children) in query.iter() {
        if !debug_config.should_show_axes_for(&axes.entity_id) {
            continue;
        }

        // Find the axis root among the entity's children.
        for &child in children {
            if axis_root_query.contains(child) {
                log::info!(
                    "update_debug_axes_on_change: axis length changed for entity {:?} (id: {}), despawning old root {:?}",
                    entity,
                    axes.entity_id,
                    child
                );

                // Despawn the old axis root entity.
                commands.entity(child).despawn_recursive();

                // Spawn a new axis root with the updated length.
                add_debug_axes_to_entity(&mut commands, &mut meshes, &mut materials, axes, entity);
                break;
            }
        }
    }
}

/// Updates the rotation of all debug axis roots to match their target's rotation.
///
/// Runs every frame in `Update` when `AppState::InGame`. For each axis root,
/// computes the inverse of the target's rotation and applies it to the axis root.
/// This causes the axes to remain world-aligned even as the target rotates.
#[allow(clippy::needless_pass_by_value)]
pub fn update_debug_axes_rotation(
    mut axis_query: Query<'_, '_, (&Parent, &mut Transform), With<DebugAxisRootMarker>>,
    target_query: Query<'_, '_, &Transform, Without<DebugAxisRootMarker>>,
) {
    for (parent, mut axis_transform) in &mut axis_query {
        let Ok(target_transform) = target_query.get(**parent) else {
            continue;
        };
        // Set local rotation to the inverse of the parent's rotation.
        // This cancels out the parent's rotation, making axes appear world-aligned.
        axis_transform.rotation = target_transform.rotation.inverse();
    }
}

/// Adds debug axes to an entity as children.
///
/// Spawns an axis root as a child of the target entity, with axis meshes
/// and labels as children of the root. The axis root is marked with
/// `DebugAxisRootMarker` for the rotation update system.
fn add_debug_axes_to_entity(
    commands: &mut Commands<'_, '_>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    axes: &DebugAxes,
    target: Entity,
) {
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

    // Spawn axis root as a standalone entity first, then parent it to the target.
    let axis_root = commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            DebugAxisRootMarker,
        ))
        .id();

    // Add axis meshes and labels as children of the root.
    commands.entity(axis_root).with_children(|axis_parent| {
        // X axis (red) with "X" label
        axis_parent.spawn(PbrBundle {
            mesh: meshes.add(x_axis_mesh),
            material: x_material,
            ..default()
        });
        spawn_axis_label(
            axis_parent,
            "X (right)",
            length * LABEL_POSITION_FRACTION,
            0.0,
            0.0,
            Color::srgb(1.0, 0.0, 0.0),
            length,
        );

        // Y axis (green) with "Y (up)" label
        axis_parent.spawn(PbrBundle {
            mesh: meshes.add(y_axis_mesh),
            material: y_material,
            ..default()
        });
        spawn_axis_label(
            axis_parent,
            "Y (up)",
            0.0,
            length * LABEL_POSITION_FRACTION,
            0.0,
            Color::srgb(0.0, 1.0, 0.0),
            length,
        );

        // Z axis (blue) with "Z (back)" label
        axis_parent.spawn(PbrBundle {
            mesh: meshes.add(z_axis_mesh),
            material: z_material,
            ..default()
        });
        spawn_axis_label(
            axis_parent,
            "Z (back)",
            0.0,
            0.0,
            length * LABEL_POSITION_FRACTION,
            Color::srgb(0.0, 0.0, 1.0),
            length,
        );
    });

    // Make the axis root a child of the target entity.
    commands.entity(target).push_children(&[axis_root]);
}

/// Base font size for axis labels at the reference axis length.
const LABEL_BASE_FONT_SIZE: f32 = 50.0;

/// Base transform scale for axis labels at the reference axis length.
const LABEL_BASE_SCALE: f32 = 0.01;

/// Reference axis length for label sizing.
const LABEL_REFERENCE_LENGTH: f32 = 2.0;

/// Fraction of axis length where labels are placed.
const LABEL_POSITION_FRACTION: f32 = 1.1;

/// Helper function to spawn an axis label with size adapted to the ship scale.
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
#[allow(clippy::default_trait_access)]
fn create_line_mesh(start: Vec3, end: Vec3) -> Mesh {
    let vertices = vec![start, end];
    let indices = Indices::U32(vec![0, 1]);

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, default());

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(indices);

    mesh
}
