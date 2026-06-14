// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Cockpit overlay components and types.
//!
//! Defines the data structures for cockpit overlays, stations, and gauge slots.

use bevy::prelude::*;

/// Marker component for the cockpit overlay entity.
///
/// Holds the handle to the current overlay texture (PNG with alpha transparency).
#[derive(Component)]
pub struct CockpitOverlay {
    /// Handle to the overlay texture.
    pub texture: Handle<Image>,
}

/// Marker component for the cockpit overlay entity.
#[derive(Component)]
pub struct CockpitOverlayEntity;

/// Resource holding the active cockpit station ID.
///
/// This resource is inserted by [`super::spawn::spawn_cockpit_overlay`] with
/// the station ID from the loaded cockpit template. It is NOT initialized
/// with a default — missing cockpit configuration is a hard error (ADR-0013).
#[derive(Resource)]
pub struct ActiveCockpitStation {
    /// ID of the currently active station.
    pub station_id: String,
}

// Re-export types from ship_templates for convenience
pub use crate::ship_templates::{CockpitDefinition, CockpitStation, GaugeShape, GaugeSlot};
