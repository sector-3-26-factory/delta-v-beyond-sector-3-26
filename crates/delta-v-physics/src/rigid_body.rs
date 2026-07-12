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
// MERCHANTABILITY OR FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Rigid body component for Newtonian physics simulation.
//!
//! The [`RigidBody`] component wraps the plain data type [`delta_v_types::RigidBodyData`]
//! and adds the Bevy `Component` derive. See ADR-0046 for the data/runtime type
//! separation pattern.

use bevy::prelude::*;
use delta_v_types::RigidBodyData;
use std::ops::Deref;

/// A rigid body in the physics simulation.
///
/// This component wraps [`RigidBodyData`] (from `delta-v-types`) and adds
/// the Bevy `Component` derive. Implements [`Deref`] for transparent
/// access to the inner data — code using `RigidBody` can access
/// `mass`, `velocity`, `angular_velocity`, etc. directly.
///
/// The component is updated each fixed timestep by the physics integration
/// systems (see [`crate::systems`]).
///
/// See ADR-0009 (Newtonian physics with gravity) and
/// ADR-0017 (Fixed timestep and determinism).
#[derive(Component, Clone, Copy, Debug)]
pub struct RigidBody(pub RigidBodyData);

impl Deref for RigidBody {
    type Target = RigidBodyData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RigidBody {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
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
        Self(RigidBodyData::new(mass, inertia_scale))
    }

    /// Applies a force to this body (accumulates for the current tick).
    pub fn apply_force(&mut self, force: Vec3) {
        self.0.apply_force(force);
    }

    /// Applies a torque to this body (accumulates for the current tick).
    pub fn apply_torque(&mut self, torque: Vec3) {
        self.0.apply_torque(torque);
    }

    /// Applies an impulse to this body (instantaneous velocity change).
    ///
    /// Impulse is force applied over an infinitesimal time interval.
    /// The velocity change is: `delta_v = impulse / mass`.
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        self.0.apply_impulse(impulse);
    }

    /// Clears accumulated forces and torques (called after integration each tick).
    #[allow(clippy::missing_const_for_fn)]
    pub fn clear_accumulators(&mut self) {
        self.0.clear_accumulators();
    }

    /// Integrates velocity from accumulated forces using F = ma.
    pub fn integrate_velocity(&mut self, delta_time: f32) {
        self.0.integrate_velocity(delta_time);
    }

    /// Integrates angular velocity from accumulated torques using tau = I * alpha.
    pub fn integrate_angular_velocity(&mut self, delta_time: f32) {
        self.0.integrate_angular_velocity(delta_time);
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
