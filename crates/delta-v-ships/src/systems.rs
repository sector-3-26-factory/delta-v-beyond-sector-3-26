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

use std::collections::BTreeSet;

use bevy::prelude::*;
use delta_v_core::{
    ActiveActions, FlightAssist, FlightAssistConfig, FlightAssistState, LogicalAction,
    PlayerShipEntity,
};
use delta_v_physics::RigidBody;

use crate::ship_templates::{ShipPropulsionConfig, ThrustCommand, TorqueCommand};

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

/// Tracks the previous tick's [`ActiveActions`] for edge detection.
///
/// Used by [`flight_assist_toggle_system`] to detect key press transitions
/// (on press, not on hold).
// allow-default: Bevy requires Default on resources for init_resource.
// This is per-tick state, not configuration.
#[derive(Resource, Default, Debug)]
pub struct PreviousActions(pub BTreeSet<LogicalAction>);

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
            LogicalAction::ToggleFlightAssist | LogicalAction::FirePrimary => {
                // Handled by other systems (flight_assist_toggle_system / weapons plugin)
            }
        }
    }
}

/// Toggles flight assist on/off when the player presses the toggle key.
///
/// Runs in `FixedUpdate` after [`delta_v_core::InputSet::Translate`].
/// Only toggles on the frame the action transitions from not-active to active
/// (i.e. on press, not on hold). Uses [`PreviousActions`] for edge detection.
#[allow(clippy::needless_pass_by_value)]
pub fn flight_assist_toggle_system(
    active: Res<'_, ActiveActions>,
    mut prev: ResMut<'_, PreviousActions>,
    mut state: ResMut<'_, FlightAssistState>,
) {
    let toggle = LogicalAction::ToggleFlightAssist;
    let is_pressed = active.0.contains(&toggle);
    let was_pressed = prev.0.contains(&toggle);

    // Edge detection: only toggle on the transition from not-pressed to pressed.
    if is_pressed && !was_pressed {
        state.enabled = !state.enabled;
        log::info!("flight assist toggled: {}", state.enabled);
    }

    // Update previous state for next tick.
    prev.0.clone_from(&active.0);
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

/// Minimum angular velocity threshold below which rotation is fully stopped.
///
/// When flight assist damping reduces angular velocity below this value,
/// the velocity is zeroed out entirely to prevent endless micro-rotation.
const ANGULAR_VELOCITY_THRESHOLD: f32 = 0.001;

/// Applies flight-assist velocity damping when enabled.
///
/// When flight assist is active, this system reduces both linear and angular
/// velocity by the damping coefficient each tick. The coefficient is a 0-1
/// value where 0 = no damping and 1 = full stop in one tick.
///
/// Angular velocity is zeroed out entirely when it falls below
/// [`ANGULAR_VELOCITY_THRESHOLD`] to prevent endless micro-rotation.
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

    let one_minus_coeff = 1.0 - config.damping_coefficient;

    // Apply linear damping: reduce velocity by coefficient directly.
    body.velocity *= one_minus_coeff;

    // Apply angular damping the same way. When angular velocity is very
    // small, zero it out entirely to prevent endless micro-rotation.
    let ang_vel = body.angular_velocity;
    if ang_vel.length_squared() < ANGULAR_VELOCITY_THRESHOLD * ANGULAR_VELOCITY_THRESHOLD {
        body.angular_velocity = Vec3::ZERO;
    } else {
        body.angular_velocity *= one_minus_coeff;
    }
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
