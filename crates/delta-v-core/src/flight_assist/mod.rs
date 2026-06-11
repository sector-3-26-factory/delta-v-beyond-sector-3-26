// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Flight assist (inertial damping) component and configuration.
//!
//! Flight assist is an accessibility feature that applies velocity damping
//! to the player ship when enabled. It does NOT instantly kill velocity;
//! instead it applies soft deceleration proportional to current velocity.
//!
//! See ADR-0009 (Newtonian physics) and ADR-0010 (configuration system).

use bevy::prelude::*;
use serde::Deserialize;

/// Marker component for entities that use flight assist (inertial damping).
///
/// When this component is present on an entity and flight assist is enabled,
/// the flight-assist system applies velocity damping each fixed tick.
#[derive(Component, Clone, Copy, Debug)]
pub struct FlightAssist;

/// Flight assist configuration loaded from `flight-assist.json`.
///
/// All defaults are in the JSON schema (ADR-0012, ADR-0039).
/// This type is deserialised via `delta-v-json` (ADR-0040).
#[derive(Resource, Debug, Clone, Deserialize)]
pub struct FlightAssistConfig {
    /// Whether flight assist is enabled on startup.
    pub enabled_by_default: bool,
    /// Velocity damping coefficient (0-1). Higher = more aggressive damping.
    pub damping_coefficient: f32,
}

/// Current runtime state of flight assist.
///
/// Toggled by the player via the `toggle_flight_assist` action.
/// Initialised from [`FlightAssistConfig::enabled_by_default`].
// allow-default: Bevy requires Default on resources for init_resource.
// This is runtime state, not configuration.
#[derive(Resource, Default, Debug)]
pub struct FlightAssistState {
    /// Whether flight assist is currently active.
    pub enabled: bool,
}
