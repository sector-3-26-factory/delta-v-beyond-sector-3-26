// AGENTS: before modifying this file, read AGENTS.md at the repository root.

use bevy::prelude::*;

/// Marker component for debug axis labels.
///
/// Stores the entity being labeled and the world-space offset from that entity
/// where the label should be positioned.
#[derive(Component)]
pub struct AxisLabel {
    /// The entity being labeled.
    pub entity: Entity,
    /// World-space offset from the entity where the label is positioned.
    pub offset: Vec3,
}

/// Marker component for entities eligible to have debug axes rendered.
///
/// Domain plugins (e.g., `ShipsPlugin`) spawn entities and mark them with this component.
/// The `mark_debug_axes` system (in `CorePlugin`) reads this marker and adds `DebugAxes`
/// if debug config enables visualization. Per ADR-0005, this decouples debug
/// visualization from domain plugins.
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
/// The axes are rendered using Bevy's gizmo system for lines, with UI text
/// labels positioned via viewport projection. Per ADR-0006, axes follow the
/// right-handed coordinate system: +X right, +Y up, -Z forward.
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

/// Resource tracking whether debug axes should be visible based on active camera.
///
/// This is set by `debug_axes_visibility_system` and read by `update_debug_axis_labels`.
/// Axes are visible when the cockpit or front camera is active.
// allow-default: Bevy requires Default on resources for init_resource.
// This is runtime state, not configuration.
#[derive(Resource, Default)]
pub struct AxesVisibility {
    /// Whether axes should be visible.
    pub visible: bool,
}
