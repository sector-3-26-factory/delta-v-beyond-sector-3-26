// AGENTS: before modifying this file, read AGENTS.md at the repository root.

use bevy::prelude::*;

use super::axes::{AxisLabel, DebugAxes, DebugAxesEligible};
use super::debug_config::DebugConfig;
use crate::camera::ActiveMainCamera;

/// Marks eligible entities with `DebugAxes` if debug config enables visualization.
#[allow(clippy::needless_pass_by_value)]
pub fn mark_debug_axes(
    mut commands: Commands<'_, '_>,
    debug_config: Res<'_, DebugConfig>,
    query: Query<'_, '_, (Entity, &DebugAxesEligible), Added<DebugAxesEligible>>,
) {
    if !debug_config.show_axis_indicators {
        return;
    }

    for (entity, eligible) in query.iter() {
        if debug_config.should_show_axes_for(&eligible.entity_id) {
            commands.entity(entity).insert(DebugAxes::new(
                eligible.entity_id.clone(),
                eligible.axis_length,
            ));
        }
    }
}

/// Renders debug axis lines using gizmos.
///
/// Gizmo render layers are configured at app startup to match the chase
/// camera's render layer, so gizmos are only rendered by the active camera.
#[allow(clippy::needless_pass_by_value)]
pub fn render_debug_axes(
    debug_config: Res<'_, DebugConfig>,
    query: Query<'_, '_, (&GlobalTransform, &DebugAxes)>,
    mut gizmos: Gizmos<'_, '_>,
) {
    for (global_transform, axes) in &query {
        if !debug_config.should_show_axes_for(&axes.entity_id) {
            continue;
        }

        let length = axes.axis_length;
        let origin = global_transform.translation();

        gizmos.line(
            origin,
            origin + Vec3::new(length, 0.0, 0.0),
            Color::srgb(1.0, 0.0, 0.0),
        );
        gizmos.line(
            origin,
            origin + Vec3::new(0.0, length, 0.0),
            Color::srgb(0.0, 1.0, 0.0),
        );
        gizmos.line(
            origin,
            origin + Vec3::new(0.0, 0.0, length),
            Color::srgb(0.0, 0.0, 1.0),
        );
    }
}

/// Spawns debug axis label UI text entities.
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_debug_axis_labels(
    debug_config: Res<'_, DebugConfig>,
    query: Query<'_, '_, (Entity, &GlobalTransform, &DebugAxes), Added<DebugAxes>>,
    mut commands: Commands<'_, '_>,
) {
    for (entity, _global_transform, axes) in &query {
        if !debug_config.should_show_axes_for(&axes.entity_id) {
            continue;
        }

        let length = axes.axis_length;

        for (label_text, offset, color) in [
            ("X", Vec3::new(length, 0.0, 0.0), Color::srgb(1.0, 0.0, 0.0)),
            ("Y", Vec3::new(0.0, length, 0.0), Color::srgb(0.0, 1.0, 0.0)),
            ("Z", Vec3::new(0.0, 0.0, length), Color::srgb(0.0, 0.0, 1.0)),
        ] {
            commands.spawn((
                Text::new(label_text),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(color),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                AxisLabel { entity, offset },
            ));
        }
    }
}

/// Updates debug axis labels by projecting 3D world positions to 2D screen coordinates.
///
/// Uses the `ActiveMainCamera` marker to find the currently active camera,
/// and computes world position as `translation() + offset` (world-space, no rotation)
/// to match how `render_debug_axes` draws the gizmo lines.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::type_complexity)]
pub fn update_debug_axis_labels(
    camera_query: Query<
        '_,
        '_,
        (&Camera, &GlobalTransform),
        (With<Camera3d>, With<ActiveMainCamera>),
    >,
    window_query: Query<'_, '_, &Window>,
    entity_query: Query<'_, '_, &GlobalTransform>,
    mut label_query: Query<'_, '_, (&AxisLabel, &mut Node, &mut Visibility)>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(window) = window_query.single() else {
        return;
    };

    let window_height = window.height();

    for (label, mut node, mut visibility) in &mut label_query {
        let Ok(entity_transform) = entity_query.get(label.entity) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let world_pos = entity_transform.translation() + label.offset;

        if let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_pos) {
            node.left = Val::Px(viewport_pos.x);
            node.top = Val::Px(window_height - viewport_pos.y);
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
