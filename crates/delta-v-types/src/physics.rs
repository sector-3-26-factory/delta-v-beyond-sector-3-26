// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Physics-related types for physical quantities with units.

use serde::Deserialize;

/// Physical quantity with value and unit (ADR-0008).
///
/// Deserialized from `{"value": N, "unit": "..."}` objects in template JSON.
/// The unit field is validated by the JSON schema; we only read the value.
#[derive(Debug, Deserialize, Clone)]
pub struct PhysicalQuantityJson {
    /// Numeric magnitude.
    pub value: f32,
    /// Unit identifier (e.g. "kg", "N", "N⋅m"). Validated by schema.
    pub unit: String,
}

impl PhysicalQuantityJson {
    /// Returns the numeric value.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.value
    }

    /// Returns true if the unit matches the given string.
    #[must_use]
    pub fn unit_is(&self, unit: &str) -> bool {
        self.unit == unit
    }
}
