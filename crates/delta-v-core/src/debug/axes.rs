// AGENTS: before modifying this file, read AGENTS.md at the repository root.

use bevy::prelude::*;

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
/// The axes are spawned as CHILDREN of the target entity, with a [`DebugAxisRootMarker`]
/// component on the root. The `update_debug_axes_rotation` system inverts the parent's
/// rotation to keep the axes world-aligned.
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

/// Marker component on a debug axis root entity.
///
/// Used to identify axis root entities for the inverse rotation update system.
/// The axis root is a child of the target entity, and this marker allows the
/// update system to find it and set its rotation to the inverse of the parent's.
#[derive(Component)]
pub struct DebugAxisRootMarker;
