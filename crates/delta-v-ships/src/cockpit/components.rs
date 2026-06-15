// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
