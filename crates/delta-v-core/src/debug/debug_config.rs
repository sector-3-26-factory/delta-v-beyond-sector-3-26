// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Debug configuration types.
//!
//! Per ADR-0010 (configuration system), debug settings are loaded from
//! `assets/config/debug.json` with optional user overrides from
//! `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/debug.json`.
//!
//! Per ADR-0013 (no silent fallbacks), missing or invalid debug config is a hard error.

use bevy::prelude::Resource;

use delta_v_types::DebugConfigJson;

#[cfg(test)]
#[path = "debug_config_tests.rs"]
mod debug_config_tests;

/// Debug configuration loaded from `debug.json`.
///
/// Controls debug features like axis indicators, collision shape visualization,
/// and profiling. Loaded as a resource during `LoadingDefaults` state.
/// Supports hot-reload in dev builds (ADR-0035).
#[derive(Debug, Clone, Resource)]
pub struct DebugConfig {
    /// Master switch for debug axis indicators (RGB arrows from entity origins).
    /// When true with empty `axis_indicator_entities`, show axes for ALL entities.
    /// When false, no axes are shown regardless of `axis_indicator_entities`.
    pub show_axis_indicators: bool,

    /// Array of entity IDs to show debug axis indicators for.
    /// If empty and `show_axis_indicators` is true, axes are shown for all entities.
    /// If non-empty, axes are shown ONLY for these IDs (selective debugging).
    pub axis_indicator_entities: Vec<String>,

    /// Master switch for collision shape visualization.
    /// When true, renders wireframe boxes/spheres around entities with collision
    /// shapes. Green = no collision, red = collision detected this frame.
    pub show_collision_shapes: bool,
}

impl From<DebugConfigJson> for DebugConfig {
    fn from(json: DebugConfigJson) -> Self {
        Self {
            show_axis_indicators: json.show_axis_indicators,
            axis_indicator_entities: json.axis_indicator_entities,
            show_collision_shapes: json.show_collision_shapes,
        }
    }
}

impl DebugConfig {
    /// Check if axes should be shown for a given entity ID.
    ///
    /// Returns `true` if:
    /// - `show_axis_indicators` is true AND
    /// - Either `axis_indicator_entities` is empty OR the entity ID is in the list
    pub fn should_show_axes_for(&self, entity_id: &str) -> bool {
        if !self.show_axis_indicators {
            return false;
        }

        if self.axis_indicator_entities.is_empty() {
            return true;
        }

        self.axis_indicator_entities
            .contains(&entity_id.to_string())
    }
}
