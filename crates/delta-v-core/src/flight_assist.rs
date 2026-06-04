// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Flight assist (inertial damping) component and configuration.
//!
//! Flight assist is an accessibility feature that applies velocity damping
//! to the player ship when enabled. It does NOT instantly kill velocity;
//! instead it applies soft deceleration proportional to current velocity.
//!
//! This module also owns the deserializable ship template structs that
//! `delta-v-ships` uses to extract gameplay values from validated template
//! JSON (ADR-0014, ADR-0039, ADR-0040).
//!
//! See ADR-0009 (Newtonian physics) and ADR-0010 (configuration system).

use bevy::prelude::*;
use serde::Deserialize;

use crate::camera::ShipCamerasTemplate;

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

// ---------------------------------------------------------------------------
// Ship template structs (ADR-0014, ADR-0039, ADR-0040)
// ---------------------------------------------------------------------------

/// Base deserialized ship template JSON.
///
/// Contains common properties for all ship types (mass, inertia, propulsion).
/// This struct is produced by deserializing the validated + default-filled
/// `serde_json::Value` from the template file. Per ADR-0040, the schema is
/// the only source of defaults — no `#[serde(default)]` or `impl Default`.
#[derive(Debug, Deserialize)]
pub struct ShipTemplate {
    /// Ship mass in kilograms.
    pub mass: PhysicalQuantity,
    /// Dimensionless inertia multiplier (default 1.0 from schema).
    pub inertia_scale: f32,
    /// Propulsion system configuration.
    pub propulsion: ShipPropulsionTemplate,
}

/// Deserialized player-controlled ship template JSON.
///
/// Extends [`ShipTemplate`] with camera definitions.
/// Used only for `entity_type` `player_controlled_ship`.
/// The template JSON is merged from the `player_controlled_ship` template
/// and the referenced ship template at load time.
#[derive(Debug, Deserialize)]
pub struct PlayerShipTemplate {
    /// Ship mass in kilograms (from merged ship template).
    pub mass: PhysicalQuantity,
    /// Dimensionless inertia multiplier (from merged ship template).
    pub inertia_scale: f32,
    /// Propulsion system configuration (from merged ship template).
    pub propulsion: ShipPropulsionTemplate,
    /// Camera definitions (cockpit required, chase optional).
    pub cameras: ShipCamerasTemplate,
}

/// Physical quantity with value and unit (ADR-0008).
///
/// Deserialized from `{"value": N, "unit": "..."}` objects in template JSON.
/// The unit field is validated by the JSON schema; we only read the value.
#[derive(Debug, Deserialize)]
pub struct PhysicalQuantity {
    /// Numeric magnitude.
    pub value: f32,
    /// Unit identifier (e.g. "kg", "N", "N⋅m"). Validated by schema.
    pub unit: String,
}

/// Propulsion configuration from the ship template.
#[derive(Debug, Deserialize)]
pub struct ShipPropulsionTemplate {
    /// Main thruster configurations. For M2, exactly one is active.
    #[serde(rename = "main_thrusters")]
    pub main_thrusters: Vec<MainThrusterTemplate>,
    /// Maneuver thruster (RCS) configuration.
    pub maneuvering_thruster: ManeuveringThrusterTemplate,
}

/// Main thruster configuration from the template.
#[derive(Debug, Deserialize)]
pub struct MainThrusterTemplate {
    /// Unique identifier for this thruster within the ship.
    pub id: String,
    /// Thruster type (e.g. "chemical", "ion"). Metadata for M2.
    #[serde(rename = "type")]
    pub thruster_type: String,
    /// Maximum forward thrust.
    pub max_forward_thrust: PhysicalQuantity,
    /// Maximum backward/reverse thrust.
    pub max_backward_thrust: PhysicalQuantity,
}

/// Maneuvering thruster (RCS) configuration from the template.
#[derive(Debug, Deserialize)]
pub struct ManeuveringThrusterTemplate {
    /// Maneuvering thruster type (e.g. "rcs", "vernier"). Metadata for M2.
    #[serde(rename = "type")]
    pub thruster_type: String,
    /// Maximum torque per rotation axis.
    pub max_torque: PhysicalQuantity,
    /// Maximum strafe thrust per lateral/vertical axis.
    pub max_strafe_thrust: PhysicalQuantity,
}

/// Ship propulsion configuration read from the ship template JSON.
///
/// Contains the active thruster's force/torque values that the input → forces
/// pipeline uses each tick. Inserted as a resource at ship spawn time.
/// Per ADR-0014, all gameplay values come from JSON — this resource is
/// populated from the template, not from Rust constants.
// allow-default: Bevy requires Default on resources for init_resource.
// This is runtime state, not configuration.
#[derive(Resource, Default, Debug)]
pub struct ShipPropulsionConfig {
    /// Maximum forward thrust in Newtons (applied along -Z local axis).
    pub max_forward_thrust: f32,
    /// Maximum backward/reverse thrust in Newtons (applied along +Z local axis).
    pub max_backward_thrust: f32,
    /// Maximum torque in Newton-meters per rotation axis.
    pub max_torque: f32,
    /// Maximum strafe thrust in Newtons per lateral/vertical axis.
    pub max_strafe_thrust: f32,
    /// Index of the currently active main thruster (for M3+ thruster switching).
    pub active_main_thruster_index: usize,
}

/// Accumulated thrust command for the current fixed tick.
///
/// Written by the input reader system, consumed by the thrust system,
/// and cleared each tick.
///
/// The force is applied in the ship's local frame:
/// - +X = right, +Y = up, -Z = forward (per ADR-0006).
// allow-default: Bevy requires Default on resources for init_resource.
// This is a per-tick command buffer, not configuration.
#[derive(Resource, Default, Debug)]
pub struct ThrustCommand {
    /// Linear thrust force in Newtons (local frame).
    pub force: Vec3,
}

/// Accumulated torque command for the current fixed tick.
///
/// Written by the input reader system, consumed by the torque system,
/// and cleared each tick.
///
/// The torque is applied around the ship's local axes:
/// - X = pitch, Y = yaw, Z = roll (per ADR-0006).
// allow-default: Bevy requires Default on resources for init_resource.
// This is a per-tick command buffer, not configuration.
#[derive(Resource, Default, Debug)]
pub struct TorqueCommand {
    /// Torque in Newton-meters (local frame).
    pub torque: Vec3,
}
