// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! AI-driven NPC ship spawning.
//!
//! Per ADR-0038 (entity template system), AI-controlled ships are spawned from
//! templates loaded and validated by delta-v-json. The template `Value` is
//! deserialized into [`AiControlledShipTemplate`] via `serde_json::from_value`
//! (step 4 of the ADR-0040 JSON pipeline: read, validate, fill defaults,
//! deserialize). Steps 1-3 are performed upstream by `delta-v-assets`.
//!
//! Per ADR-0014, all gameplay values (mass, health, AI ranges) come from JSON
//! — never from Rust constants. The template is validated by delta-v-json before
//! deserialization, so missing required fields are hard errors via serde.
//!
//! See also ADR-0005 (plugin architecture) and ADR-0017 (Fixed timestep).

use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_core::{DebugAxesEligible, Health, SpawnEntity, Weapon};
use delta_v_physics::{CollisionShape, RigidBody};
use delta_v_spawn::collision::shape_from_json;
use delta_v_spawn::template_extraction::compute_debug_axis_length;
use delta_v_types::{
    AiConfigJson, BoundingBoxJson, CollisionShapeJson, PhysicalQuantityJson, WeaponTemplateJson,
};
use serde::Deserialize;

use crate::components::{AiConfig, AiState, AiTask, NpcShip};
use crate::resources::SkirmishState;

/// Deserialized AI-controlled ship template JSON.
///
/// Contains all fields needed to spawn an AI-controlled NPC ship: physical
/// properties (mass, inertia, health), collision shape, bounding box, weapons,
/// and AI behavioral configuration. This struct is produced by deserializing
/// the validated + default-filled `serde_json::Value` from the template file.
/// Per ADR-0040, the schema is the only source of defaults — no
/// `#[serde(default)]` or `impl Default`.
#[derive(Debug, Deserialize)]
pub struct AiControlledShipTemplate {
    /// Ship mass in kilograms.
    pub mass: PhysicalQuantityJson,
    /// Dimensionless inertia multiplier (default 1.0 from schema).
    pub inertia_scale: f32,
    /// Ship health in hit points (default 100.0 from schema).
    pub health: PhysicalQuantityJson,
    /// Collision shape for the ship.
    pub collision_shape: CollisionShapeJson,
    /// Axis-aligned bounding box in ship-local coordinates (metres).
    pub bounding_box: BoundingBoxJson,
    /// Weapon configurations. Defaults to `[]` via schema.
    pub weapons: Vec<WeaponTemplateJson>,
    /// AI behavioral parameters (aggro, attack, leash, patrol ranges).
    pub ai: AiConfigJson,
}

/// Marker component for a pending NPC ship mesh waiting for its glTF to load.
#[derive(Component)]
pub struct PendingNpcShipMesh {
    /// Handle to the glTF asset being loaded.
    gltf_handle: Handle<Gltf>,
}

impl delta_v_spawn::mesh_attachment::PendingMesh for PendingNpcShipMesh {
    fn gltf_handle(&self) -> &Handle<Gltf> {
        &self.gltf_handle
    }
}

/// Deserializes the template JSON into an [`AiControlledShipTemplate`] struct.
///
/// This is step 4 of the ADR-0040 JSON pipeline (deserialization).
/// Steps 1-3 (read, validate against schema, fill defaults) are performed
/// upstream by `delta-v-assets` via `delta-v-json` before the `SpawnEntity`
/// event is emitted. At this point the `Value` is guaranteed to be valid
/// and fully default-filled, so `serde_json::from_value` is a one-liner.
#[allow(clippy::expect_used)] // INVARIANT: template validated + default-filled by delta-v-json upstream; cannot fail
fn deserialize_template(event: &SpawnEntity) -> AiControlledShipTemplate {
    serde_json::from_value(event.template.clone())
        .expect("template deserialization must succeed (validated by delta-v-json, ADR-0040)")
}

/// Spawns AI-driven NPC ship entities in response to `SpawnEntity` events.
///
/// Listens for events with `entity_type == "ai_controlled_ship"` and spawns
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
        if event.entity_type != "ai_controlled_ship" {
            continue;
        }

        // Track total enemies for skirmish detection
        skirmish_state.total_enemies += 1;

        // Deserialize template JSON into typed struct (ADR-0040 one-liner).
        let template = deserialize_template(event);

        let scale = event.scale.x.max(event.scale.y).max(event.scale.z);

        // INVARIANT: collision_shape is validated by delta-v-json (ADR-0013)
        let collision_shape_data = shape_from_json(&template.collision_shape, scale)
            .expect("collision shape must be valid (ADR-0013)");

        // Compute debug axes length from the bounding box in the template JSON, scaled.
        let axis_length = compute_debug_axis_length(&template.bounding_box) * scale;

        let mesh_path = event
            .mesh_template_path
            .replace("ship.json", "mesh.glb")
            .replace("ai_controlled_ship.json", "mesh.glb");

        let gltf_handle = asset_server.load::<Gltf>(&mesh_path);

        // INVARIANT: ai_task is validated by schema (ADR-0013)
        let ai_task = match event.ai_task.as_deref() {
            Some("patrol") => AiTask::Patrol,
            Some(other) => panic!("Unknown AI task '{other}' for entity '{}'", event.id),
            None => panic!("AI-controlled ship '{}' has no task assigned", event.id),
        };

        let ship_entity = commands
            .spawn((
                Transform {
                    translation: event.position,
                    rotation: event.rotation,
                    scale: event.scale,
                },
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                PendingNpcShipMesh { gltf_handle },
                DebugAxesEligible::new(event.id.clone(), axis_length),
                RigidBody::new(template.mass.value, template.inertia_scale),
                CollisionShape(collision_shape_data),
                Health::new(template.health.value),
                NpcShip {
                    entity_id: event.id.clone(),
                },
                AiState::Patrol,
                AiConfig {
                    aggro_range: template.ai.aggro_range.value,
                    attack_range: template.ai.attack_range.value,
                    leash_range: template.ai.leash_range.value,
                    flee_health_threshold: template.ai.flee_health_threshold as f32,
                    patrol_radius: template.ai.patrol_radius.value,
                },
                ai_task,
            ))
            .id();

        // Add Weapon components from template (M4).
        // Per ADR-0014, all gameplay values come from JSON.
        for (i, weapon_json) in template.weapons.iter().enumerate() {
            commands.entity(ship_entity).insert(Weapon {
                // INVARIANT: weapon slot fits in u32 (ADR-0013)
                slot: u32::try_from(i).expect("weapon slot overflow"),
                cooldown: 0.0,
                projectile_speed: weapon_json.projectile_speed.value,
                damage: weapon_json.damage.value,
                fire_rate: weapon_json.fire_rate.value,
                lifetime: weapon_json.lifetime.value,
                projectile_radius: weapon_json.projectile_radius.value,
                sound: weapon_json.sound.clone(),
            });
        }

        tracing::info!(
            "AI ship '{}' spawned at ({:.1}, {:.1}, {:.1}) from {} (mass={}kg, task={:?})",
            event.id,
            event.position.x,
            event.position.y,
            event.position.z,
            mesh_path,
            template.mass.value,
            ai_task,
        );
    }
}
