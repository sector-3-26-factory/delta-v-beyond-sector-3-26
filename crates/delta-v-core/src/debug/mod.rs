// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Debug axis indicators: RGB arrows from entity centers.
//!
//! Per ADR-0022 (performance instrumentation), debug axes help visualize
//! entity orientations and alignment during development.
//!
//! The spawning process is decoupled per ADR-0005 (plugin architecture):
//! - Domain plugins (`ShipsPlugin`, etc.) mark entities with `DebugAxesEligible`.
//! - `CorePlugin`'s `mark_debug_axes` system converts eligible entities to `DebugAxes`.
//! - `CorePlugin`'s `spawn_debug_axes` system renders the axes and labels.
//!
//! Axes are spawned as CHILDREN of their target entity. Each axis root has a
//! `DebugAxisRootMarker` component. The `update_debug_axes_rotation` system
//! queries these markers, finds their parent's rotation, and sets the root's
//! local rotation to the inverse, keeping axes world-aligned.
//!
//! The visibility chain is: target → axis root → axis (→ axis label?).
//! The axis root has `Visibility`, `InheritedVisibility`, and `ViewVisibility`
//! to ensure proper visibility propagation to its children.
//!
//! - X axis: red line with "X (right)" label
//! - Y axis: green line with "Y (up)" label
//! - Z axis: blue line with "Z (backward)" label
//!
//! Length is calculated as 2× the entity's longest expansion along any axis.
//! When the axis length changes (e.g. after a glTF mesh finishes loading and
//! the bounding box is computed), the old axis root is despawned and a new one
//! is created with the updated length.

/// Debug axis marker components (`DebugAxesEligible`, `DebugAxes`, `DebugAxisRootMarker`).
pub mod axes;
/// Debug axis systems (`mark_debug_axes`, `spawn_debug_axes`, `update_debug_axes_on_change`, `update_debug_axes_rotation`).
pub mod axes_system;
/// Debug configuration (`DebugConfig`).
pub mod debug_config;

#[cfg(test)]
#[path = "axes_tests.rs"]
mod axes_tests;

pub use axes::{DebugAxes, DebugAxesEligible, DebugAxisRootMarker};
pub use axes_system::{
    mark_debug_axes, spawn_debug_axes, update_debug_axes_on_change, update_debug_axes_rotation,
};
pub use debug_config::DebugConfig;
