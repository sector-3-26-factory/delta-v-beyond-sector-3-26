// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! AI-driven NPC ship spawning.

use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_core::{DebugAxesEligible, Health, SpawnEntity, Weapon};
use delta_v_physics::{CollisionShape, RigidBody};
use delta_v_spawn::collision::shape_from_json;
use delta_v_types::WeaponTemplateJson;

use crate::components::{AiConfig, AiState, AiTask, NpcShip};
use crate::resources::SkirmishState;

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
/// Panics if the template JSON is missing required fields or if collision shape
/// conversion fails. This is intentional per ADR-0013 (no silent fallbacks).
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
    mut events: EventReader<'_, '_, SpawnEntity>,
    mut skirmish_state: ResMut<'_, SkirmishState>,
) {
    for event in events.read() {
        if event.entity_type != "ai_controlled_ship" {
            continue;
        }

        // Track total enemies for skirmish detection
        skirmish_state.total_enemies += 1;

        let template = &event.template;

        // INVARIANT: mass.value is required by ai_controlled_ship schema (ADR-0013)
        let mass = template
            .get("mass")
            .and_then(|m| m.get("value"))
            .and_then(|v| v.as_f64())
            .expect("ai_controlled_ship template must have mass.value") as f32;

        // INVARIANT: inertia_scale is required by ai_controlled_ship schema (ADR-0013)
        let inertia_scale = template
            .get("inertia_scale")
            .and_then(|v| v.as_f64())
            .expect("ai_controlled_ship template must have inertia_scale (ADR-0014)")
            as f32;

        // INVARIANT: health.value is required by ai_controlled_ship schema (ADR-0013)
        // Read health from template (ADR-0014: gameplay values from JSON)
        let health_value = template
            .get("health")
            .and_then(|h| h.get("value"))
            .and_then(|v| v.as_f64())
            .expect("ai_controlled_ship template must have health.value")
            as f32;

        // INVARIANT: collision_shape is required by ai_controlled_ship schema (ADR-0013)
        let collision_shape = template
            .get("collision_shape")
            .expect("ai_controlled_ship template must have collision_shape");

        let scale = event.scale.x.max(event.scale.y).max(event.scale.z);

        // INVARIANT: collision_shape is validated by delta-v-json (ADR-0013)
        let collision_shape_data = shape_from_json(
            &serde_json::from_value(collision_shape.clone())
                .expect("collision_shape must be valid JSON"),
            scale,
        )
        .expect("collision shape must be valid (ADR-0013)");

        // INVARIANT: bounding_box is required by ai_controlled_ship schema (ADR-0013)
        let bbox = template
            .get("bounding_box")
            .expect("ai_controlled_ship template must have bounding_box");
        let half_extent = Vec3::new(
            (bbox
                .get("max")
                .and_then(|m| m.get("x"))
                .and_then(|v| v.as_f64())
                .expect("bounding_box.max.x is required (ADR-0013)")
                - bbox
                    .get("min")
                    .and_then(|m| m.get("x"))
                    .and_then(|v| v.as_f64())
                    .expect("bounding_box.min.x is required (ADR-0013)")) as f32
                / 2.0,
            (bbox
                .get("max")
                .and_then(|m| m.get("y"))
                .and_then(|v| v.as_f64())
                .expect("bounding_box.max.y is required (ADR-0013)")
                - bbox
                    .get("min")
                    .and_then(|m| m.get("y"))
                    .and_then(|v| v.as_f64())
                    .expect("bounding_box.min.y is required (ADR-0013)")) as f32
                / 2.0,
            (bbox
                .get("max")
                .and_then(|m| m.get("z"))
                .and_then(|v| v.as_f64())
                .expect("bounding_box.max.z is required (ADR-0013)")
                - bbox
                    .get("min")
                    .and_then(|m| m.get("z"))
                    .and_then(|v| v.as_f64())
                    .expect("bounding_box.min.z is required (ADR-0013)")) as f32
                / 2.0,
        );
        let axis_length = half_extent.x.max(half_extent.y).max(half_extent.z) * 2.0 * scale;

        let mesh_path = event
            .mesh_template_path
            .replace("ship.json", "mesh.glb")
            .replace("ai_controlled_ship.json", "mesh.glb");

        let gltf_handle = asset_server.load::<Gltf>(&mesh_path);

        // INVARIANT: ai config is required by ai_controlled_ship schema (ADR-0013)
        let ai_config = template
            .get("ai")
            .expect("ai_controlled_ship template must have ai field");

        // INVARIANT: AI config fields are required by schema (ADR-0013)
        let aggro_range = ai_config
            .get("aggro_range")
            .and_then(|a| a.get("value"))
            .and_then(|v| v.as_f64())
            .expect("ai.aggro_range.value is required") as f32;
        let attack_range = ai_config
            .get("attack_range")
            .and_then(|a| a.get("value"))
            .and_then(|v| v.as_f64())
            .expect("ai.attack_range.value is required") as f32;
        let leash_range = ai_config
            .get("leash_range")
            .and_then(|a| a.get("value"))
            .and_then(|v| v.as_f64())
            .expect("ai.leash_range.value is required") as f32;
        let flee_health_threshold = ai_config
            .get("flee_health_threshold")
            .and_then(|v| v.as_f64())
            .expect("ai.flee_health_threshold is required")
            as f32;
        let patrol_radius = ai_config
            .get("patrol_radius")
            .and_then(|a| a.get("value"))
            .and_then(|v| v.as_f64())
            .expect("ai.patrol_radius.value is required") as f32;

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
                RigidBody::new(mass, inertia_scale),
                CollisionShape(collision_shape_data),
                Health::new(health_value),
                NpcShip {
                    entity_id: event.id.clone(),
                },
                AiState::Patrol,
                AiConfig {
                    aggro_range,
                    attack_range,
                    leash_range,
                    flee_health_threshold,
                    patrol_radius,
                },
                ai_task,
            ))
            .id();

        if let Some(weapons) = template.get("weapons").and_then(|w| w.as_array()) {
            for (i, weapon_json) in weapons.iter().enumerate() {
                // INVARIANT: weapon JSON is validated by delta-v-json (ADR-0013)
                let weapon: WeaponTemplateJson = serde_json::from_value(weapon_json.clone())
                    .expect("weapon must be valid JSON (validated by delta-v-json)");
                commands.entity(ship_entity).insert(Weapon {
                    // INVARIANT: weapon slot fits in u32 (ADR-0013)
                    slot: u32::try_from(i).expect("weapon slot overflow"),
                    cooldown: 0.0,
                    projectile_speed: weapon.projectile_speed.value,
                    damage: weapon.damage.value,
                    fire_rate: weapon.fire_rate.value,
                    lifetime: weapon.lifetime.value,
                    projectile_radius: weapon.projectile_radius.value,
                });
            }
        }

        log::info!(
            "AI ship '{}' spawned at ({:.1}, {:.1}, {:.1}) from {} (mass={}kg, task={:?})",
            event.id,
            event.position.x,
            event.position.y,
            event.position.z,
            mesh_path,
            mass,
            ai_task,
        );
    }
}
