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

/// Marker component for the velocity vector indicator entity.
///
/// The indicator is a small UI element (arrow/reticle) that points in the
/// direction of the ship's velocity vector, projected onto the active camera's
/// view plane. Visible in all cameras (not just cockpit).
#[derive(Component)]
pub struct VelocityVectorIndicator;

/// Marker component for the speed display text entity.
///
/// Displays the ship's current speed next to the velocity vector indicator.
/// Positioned independently (not a child of the rotating arrow sprite).
#[derive(Component)]
pub struct SpeedText;

/// A status gauge UI element rendered in a cockpit slot.
///
/// Gauges display gameplay state (health, weapon heat) as a fill indicator.
/// The gauge type is determined by the `default_gauge` field of the [`GaugeSlot`]
/// definition in the cockpit JSON. Supported types: `"health"`, `"weapon_heat"`.
///
/// The gauge is spawned as a UI node at the position/size defined by the slot shape.
/// The `status_gauge_system` updates the fill level each frame.
#[derive(Component)]
pub struct StatusGauge {
    /// The station ID this gauge belongs to.
    pub station_id: String,
    /// The gauge slot ID this gauge is bound to.
    pub slot_id: String,
    /// The shape defining the gauge position and size.
    pub shape: GaugeShape,
    /// The gauge type (e.g., "health", "`weapon_heat`").
    pub gauge_type: String,
}

/// Marker component for the circular gauge needle/pointer.
/// Used to identify the needle sprite for rotation updates.
#[derive(Component)]
pub struct CircularGaugeNeedle {
    /// The station ID this needle belongs to.
    pub station_id: String,
    /// The gauge slot ID this needle is bound to.
    pub slot_id: String,
    /// The center X position of the gauge in viewport coordinates (0-100).
    pub center_x: f32,
    /// The center Y position of the gauge in viewport coordinates (0-100).
    pub center_y: f32,
}

/// Marker component for entities that can be targeted in combat.
///
/// These entities appear in the targeting list and can be selected as the
/// player's current target.
#[derive(Component)]
pub struct Targetable;

/// Marker component for entities that appear in the navigation list.
///
/// These entities can be selected for navigation purposes (ships, fleets,
/// planets, stations, asteroids).
#[derive(Component)]
pub struct Navigable;

// EntityType, WorldEntityId, SelectedTarget, SelectedNavObject, TargetingMode, TargetingModeType
// are now defined in delta-v-core::navigation
// Re-export them for convenience
pub use delta_v_core::navigation::{
    EntityType, SelectedNavObject, SelectedTarget, TargetingMode, TargetingModeType, WorldEntityId,
};

/// Marker component for the bearing indicator arrow.
///
/// Points toward the selected target when it's off-screen.
#[derive(Component)]
pub struct BearingIndicator;

/// Marker component for the on-screen target reticle.
///
/// Appears around the selected target when it's on-screen.
#[derive(Component)]
pub struct TargetReticle;

/// Component for camera shake effect.
///
/// When active, applies a random offset to the camera's position that decays over time.
/// Triggered by weapon fire, projectile hits, and collisions.
#[derive(Component)]
pub struct CameraShake {
    /// Current shake intensity (world units).
    pub intensity: f32,
    /// Total duration in fixed timestep ticks (60 Hz).
    pub duration_ticks: u32,
    /// Number of ticks elapsed since shake started.
    pub elapsed_ticks: u32,
    /// The camera's original translation before the shake started.
    /// Used to restore the camera position when the shake completes.
    pub original_translation: Vec3,
}

impl CameraShake {
    /// Creates a new camera shake with the given intensity and duration.
    ///
    /// # Arguments
    /// * `intensity` - Maximum offset in world units.
    /// * `duration_ticks` - Duration in fixed timestep ticks (60 Hz).
    #[must_use]
    pub const fn new(intensity: f32, duration_ticks: u32) -> Self {
        Self {
            intensity,
            duration_ticks,
            elapsed_ticks: 0,
            original_translation: Vec3::ZERO,
        }
    }
}

// Re-export types from ship_templates for convenience
pub use crate::ship_templates::{CockpitDefinition, CockpitStation, GaugeShape, GaugeSlot};
