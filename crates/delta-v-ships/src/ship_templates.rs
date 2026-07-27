// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Ship-specific runtime types for the ships plugin.
//!
//! These types are runtime state used by the ships plugin, not template JSON types.
//! Template JSON types are defined in `delta-v-types` with `Json` suffix.
//!
//! See ADR-0009 (Newtonian physics) and ADR-0010 (configuration system).

use bevy::prelude::*;

/// Ship propulsion configuration for resource insertion.
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
    /// Number of ticks for torque to ramp from 0% to 100% when a rotation key
    /// is first pressed. 0 = instant full torque (no ramp).
    pub rotation_ramp_ticks: u32,
    /// Pre-resolved thrust sound file path relative to assets/ (e.g., "audio/thrust.mp3" or "components/propulsion/main-thrusters/chemical-main/thrust.mp3").
    /// None if no thrust sound.
    pub thrust_sound: Option<String>,
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
