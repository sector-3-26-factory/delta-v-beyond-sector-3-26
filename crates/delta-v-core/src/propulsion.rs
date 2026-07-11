// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Propulsion component for ship entities.

use bevy::prelude::*;

/// Propulsion component for a ship entity.
///
/// Contains references to thruster definitions and merged thrust/torque values.
/// This component is added to ship entities at spawn time.
#[derive(Component, Debug, Clone)]
pub struct Propulsion {
    /// Names of main thruster assets (references to directories under assets/components/propulsion/main-thrusters/).
    pub main_thruster_names: Vec<String>,
    /// Name of the maneuvering thruster (reference to a directory under assets/components/propulsion/maneuvering-thrusters/).
    pub maneuvering_thruster_name: String,
    /// Maximum forward thrust in Newtons (merged from active main thruster).
    pub max_forward_thrust: f32,
    /// Maximum backward thrust in Newtons (merged from active main thruster).
    pub max_backward_thrust: f32,
    /// Maximum torque in Newton-meters (from maneuvering thruster).
    pub max_torque: f32,
    /// Maximum strafe thrust in Newtons (from maneuvering thruster).
    pub max_strafe_thrust: f32,
    /// Number of ticks for torque to ramp from 0% to 100% when a rotation key is first pressed.
    pub rotation_ramp_ticks: u32,
    /// Index of the currently active main thruster (for M3+ thruster switching).
    pub active_main_thruster_index: usize,
}
