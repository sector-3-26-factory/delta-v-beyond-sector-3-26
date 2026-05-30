// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Debug axis indicators: RGB arrows from entity centers.
//!
//! Per ADR-0022 (performance instrumentation), debug axes help visualize
//! entity orientations and alignment during development.
//!
//! TODO (Phase 8): Implement `spawn_debug_axes` system that creates line meshes
//! based on `DebugConfig` settings. System will run during `SpawningEntities` state.

use bevy::prelude::*;

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
