// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Physics-related types for physical quantities with units.

use bevy::prelude::Vec3;
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

    /// Converts a length quantity to meters (SI base unit).
    ///
    /// # Panics
    ///
    /// Panics if the unit is not a valid length unit (m, km, Mm, Gm, AU, ly, pc, kpc, Mpc).
    #[allow(clippy::panic)] // INVARIANT: unit is validated by JSON schema (ADR-0008, ADR-0013)
    #[must_use]
    pub fn to_meters(&self) -> f32 {
        let conversion_factor = match self.unit.as_str() {
            "m" => 1.0,
            "km" => 1e3,
            "Mm" => 1e6,
            "Gm" => 1e9,
            "AU" => 1.495_978_6e11,
            "ly" => 9.460_731e15,
            "pc" => 3.085_677_6e16,
            "kpc" => 3.085_677_6e19,
            "Mpc" => 3.085_677_5e22,
            _ => panic!("Invalid length unit: {}", self.unit),
        };
        self.value * conversion_factor
    }

    /// Converts a mass quantity to kilograms (SI base unit).
    ///
    /// # Panics
    ///
    /// Panics if the unit is not a valid mass unit (kg, t, `M_earth`, `M_sun`).
    #[allow(clippy::panic)] // INVARIANT: unit is validated by JSON schema (ADR-0008, ADR-0013)
    #[must_use]
    pub fn to_kilograms(&self) -> f32 {
        let conversion_factor = match self.unit.as_str() {
            "kg" => 1.0,
            "t" => 1e3,
            "M_earth" => 5.972_2e24,
            "M_sun" => 1.988_47e30,
            _ => panic!("Invalid mass unit: {}", self.unit),
        };
        self.value * conversion_factor
    }

    /// Converts a time quantity to seconds (SI base unit).
    ///
    /// # Panics
    ///
    /// Panics if the unit is not a valid time unit (s, min, h, d, a).
    #[allow(clippy::panic)] // INVARIANT: unit is validated by JSON schema (ADR-0008, ADR-0013)
    #[must_use]
    pub fn to_seconds(&self) -> f32 {
        let conversion_factor = match self.unit.as_str() {
            "s" => 1.0,
            "min" => 60.0,
            "h" => 3600.0,
            "d" => 86400.0,
            "a" => 31_557_600.0,
            _ => panic!("Invalid time unit: {}", self.unit),
        };
        self.value * conversion_factor
    }

    /// Converts an angle quantity to radians.
    ///
    /// # Panics
    ///
    /// Panics if the unit is not a valid angle unit (rad, deg).
    #[allow(clippy::panic)] // INVARIANT: unit is validated by JSON schema (ADR-0008, ADR-0013)
    #[must_use]
    pub fn to_radians(&self) -> f32 {
        match self.unit.as_str() {
            "rad" => self.value,
            "deg" => self.value.to_radians(),
            _ => panic!("Invalid angle unit: {}", self.unit),
        }
    }

    /// Converts a force quantity to Newtons (SI base unit).
    ///
    /// # Panics
    ///
    /// Panics if the unit is not a valid force unit (N, kN, MN).
    #[allow(clippy::panic)] // INVARIANT: unit is validated by JSON schema (ADR-0008, ADR-0013)
    #[must_use]
    pub fn to_newtons(&self) -> f32 {
        let conversion_factor = match self.unit.as_str() {
            "N" => 1.0,
            "kN" => 1e3,
            "MN" => 1e6,
            _ => panic!("Invalid force unit: {}", self.unit),
        };
        self.value * conversion_factor
    }

    /// Converts a torque quantity to Newton-meters (SI base unit).
    ///
    /// # Panics
    ///
    /// Panics if the unit is not a valid torque unit (N⋅m).
    #[allow(clippy::panic)] // INVARIANT: unit is validated by JSON schema (ADR-0008, ADR-0013)
    #[must_use]
    pub fn to_newton_meters(&self) -> f32 {
        // Torque is always in N⋅m per the schema - no conversion needed
        // but we validate the unit for consistency
        match self.unit.as_str() {
            "N⋅m" => self.value,
            _ => panic!("Invalid torque unit: {}", self.unit),
        }
    }

    /// Converts a velocity quantity to meters per second (SI base unit).
    ///
    /// # Panics
    ///
    /// Panics if the unit is not a valid velocity unit (m/s, km/s).
    #[allow(clippy::panic)] // INVARIANT: unit is validated by JSON schema (ADR-0008, ADR-0013)
    #[must_use]
    pub fn to_meters_per_second(&self) -> f32 {
        let conversion_factor = match self.unit.as_str() {
            "m/s" => 1.0,
            "km/s" => 1e3,
            _ => panic!("Invalid velocity unit: {}", self.unit),
        };
        self.value * conversion_factor
    }

    /// Converts an acceleration quantity to meters per second squared (SI base unit).
    ///
    /// # Panics
    ///
    /// Panics if the unit is not a valid acceleration unit (m/s²).
    #[allow(clippy::panic)] // INVARIANT: unit is validated by JSON schema (ADR-0008, ADR-0013)
    #[must_use]
    pub fn to_meters_per_second_squared(&self) -> f32 {
        // Acceleration is always in m/s² per the schema - no conversion needed
        // but we validate the unit for consistency
        match self.unit.as_str() {
            "m/s²" => self.value,
            _ => panic!("Invalid acceleration unit: {}", self.unit),
        }
    }

    /// Converts a frequency quantity to Hertz (SI base unit).
    ///
    /// # Panics
    ///
    /// Panics if the unit is not a valid frequency unit (Hz).
    #[allow(clippy::panic)] // INVARIANT: unit is validated by JSON schema (ADR-0008, ADR-0013)
    #[must_use]
    pub fn to_hertz(&self) -> f32 {
        // Frequency is always in Hz per the schema - no conversion needed
        // but we validate the unit for consistency
        match self.unit.as_str() {
            "Hz" => self.value,
            _ => panic!("Invalid frequency unit: {}", self.unit),
        }
    }

    /// Converts a damage quantity to hit points.
    ///
    /// # Panics
    ///
    /// Panics if the unit is not a valid damage unit (hp).
    #[allow(clippy::panic)] // INVARIANT: unit is validated by JSON schema (ADR-0008, ADR-0013)
    #[must_use]
    pub fn to_hit_points(&self) -> f32 {
        // Damage is always in hp per the schema - no conversion needed
        // but we validate the unit for consistency
        match self.unit.as_str() {
            "hp" => self.value,
            _ => panic!("Invalid damage unit: {}", self.unit),
        }
    }

    /// Converts a dimensionless quantity to its numeric value.
    ///
    /// # Panics
    ///
    /// Panics if the unit is not "dimensionless".
    #[allow(clippy::panic)] // INVARIANT: unit is validated by JSON schema (ADR-0008, ADR-0013)
    #[must_use]
    pub fn to_dimensionless(&self) -> f32 {
        // Dimensionless is always dimensionless - no conversion needed
        // but we validate the unit for consistency
        match self.unit.as_str() {
            "dimensionless" => self.value,
            _ => panic!("Invalid dimensionless unit: {}", self.unit),
        }
    }
}

