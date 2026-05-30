// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Debug configuration types.
//!
//! Per ADR-0010 (configuration system), debug settings are loaded from
//! `assets/config/debug.json` with optional user overrides from
//! `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/debug.json`.
//!
//! Per ADR-0013 (no silent fallbacks), missing or invalid debug config is a hard error.

use serde::Deserialize;

/// Debug configuration loaded from `debug.json`.
///
/// Controls debug features like axis indicators and profiling.
/// Loaded as a resource during `LoadingDefaults` state.
/// Supports hot-reload in dev builds (ADR-0035).
#[derive(Debug, Deserialize, Clone)]
pub struct DebugConfig {
    /// Master switch for debug axis indicators (RGB arrows from entity origins).
    /// When true with empty `axis_indicator_entities`, show axes for ALL entities.
    /// When false, no axes are shown regardless of `axis_indicator_entities`.
    pub show_axis_indicators: bool,

    /// Array of entity IDs to show debug axis indicators for.
    /// If empty and `show_axis_indicators` is true, axes are shown for all entities.
    /// If non-empty, axes are shown ONLY for these IDs (selective debugging).
    pub axis_indicator_entities: Vec<String>,
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
