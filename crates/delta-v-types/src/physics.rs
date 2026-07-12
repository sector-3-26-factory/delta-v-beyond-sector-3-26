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
