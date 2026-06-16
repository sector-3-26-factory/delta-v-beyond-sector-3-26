// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! AI state machine and skirmish tracking systems.

use bevy::prelude::*;
use delta_v_core::{FireWeapon, Health, PlayerShipEntity};
use delta_v_physics::RigidBody;

use crate::components::{AiConfig, AiState, AiTask, NpcShip};
use crate::resources::SkirmishState;

/// System set for AI systems running in `FixedUpdate`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AiSet {
    /// Run AI state machines to determine behavior.
    StateMachine,
    /// Handle AI weapon firing.
    FireWeapons,
    /// Check skirmish win/lose conditions.
    SkirmishCheck,
}

/// AI state machine system for NPC behavior.
///
/// Runs in `FixedUpdate` during `InGame`. For each AI-controlled entity,
/// evaluates the current state and transitions based on distance to player,
/// health, and the assigned [`AiTask`].
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::type_complexity
)]
pub fn ai_state_machine_system(
    player_entity: Res<'_, PlayerShipEntity>,
    mut npc_query: Query<
        '_,
        '_,
        (
            Entity,
            &mut AiState,
            &AiConfig,
            &AiTask,
            &Transform,
            &mut RigidBody,
            &Health,
        ),
        With<NpcShip>,
    >,
    player_query: Query<'_, '_, &Transform>,
    mut fire_events: MessageWriter<'_, FireWeapon>,
) {
    let Ok(player_transform) = player_query.get(player_entity.0) else {
        return;
    };
    let player_pos = player_transform.translation;

    for (entity, mut ai_state, ai_config, ai_task, transform, mut body, health) in &mut npc_query {
        let self_pos = transform.translation;
        let distance = (player_pos - self_pos).length();
        let health_fraction = health.current / health.max;

        match ai_task {
            AiTask::Patrol => match *ai_state {
                AiState::Patrol => {
                    if distance < ai_config.aggro_range {
                        *ai_state = AiState::Pursue;
                    }
                }
                AiState::Pursue => {
                    if distance < ai_config.attack_range {
                        *ai_state = AiState::Attack;
                    } else if distance > ai_config.leash_range {
                        *ai_state = AiState::Patrol;
                    } else {
                        let forward = transform.rotation * Vec3::NEG_Z;
                        body.apply_force(forward * 50_000.0);
                        let to_player = (player_pos - self_pos).normalize();
                        let current_forward = transform.rotation * Vec3::NEG_Z;
                        let rotation_axis = current_forward.cross(to_player);
                        if rotation_axis.length_squared() > f32::EPSILON {
                            body.apply_torque(
                                rotation_axis.normalize() * rotation_axis.length() * 50_000.0,
                            );
                        }
                    }
                }
                AiState::Attack => {
                    if health_fraction < ai_config.flee_health_threshold {
                        *ai_state = AiState::Flee;
                    } else if distance > ai_config.attack_range {
                        *ai_state = AiState::Pursue;
                    } else {
                        let to_player = (player_pos - self_pos).normalize();
                        let current_forward = transform.rotation * Vec3::NEG_Z;
                        let rotation_axis = current_forward.cross(to_player);
                        if rotation_axis.length_squared() > f32::EPSILON {
                            body.apply_torque(
                                rotation_axis.normalize() * rotation_axis.length() * 50_000.0,
                            );
                        }
                        if current_forward.dot(to_player) > 0.9 {
                            fire_events.write(FireWeapon {
                                source: entity,
                                weapon_index: 0,
                            });
                        }
                    }
                }
                AiState::Flee => {
                    if distance > ai_config.leash_range
                        && health_fraction > ai_config.flee_health_threshold
                    {
                        *ai_state = AiState::Patrol;
                    } else {
                        let forward = transform.rotation * Vec3::NEG_Z;
                        body.apply_force(forward * 50_000.0);
                    }
                }
            },
        }
    }
}

/// Checks skirmish win/lose conditions.
///
/// Only transitions to [`AppState::SkirmishOver`] if this is actually a skirmish
/// world (i.e., `total_enemies > 0`). Worlds with no enemies skip the skirmish
/// entirely and remain in [`AppState::InGame`].
#[allow(clippy::needless_pass_by_value)]
pub fn skirmish_check_system(
    mut skirmish_state: ResMut<'_, SkirmishState>,
    npc_query: Query<'_, '_, &Health, With<NpcShip>>,
    player_query: Res<'_, PlayerShipEntity>,
    player_health: Query<'_, '_, &Health>,
    mut next_state: ResMut<'_, NextState<delta_v_core::AppState>>,
) {
    skirmish_state.enemies_alive = npc_query.iter().filter(|h| !h.is_destroyed()).count();

    skirmish_state.player_alive = player_health
        .get(player_query.0)
        .map(|h| !h.is_destroyed())
        .unwrap_or(false);

    if !skirmish_state.player_alive {
        log::info!("Skirmish lost -- player destroyed");
        next_state.set(delta_v_core::AppState::SkirmishOver);
    } else if skirmish_state.total_enemies > 0 && skirmish_state.enemies_alive == 0 {
        log::info!("Skirmish won -- all enemies destroyed");
        next_state.set(delta_v_core::AppState::SkirmishOver);
    }
}
