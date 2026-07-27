// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Maneuvering thruster definition types.

use serde::Deserialize;

use crate::physics::PhysicalQuantityJson;

/// Maneuvering thruster definition deserialized from thruster definition JSON.
///
/// Defines an RCS maneuvering thruster that can be referenced by ship templates.
/// All gameplay values come from JSON per ADR-0014.
#[derive(Debug, Deserialize, Clone)]
pub struct ManeuveringThrusterDefinitionJson {
    /// Maneuvering thruster type identifier (e.g., "rcs", "vernier", "electromagnetic").
    #[serde(rename = "type")]
    pub thruster_type: String,
    /// Maximum torque in Newton-meters per rotation axis.
    pub max_torque: PhysicalQuantityJson,
    /// Maximum strafe thrust in Newtons per lateral/vertical axis.
    pub max_strafe_thrust: PhysicalQuantityJson,
    /// Number of ticks for torque to ramp from 0% to 100% when a rotation key is first pressed.
    /// Default 60 via schema.
    pub rotation_ramp_ticks: u32,
}

/// Runtime maneuvering thruster definition with SI units.
///
/// This is the converted version of [`ManeuveringThrusterDefinitionJson`] with all
/// physical quantities converted to SI base units (Newtons for force, N⋅m for torque).
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct ManeuveringThrusterDefinition {
    /// Maneuvering thruster type identifier (e.g., "rcs", "vernier", "electromagnetic").
    pub thruster_type: String,
    /// Maximum torque in Newton-meters per rotation axis (SI base unit).
    pub max_torque: f32,
    /// Maximum strafe thrust in Newtons per lateral/vertical axis (SI base unit).
    pub max_strafe_thrust: f32,
    /// Number of ticks for torque to ramp from 0% to 100% when a rotation key is first pressed.
    pub rotation_ramp_ticks: u32,
}

impl From<ManeuveringThrusterDefinitionJson> for ManeuveringThrusterDefinition {
    fn from(json: ManeuveringThrusterDefinitionJson) -> Self {
        Self {
            thruster_type: json.thruster_type,
            max_torque: json.max_torque.to_newton_meters(),
            max_strafe_thrust: json.max_strafe_thrust.to_newtons(),
            rotation_ramp_ticks: json.rotation_ramp_ticks,
        }
    }
}
