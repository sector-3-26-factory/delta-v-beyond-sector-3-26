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
