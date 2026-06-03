// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Input → Forces pipeline: reads [`ActiveActions`] and produces thrust/torque
//! commands, applies them to the player ship's [`RigidBody`], and handles
//! flight-assist toggling and damping.
//!
//! All systems run in `FixedUpdate` during `InGame` (ADR-0017).
//!
//! Thrust and torque magnitudes come from [`ShipPropulsionConfig`], which is
//! populated from the ship template JSON at spawn time (ADR-0014).
//!
//! System execution order within `FixedUpdate`:
//! 1. Input reader (Translate) — samples keybindings → [`ActiveActions`] (from `delta-v-core`)
//! 2. Input reader (Ships) — [`ActiveActions`] → [`ThrustCommand`] + [`TorqueCommand`]
//! 3. Flight-assist toggle — [`LogicalAction::ToggleFlightAssist`] action → flip [`FlightAssistState`]
//! 4. Thrust system — [`ThrustCommand`] → `apply_force` on [`RigidBody`]
//! 5. Torque system — [`TorqueCommand`] → `apply_torque` on [`RigidBody`]
//! 6. Flight-assist damping — if enabled, damp velocity
//! 7. Clear commands — zero out [`ThrustCommand`] + [`TorqueCommand`]

use bevy::prelude::*;
use delta_v_core::{
    ActiveActions, FlightAssist, FlightAssistConfig, FlightAssistState, LogicalAction,
    PlayerShipEntity, ShipPropulsionConfig, ThrustCommand, TorqueCommand,
};
use delta_v_physics::RigidBody;

/// System set for the input → forces pipeline within `FixedUpdate`.
///
/// These sets are configured to run after [`delta_v_core::InputSet::Translate`] (which
/// populates [`ActiveActions`]) and before the physics integration sets.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShipInputSet {
    /// Read [`ActiveActions`] and accumulate [`ThrustCommand`] + [`TorqueCommand`].
    AccumulateCommands,
    /// Toggle flight assist on/off based on [`LogicalAction::ToggleFlightAssist`].
    ToggleFlightAssist,
    /// Apply accumulated [`ThrustCommand`] as forces on the player ship.
    ApplyThrust,
    /// Apply accumulated [`TorqueCommand`] as torques on the player ship.
    ApplyTorque,
    /// Apply flight-assist velocity damping if enabled.
    FlightAssistDamping,
    /// Clear [`ThrustCommand`] and [`TorqueCommand`] after application.
    ClearCommands,
}

/// Reads [`ActiveActions`] and accumulates thrust/torque commands.
///
/// Runs in `FixedUpdate` after [`delta_v_core::InputSet::Translate`].
/// For each active action, applies the corresponding force or torque
/// direction. The magnitude is read from [`ShipPropulsionConfig`], which
/// comes from the ship template JSON (ADR-0014).
#[allow(clippy::needless_pass_by_value)]
pub fn input_reader_system(
    active: Res<'_, ActiveActions>,
    propulsion: Res<'_, ShipPropulsionConfig>,
    mut thrust_cmd: ResMut<'_, ThrustCommand>,
    mut torque_cmd: ResMut<'_, TorqueCommand>,
) {
    for action in &active.0 {
        match action {
            // Thrust: apply force in local frame
            // Forward = -Z, Backward = +Z (ADR-0006)
            LogicalAction::ThrustForward => {
                thrust_cmd.force.z -= propulsion.max_forward_thrust;
            }
            LogicalAction::ThrustBackward => {
                thrust_cmd.force.z += propulsion.max_backward_thrust;
            }
            // Strafe: apply force along X/Y axes
            LogicalAction::StrafeLeft => {
                thrust_cmd.force.x -= propulsion.max_strafe_thrust;
            }
            LogicalAction::StrafeRight => {
                thrust_cmd.force.x += propulsion.max_strafe_thrust;
            }
            LogicalAction::StrafeUp => {
                thrust_cmd.force.y += propulsion.max_strafe_thrust;
            }
            LogicalAction::StrafeDown => {
                thrust_cmd.force.y -= propulsion.max_strafe_thrust;
            }
            // Rotation: apply torque around local axes
            // Pitch around X: nose up = +X torque, nose down = -X torque
            LogicalAction::PitchUp => {
                torque_cmd.torque.x += propulsion.max_torque;
            }
            LogicalAction::PitchDown => {
                torque_cmd.torque.x -= propulsion.max_torque;
            }
            // Yaw around Y: left = +Y, right = -Y
            LogicalAction::YawLeft => {
                torque_cmd.torque.y += propulsion.max_torque;
            }
            LogicalAction::YawRight => {
                torque_cmd.torque.y -= propulsion.max_torque;
            }
            // Roll around Z: CCW = +Z, CW = -Z
            LogicalAction::RollLeft => {
                torque_cmd.torque.z += propulsion.max_torque;
            }
            LogicalAction::RollRight => {
                torque_cmd.torque.z -= propulsion.max_torque;
            }
            LogicalAction::ToggleFlightAssist => {
                // Handled by flight_assist_toggle_system
            }
        }
    }
}

