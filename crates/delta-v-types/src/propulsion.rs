// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Propulsion template types for ship configuration.

use serde::Deserialize;

/// Propulsion configuration from the ship template.
///
/// References main thruster and maneuvering thruster definitions by name.
/// The actual thruster definitions are loaded at spawn time.
#[derive(Debug, Deserialize)]
pub struct ShipPropulsionTemplate {
    /// Array of main thruster names (references to directories under assets/main-thrusters/).
    /// For M2, exactly one is active. In M3+, the player can switch between thrusters.
    #[serde(rename = "main_thrusters")]
    pub main_thruster_names: Vec<String>,
    /// Name of the maneuvering thruster (reference to a directory under assets/maneuvering-thrusters/).
    pub maneuvering_thruster: String,
}

/// Propulsion configuration returned by `build_physical_ship`.
///
/// This is a plain struct that is converted to the resource/component in the domain crate.
#[derive(Debug, Clone, Copy)]
pub struct PropulsionConfig {
    /// Maximum forward thrust in Newtons.
    pub max_forward_thrust: f32,
    /// Maximum backward thrust in Newtons.
    pub max_backward_thrust: f32,
    /// Maximum torque in Newton-meters.
    pub max_torque: f32,
    /// Maximum strafe thrust in Newtons.
    pub max_strafe_thrust: f32,
    /// Number of ticks for rotation ramp.
    pub rotation_ramp_ticks: u32,
}
