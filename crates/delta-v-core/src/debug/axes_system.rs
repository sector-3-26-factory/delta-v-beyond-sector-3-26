// AGENTS: before modifying this file, read AGENTS.md at the repository root.

use bevy::prelude::*;

use super::axes::{AxesVisibility, AxisLabel, DebugAxes, DebugAxesEligible};
use super::debug_config::DebugConfig;
use crate::camera::{ActiveCameraName, ActiveMainCamera, RenderLayer};
use crate::events::CameraSwitched;

/// Updates gizmo render layers to match the active camera's layer.
///
/// This system runs when the active camera changes and updates the gizmo
/// configuration so that debug axes are only rendered by the active camera.
// INVARIANT: Query returns at most one camera with ActiveMainCamera marker.
#[allow(clippy::needless_pass_by_value)]
pub fn update_gizmo_render_layers(
    mut gizmo_config_store: ResMut<'_, bevy::gizmos::config::GizmoConfigStore>,
    camera_query: Query<'_, '_, &bevy::camera::visibility::RenderLayers, (With<ActiveMainCamera>,)>,
) {
    let Ok(camera_layers) = camera_query.single() else {
        return;
    };

    // Extract the first layer from the camera's render layers
    // The active camera should have exactly one layer assigned
    if let Some(_layer) = camera_layers.iter().next() {
        let (gizmo_config, _) =
            gizmo_config_store.config_mut::<bevy::gizmos::config::DefaultGizmoConfigGroup>();
        gizmo_config.render_layers = RenderLayer::Gameplay.render_layers();
    }
}

/// Marks eligible entities with `DebugAxes` if debug config enables visualization.
// INVARIANT: Query returns only newly-marked entities via Added<DebugAxesEligible>.
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
/// Gizmo render layers are configured at app startup to match the active
/// camera's render layer, so gizmos are only rendered by the active camera.
// INVARIANT: Gizmos are drawn in world space; no pass-by-value optimization applies.
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
// INVARIANT: Query returns only newly-spawned debug axes via Added<DebugAxes>.
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
                    font_size: bevy::prelude::FontSize::Px(16.0),
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
///
/// Respects the `AxesVisibility` resource set by `debug_axes_visibility_system`.
/// Labels are only shown when axes are visible AND the label is on screen.
// INVARIANT: Query returns at most one camera and one window; type complexity from multiple query params.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::type_complexity)]
pub fn update_debug_axis_labels(
    axes_visibility: Res<'_, AxesVisibility>,
    camera_query: Query<
        '_,
        '_,
        (&Camera, &GlobalTransform),
        (With<Camera3d>, With<ActiveMainCamera>),
    >,
    entity_query: Query<'_, '_, &GlobalTransform>,
    mut label_query: Query<'_, '_, (&AxisLabel, &mut Node, &mut Visibility)>,
) {
    // If axes are not visible, hide all labels
    if !axes_visibility.visible {
        for (_label, _node, mut visibility) in &mut label_query {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    for (label, mut node, mut visibility) in &mut label_query {
        let Ok(entity_transform) = entity_query.get(label.entity) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        // Axes are world-aligned (not rotated with the entity), matching render_debug_axes
        let world_pos = entity_transform.translation() + label.offset;

        if let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, world_pos) {
            node.left = Val::Px(viewport_pos.x);
            node.top = Val::Px(viewport_pos.y);
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Initializes debug axes visibility based on the initial camera state.
///
/// This runs once when entering `InGame` state to set the correct initial visibility.
/// Without this, axes would be hidden on startup even if the cockpit camera is active.
#[allow(clippy::needless_pass_by_value)]
pub fn init_debug_axes_visibility(
    active_camera: Res<'_, ActiveCameraName>,
    debug_config: Res<'_, DebugConfig>,
    mut gizmo_config_store: ResMut<'_, bevy::gizmos::config::GizmoConfigStore>,
    mut axes_visibility: ResMut<'_, AxesVisibility>,
) {
    // Only process if debug axes are enabled at all
    if !debug_config.show_axis_indicators {
        return;
    }

    // Axes visible on cockpit and front cameras
    let is_active = active_camera.0 == "cockpit" || active_camera.0 == "front";

    // Toggle gizmo visibility
    let (gizmo_config, _) =
        gizmo_config_store.config_mut::<bevy::gizmos::config::DefaultGizmoConfigGroup>();
    gizmo_config.enabled = is_active;

    // Update the AxesVisibility resource
    axes_visibility.visible = is_active;

    tracing::debug!(
        "[debug] axes visibility: initialized to {} for camera '{}'",
        is_active,
        active_camera.0
    );
}

/// Toggles debug axes visibility based on the active camera.
///
/// Debug axes (gizmos and labels) are only visible when the cockpit or front camera is active.
/// This system listens for `CameraSwitched` messages and updates the `AxesVisibility` resource.
///
/// Runs in `Update` during `AppState::InGame`.
#[allow(clippy::needless_pass_by_value)]
pub fn debug_axes_visibility_system(
    mut events: MessageReader<'_, '_, CameraSwitched>,
    debug_config: Res<'_, DebugConfig>,
    mut gizmo_config_store: ResMut<'_, bevy::gizmos::config::GizmoConfigStore>,
    mut axes_visibility: ResMut<'_, AxesVisibility>,
) {
    let events: Vec<_> = events.read().collect();
    if events.is_empty() {
        return;
    }

    // Only process if debug axes are enabled at all
    if !debug_config.show_axis_indicators {
        tracing::debug!("[debug] axes visibility: debug axes disabled by config");
        return;
    }

    for event in events {
        // Axes visible on cockpit and front cameras
        let is_active = event.camera_name == "cockpit" || event.camera_name == "front";

        // Toggle gizmo visibility
        let (gizmo_config, _) =
            gizmo_config_store.config_mut::<bevy::gizmos::config::DefaultGizmoConfigGroup>();
        gizmo_config.enabled = is_active;

        // Update the AxesVisibility resource
        axes_visibility.visible = is_active;

        tracing::debug!(
            "[debug] axes visibility: set to {} for camera '{}'",
            is_active,
            event.camera_name
        );
    }
}