/// Toggles flight assist on/off when the player presses the toggle key.
///
/// Runs in `FixedUpdate` after [`delta_v_core::InputSet::Translate`].
/// Only toggles on the frame the action transitions from not-active to active
/// (i.e. on press, not on hold).
#[allow(clippy::needless_pass_by_value)]
pub fn flight_assist_toggle_system(
    active: Res<'_, ActiveActions>,
    mut state: ResMut<'_, FlightAssistState>,
) {
    if active.0.contains(&LogicalAction::ToggleFlightAssist) {
        state.enabled = !state.enabled;
        log::info!("flight assist toggled: {}", state.enabled);
    }
}

/// Applies accumulated thrust as a force on the player ship's [`RigidBody`].
///
/// The force is in the ship's local frame and must be rotated into world
/// space before application. Runs in `FixedUpdate`.
#[allow(clippy::needless_pass_by_value)]
pub fn thrust_system(
    thrust_cmd: Res<'_, ThrustCommand>,
    ship_entity: Res<'_, PlayerShipEntity>,
    mut bodies: Query<'_, '_, (&mut RigidBody, &Transform)>,
) {
    if thrust_cmd.force == Vec3::ZERO {
        return;
    }

    let Ok((mut body, transform)) = bodies.get_mut(ship_entity.0) else {
        return;
    };

    // Rotate force from local frame to world frame
    let world_force = transform.rotation * thrust_cmd.force;
    body.apply_force(world_force);
}

/// Applies accumulated torque on the player ship's [`RigidBody`].
///
/// The torque is in the ship's local frame and must be rotated into world
/// space before application. Runs in `FixedUpdate`.
#[allow(clippy::needless_pass_by_value)]
pub fn torque_system(
    torque_cmd: Res<'_, TorqueCommand>,
    ship_entity: Res<'_, PlayerShipEntity>,
    mut bodies: Query<'_, '_, (&mut RigidBody, &Transform)>,
) {
    if torque_cmd.torque == Vec3::ZERO {
        return;
    }

    let Ok((mut body, transform)) = bodies.get_mut(ship_entity.0) else {
        return;
    };

    // Rotate torque from local frame to world frame
    let world_torque = transform.rotation * torque_cmd.torque;
    body.apply_torque(world_torque);
}

/// Applies flight-assist velocity damping when enabled.
///
/// When flight assist is active, this system applies a damping force
/// proportional to the ship's current velocity, gently decelerating it.
/// The damping coefficient controls how aggressively velocity is reduced.
///
/// This is NOT an instantaneous velocity kill — it applies soft deceleration
/// each tick, giving a smooth "inertial damping" feel.
///
/// Runs in `FixedUpdate`.
#[allow(clippy::needless_pass_by_value)]
pub fn flight_assist_damping_system(
    state: Res<'_, FlightAssistState>,
    config: Res<'_, FlightAssistConfig>,
    ship_entity: Res<'_, PlayerShipEntity>,
    mut bodies: Query<'_, '_, &mut RigidBody, With<FlightAssist>>,
) {
    if !state.enabled {
        return;
    }

    let Ok(mut body) = bodies.get_mut(ship_entity.0) else {
        return;
    };

    // Apply damping: F_damp = -coeff * v
    // This gives exponential decay of velocity over time.
    let damping_force = -config.damping_coefficient * body.velocity;
    body.apply_force(damping_force);

    // Also damp angular velocity
    let damping_torque = -config.damping_coefficient * body.angular_velocity;
    body.apply_torque(damping_torque);
}

/// Clears thrust and torque commands after they have been applied.
///
/// Runs last in the pipeline to prepare for the next tick.
pub fn clear_commands_system(
    mut thrust_cmd: ResMut<'_, ThrustCommand>,
    mut torque_cmd: ResMut<'_, TorqueCommand>,
) {
    thrust_cmd.force = Vec3::ZERO;
    torque_cmd.torque = Vec3::ZERO;
}
