// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Main thruster definition types.

use serde::Deserialize;

use crate::physics::PhysicalQuantityJson;

/// Main thruster definition deserialized from thruster definition JSON.
///
/// Defines a main thruster that can be referenced by ship templates.
/// All gameplay values come from JSON per ADR-0014.
#[derive(Debug, Deserialize, Clone)]
pub struct MainThrusterDefinitionJson {
    /// Thruster type identifier (e.g., "chemical", "ion", "fusion").
    #[serde(rename = "type")]
    pub thruster_type: String,
    /// Maximum forward thrust in Newtons.
    pub max_forward_thrust: PhysicalQuantityJson,
    /// Maximum backward/reverse thrust in Newtons.
    pub max_backward_thrust: PhysicalQuantityJson,
}

/// Runtime main thruster definition with SI units.
///
/// This is the converted version of [`MainThrusterDefinitionJson`] with all
/// physical quantities converted to SI base units (Newtons for force).
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct MainThrusterDefinition {
    /// Thruster type identifier (e.g., "chemical", "ion", "fusion").
    pub thruster_type: String,
    /// Maximum forward thrust in Newtons (SI base unit).
    pub max_forward_thrust: f32,
    /// Maximum backward/reverse thrust in Newtons (SI base unit).
    pub max_backward_thrust: f32,
}

impl From<MainThrusterDefinitionJson> for MainThrusterDefinition {
    fn from(json: MainThrusterDefinitionJson) -> Self {
        Self {
            thruster_type: json.thruster_type,
            max_forward_thrust: json.max_forward_thrust.to_newtons(),
            max_backward_thrust: json.max_backward_thrust.to_newtons(),
        }
    }
}