/// Plain data for a rigid body (no Bevy `Component` derive).
///
/// Used by `delta-v-spawn` to construct physics components.
/// The physics domain wraps this in `RigidBody` (Component) in `delta-v-physics`.
/// See ADR-0046 for the data/runtime type separation pattern.
#[derive(Debug, Clone, Copy)]
pub struct RigidBodyData {
    /// Mass in kilograms.
    pub mass: f32,

    /// Linear velocity in m/s (world frame).
    pub velocity: Vec3,

    /// Angular velocity in rad/s (world frame, right-hand rule).
    pub angular_velocity: Vec3,

    /// Accumulated force in Newtons for this tick (cleared each tick).
    pub force_accumulator: Vec3,

    /// Accumulated torque in N⋅m for this tick (cleared each tick).
    pub torque_accumulator: Vec3,

    /// Moment of inertia tensor (3x3 matrix, stored as 9 components in row-major order).
    ///
    /// For now, we store the diagonal elements only; off-diagonal elements are zero.
    /// Format: [`I_xx`, `I_xy`, `I_xz`, `I_yx`, `I_yy`, `I_yz`, `I_zx`, `I_zy`, `I_zz`]
    pub inertia_tensor: [f32; 9],
}

impl RigidBodyData {
    /// Creates a new rigid body data with the given mass and zero velocity/forces.
    ///
    /// Inertia is computed as a sphere with the given mass and a default radius
    /// of 1 meter, then scaled by the provided `inertia_scale` factor.
    ///
    /// # Arguments
    ///
    /// * `mass` - Mass in kilograms (must be positive and finite).
    /// * `inertia_scale` - Multiplier on computed inertia (default 1.0 for sphere).
    ///
    /// # Panics
    ///
    /// Panics if `mass` is not positive and finite, per ADR-0009 (every rigid
    /// body MUST have mass) and ADR-0013 (no silent fallbacks).
    #[must_use]
    pub fn new(mass: f32, inertia_scale: f32) -> Self {
        assert!(
            mass > 0.0 && mass.is_finite(),
            "RigidBodyData mass must be positive and finite, got {mass} (ADR-0009)",
        );
        // Inertia of a sphere: I = (2/5) * m * r^2
        // With r = 1 m: I = (2/5) * m = 0.4 * m
        let sphere_inertia = 0.4 * mass * inertia_scale;

        let mut inertia_tensor = [0.0; 9];
        inertia_tensor[0] = sphere_inertia; // I_xx
        inertia_tensor[4] = sphere_inertia; // I_yy
        inertia_tensor[8] = sphere_inertia; // I_zz

        Self {
            mass,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            inertia_tensor,
        }
    }

