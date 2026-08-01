// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Configuration types for game settings loaded from JSON.
//!
//! These are plain data types with serde deserialization, used by
//! `delta-v-config` to load configuration from JSON files. Per ADR-0046,
//! these types live in `delta-v-types` since they are shared across crates.

use serde::Deserialize;

/// Debug configuration loaded from `debug.json`.
///
/// Controls debug features like axis indicators, collision shape visualization,
/// and profiling. Loaded as a resource during `LoadingDefaults` state.
/// Supports hot-reload in dev builds (ADR-0035).
#[derive(Debug, Deserialize, Clone)]
pub struct DebugConfigJson {
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

/// Frame-time diagnostics configuration loaded from `diagnostics.json`.
///
/// Loaded from `assets/config/diagnostics.json` during plugin build.
/// All defaults are in the JSON schema (ADR-0039).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DiagnosticsConfigJson {
    /// Frame time threshold in milliseconds (converted to seconds internally).
    /// Must be provided by diagnostics.json.
    pub frame_time_warn_threshold_ms: f64,

    /// Number of consecutive frames over threshold before a WARN is emitted.
    /// Must be provided by diagnostics.json.
    pub consecutive_frames_threshold: u32,
}

/// Flight assist configuration loaded from `flight-assist.json`.
///
/// All defaults are in the JSON schema (ADR-0012, ADR-0039).
/// This type is deserialised via `delta-v-json` (ADR-0040).
#[derive(Debug, Deserialize, Clone)]
pub struct FlightAssistConfigJson {
    /// Whether flight assist is enabled on startup.
    pub enabled_by_default: bool,

    /// Velocity damping coefficient (0-1). Higher = more aggressive damping.
    pub damping_coefficient: f32,
}
