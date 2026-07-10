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
    /// Sound file path relative to assets/audio/ (e.g., "thrust.wav").
    /// Optional; if not provided, no thrust sound is played.
    pub sound: Option<String>,
}