    /// Applies a force to this body (accumulates for the current tick).
    pub fn apply_force(&mut self, force: Vec3) {
        self.force_accumulator += force;
    }

    /// Applies a torque to this body (accumulates for the current tick).
    pub fn apply_torque(&mut self, torque: Vec3) {
        self.torque_accumulator += torque;
    }

    /// Applies an impulse to this body (instantaneous velocity change).
    ///
    /// Impulse is force applied over an infinitesimal time interval.
    /// The velocity change is: `delta_v = impulse / mass`.
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        self.velocity += impulse / self.mass;
    }

    /// Clears accumulated forces and torques (called after integration each tick).
    #[allow(clippy::missing_const_for_fn)]
    pub fn clear_accumulators(&mut self) {
        self.force_accumulator = Vec3::ZERO;
        self.torque_accumulator = Vec3::ZERO;
    }

    /// Integrates velocity from accumulated forces using F = ma.
    pub fn integrate_velocity(&mut self, delta_time: f32) {
        let acceleration = self.force_accumulator / self.mass;
        self.velocity += acceleration * delta_time;
    }

    /// Integrates angular velocity from accumulated torques using tau = I * alpha.
    pub fn integrate_angular_velocity(&mut self, delta_time: f32) {
        // For a diagonal inertia tensor, alpha = tau / I_diagonal
        let alpha_x = self.torque_accumulator.x / self.inertia_tensor[0];
        let alpha_y = self.torque_accumulator.y / self.inertia_tensor[4];
        let alpha_z = self.torque_accumulator.z / self.inertia_tensor[8];

        self.angular_velocity += Vec3::new(alpha_x, alpha_y, alpha_z) * delta_time;
    }
}

/// Resolves the final mass value, using override if present.
///
/// Mass is NOT scaled - it is used as-is from the template, or overridden if
/// `mass_override` is specified.
#[must_use]
pub fn resolve_mass(template_mass: f32, mass_override: Option<f32>) -> f32 {
    mass_override.unwrap_or(template_mass)
}
