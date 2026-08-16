// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Debug axis indicators: RGB arrows from entity centers.
//!
//! Per ADR-0022 (performance instrumentation), debug axes help visualize
//! entity orientations and alignment during development.
//!
//! The spawning process is decoupled per ADR-0005 (plugin architecture):
//! - Domain plugins (`ShipsPlugin`, etc.) mark entities with `DebugAxesEligible`.
//! - `CorePlugin`'s `mark_debug_axes` system converts eligible entities to `DebugAxes`.
//! - `CorePlugin`'s `render_debug_axes` system renders the axes using gizmos.
//! - `CorePlugin`'s `update_debug_axis_labels` system positions UI labels via viewport projection.
//!
//! Gizmos are used for axis lines, and UI text with viewport projection for labels.
//! Per ADR-0044, debug visualization is exempt from the "no visual data in Rust" rule.
//!
//! - X axis: red line with "X (right)" label
//! - Y axis: green line with "Y (up)" label
//! - Z axis: blue line with "Z (back)" label
//!
//! Length is calculated as 2× the entity's longest expansion along any axis.

/// Debug axis marker components (`DebugAxesEligible`, `DebugAxes`).
pub mod axes;
/// Debug axis systems (`mark_debug_axes`, `render_debug_axes`, `update_debug_axis_labels`).
pub mod axes_system;
/// Debug configuration (`DebugConfig`).
pub mod debug_config;
/// Light source debugging utilities.
pub mod light_debug;

#[cfg(test)]
#[path = "axes_tests.rs"]
mod axes_tests;

pub use axes::{AxesVisibility, AxisLabel, DebugAxes, DebugAxesEligible};
pub use axes_system::{
    debug_axes_visibility_system, init_debug_axes_visibility, mark_debug_axes, render_debug_axes,
    spawn_debug_axis_labels, update_debug_axis_labels, update_gizmo_render_layers,
};
pub use debug_config::DebugConfig;
pub use light_debug::{LightDebugPlugin, debug_log_all_lights, debug_log_lights_once};
