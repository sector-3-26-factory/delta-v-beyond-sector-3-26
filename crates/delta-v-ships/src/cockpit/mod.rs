// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Cockpit overlay system for ship-specific HUD views.
//!
//! This module handles:
//! - Cockpit overlay PNG rendering with alpha transparency
//! - Station switching (F2 / Shift+F2)
//! - Gauge slot positioning
//!
//! See M6 -- HUD and Feel plan.

pub mod components;
pub mod resources;
pub mod spawn;
pub mod systems;

pub use components::*;
