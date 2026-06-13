// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Rigid body component for Newtonian physics simulation.
//!
//! The [`RigidBody`] component wraps avian3d's physics body and adds
//! our custom mass and inertia tracking for deterministic gravity
//! calculations (ADR-0009) and fixed-timestep integration (ADR-0017).

use bevy::prelude::*;

/// A rigid body in the physics simulation.
///
/// This component stores the physical properties needed for Newtonian dynamics:
/// mass, linear velocity, angular velocity, and accumulated forces/torques
/// for the current tick.
///
/// The component is updated each fixed timestep by the physics integration
/// systems (see [`crate::systems`]).
///
/// See ADR-0009 (Newtonian physics with gravity) and
/// ADR-0017 (Fixed timestep and determinism).
#[derive(Component, Clone, Copy, Debug)]
pub struct RigidBody {
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

impl RigidBody {
    /// Creates a new rigid body with the given mass and zero velocity/forces.
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
    pub fn new(mass: f32, inertia_scale: f32) -> Self {
        assert!(
            mass > 0.0 && mass.is_finite(),
            "RigidBody mass must be positive and finite, got {mass} (ADR-0009)",
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

/// Marker component for entities that generate a gravitational field.
///
/// Per ADR-0009, bodies designated as gravity sources generate
/// `g = G * M / r²` on all bodies within [`crate::constants::GRAVITY_CUTOFF_RADIUS_M`].
/// The mass value is taken from the entity's [`RigidBody::mass`] field.
///
/// For M2 this is a placeholder — no suns or planets exist yet (M3).
/// The gravity system is active but will have no effect until entities
/// with `MassSource` are spawned.
#[derive(Component, Clone, Copy, Debug)]
pub struct MassSource;

#[cfg(test)]
#[path = "rigid_body_tests.rs"]
mod tests;
