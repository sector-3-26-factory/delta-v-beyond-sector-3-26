// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! AI-driven NPC ship spawning.
//!
//! Per ADR-0038 (entity template system), AI-controlled ships are spawned from
//! templates loaded and validated by delta-v-assets. The template is now an
//! [`EntityTemplate`] enum with runtime types.
//!
//! Per ADR-0014, all gameplay values (mass, health, AI ranges) come from JSON
//! — never from Rust constants. The template is validated by delta-v-json before
//! conversion, so missing required fields are hard errors via serde.
//!
//! See also ADR-0005 (plugin architecture) and ADR-0017 (Fixed timestep).

use bevy::prelude::*;
use delta_v_core::SpawnEntity;
use delta_v_spawn::build_physical_ship;

use crate::components::{AiConfig, AiState, AiTask, NpcShip};
use crate::resources::SkirmishState;

/// Spawns AI-driven NPC ship entities in response to `SpawnEntity` events.
///
/// Listens for events with `template` being `EntityTemplate::AiShip` and spawns
/// the ship with AI components ([`AiState`], [`AiConfig`], [`AiTask`], [`NpcShip`]).
///
/// Increments [`SkirmishState::total_enemies`] so the skirmish check system
/// can distinguish skirmish worlds (with enemies) from non-skirmish worlds.
///
/// Runs during [`AppState::SpawningEntities`] in
/// [`delta_v_core::WorldSpawnSet::SpawnNpcs`].
///
/// # Panics
///
/// Panics if the template JSON fails to deserialize (missing required fields
/// or invalid types). This is intentional per ADR-0013 (no silent fallbacks).
/// The template is validated by delta-v-json before deserialization, so this
/// should never happen in practice.
#[allow(
    clippy::needless_pass_by_value,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::redundant_closure,
    clippy::redundant_closure_for_method_calls,
    clippy::panic
)]
pub fn spawn_npc_ship(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    mut events: MessageReader<'_, '_, SpawnEntity>,
    mut skirmish_state: ResMut<'_, SkirmishState>,
) {
    for event in events.read() {
        // Extract AI ship template from EntityTemplate enum.
        // The template was already loaded and converted by delta-v-assets (ADR-0040).
        let template = match &event.template {
            delta_v_types::EntityTemplate::AiShip(t) => t.clone(),
            _ => continue,
        };

        // Track total enemies for skirmish detection
        skirmish_state.total_enemies += 1;

        // Build the physical ship (common components: physics, collision, health, weapons).
        let (ship_entity, _propulsion_config) =
            build_physical_ship(&mut commands, &asset_server, event, &template);

        // INVARIANT: ai_task is validated by schema (ADR-0013)
        let ai_task = match event.ai_task.as_deref() {
            Some("patrol") => AiTask::Patrol,
            Some(other) => panic!("Unknown AI task '{other}' for entity '{}'", event.id),
            None => panic!("AI-controlled ship '{}' has no task assigned", event.id),
        };

        // Add AI-specific components
        commands.entity(ship_entity).insert((
            NpcShip {
                entity_id: event.id.clone(),
            },
            AiState::Patrol,
            AiConfig {
                aggro_range: template.ai.aggro_range,
                attack_range: template.ai.attack_range,
                leash_range: template.ai.leash_range,
                flee_health_threshold: template.ai.flee_health_threshold,
                patrol_radius: template.ai.patrol_radius,
            },
            ai_task,
        ));

        tracing::info!(
            "AI ship '{}' spawned at ({:.1}, {:.1}, {:.1}) from {} (mass={}kg, task={:?})",
            event.id,
            event.position.x,
            event.position.y,
            event.position.z,
            event.template_path,
            template.mass,
            ai_task,
        );
    }
}
