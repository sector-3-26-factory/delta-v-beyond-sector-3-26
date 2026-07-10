// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Shared ship spawning utilities.
//!
//! Provides common functionality for spawning ship entities (player, NPC, static)
//! from templates. This avoids duplication across domain crates.
//!
//! See ADR-0047 (centralized spawning) and ADR-0051 (dependency hierarchy).

use crate::collision::shape_from_json;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_assets::template::{
    load_main_thruster_definition, load_maneuvering_thruster_definition,
    load_projectile_definition, load_weapon_definition,
};
use delta_v_core::{
    DebugAxesEligible, EntityType, FlightAssist, Health, Propulsion, SpawnEntity, Targetable,
    Weapon, WorldEntityId,
};
use delta_v_physics::{CollisionLayersComponent, CollisionShape, RigidBody};
use delta_v_types::{
    BoundingBoxJson, CollisionShapeJson, MainThrusterDefinitionJson,
    ManeuveringThrusterDefinitionJson, PhysicalQuantityJson, ProjectileDefinitionJson,
    PropulsionConfig, WeaponReference, WeaponTemplateJson,
};

/// Trait for accessing common ship template fields.
/// Implemented for ship template types across domain crates.
pub trait ShipTemplateBase {
    /// Returns the ship's mass in kilograms.
    fn mass(&self) -> &PhysicalQuantityJson;
    /// Returns the dimensionless inertia multiplier.
    fn inertia_scale(&self) -> f32;
    /// Returns the axis-aligned bounding box in ship-local coordinates (metres).
    fn bounding_box(&self) -> &BoundingBoxJson;
    /// Returns the collision shape for the ship.
    fn collision_shape(&self) -> &CollisionShapeJson;
    /// Returns the ship's health in hit points.
    fn health(&self) -> &PhysicalQuantityJson;
    /// Returns the weapon configurations.
    fn weapons(&self) -> &[WeaponReference];
    /// Returns the entity type string (e.g., "`player_controlled_ship`", "`ship`", "`ai_controlled_ship`").
    fn entity_type(&self) -> &str;
    /// Returns the array of main thruster names (references to directories under assets/main-thrusters/).
    fn main_thruster_names(&self) -> &[String];
    /// Returns the name of the maneuvering thruster (reference to a directory under assets/maneuvering-thrusters/).
    fn maneuvering_thruster_name(&self) -> &str;
}

/// Builds the base physical ship entity with common components.
///
/// This function creates the ship entity with physics, collision, health, weapons,
/// and propulsion components. It is called by all ship spawn functions (player, static, AI)
/// to avoid duplication.
///
/// # Arguments
/// * `commands` - Bevy commands for entity spawning
/// * `asset_server` - Asset server for loading glTF meshes
/// * `event` - `SpawnEntity` event containing position, rotation, scale, entity ID, and `mesh_template_path`
/// * `template` - Ship template implementing [`ShipTemplateBase`] with physical properties
///
/// # Returns
/// A tuple of (spawned ship entity ID, `PropulsionConfig` for resource insertion).
///
/// # Panics
/// Panics if the collision shape from the template is invalid. This should never happen
/// because templates are validated by delta-v-json before spawning (ADR-0013).
// INVARIANT: Indexing is safe (weapons iter), expect used after JSON validation.
pub fn build_physical_ship(
    commands: &mut Commands<'_, '_>,
    asset_server: &Res<'_, AssetServer>,
    event: &SpawnEntity,
    template: &impl ShipTemplateBase,
) -> (Entity, PropulsionConfig) {
    // Extract uniform scale from event (use max of x, y, z for uniform scaling).
    let scale = event.scale.x.max(event.scale.y).max(event.scale.z);

    // Compute debug axes length from the bounding box in the template JSON, scaled.
    let axis_length = compute_debug_axis_length(template, scale);

    tracing::debug!(
        "build_physical_ship: axis_length={axis_length:.1} from bounding_box in template (scale={scale})"
    );

    // Build the ship entity spawn command.
    // Use delta-v-spawn for collision shape conversion (ADR-0047).
    #[allow(clippy::expect_used)] // INVARIANT: validated by delta-v-json upstream (ADR-0013)
    let _collision_shape_data = shape_from_json(template.collision_shape(), scale)
        .map_err(|e| tracing::error!("collision shape invalid: {}", e))
        .expect("collision shape must be valid (ADR-0013)");

    let ship_entity = spawn_ship_entity(commands, asset_server, event, template, axis_length);

    // Entity type and ID for navigation list display (inserted separately to avoid tuple limit)
    commands.entity(ship_entity).insert((
        EntityType(template.entity_type().to_string()),
        WorldEntityId(event.id.clone()),
    ));

    // Add Weapon components from template (M4).
    // Per ADR-0014, all gameplay values come from JSON.
    // Weapons reference weapon definitions by name, which in turn reference projectile definitions.
    add_weapon_components(commands, ship_entity, template);

    // Load propulsion configuration from thruster definitions.
    // Use the first main thruster in the array (M2: single active thruster).
    let propulsion_config = load_propulsion_config(template);

    // Add Propulsion component to the ship entity.
    add_propulsion_component(commands, ship_entity, template, &propulsion_config);

    (ship_entity, propulsion_config)
}

/// Computes the debug axis length from the template's bounding box and scale.
fn compute_debug_axis_length(template: &impl ShipTemplateBase, scale: f32) -> f32 {
    let half_extent = Vec3::new(
        (template.bounding_box().max.x - template.bounding_box().min.x) / 2.0,
        (template.bounding_box().max.y - template.bounding_box().min.y) / 2.0,
        (template.bounding_box().max.z - template.bounding_box().min.z) / 2.0,
    );
    half_extent.x.max(half_extent.y).max(half_extent.z) * 2.0 * scale
}

/// Spawns the base ship entity with physics, collision, health, and targetable components.
#[allow(clippy::expect_used)] // INVARIANT: collision shape validated by delta-v-json upstream (ADR-0013)
fn spawn_ship_entity(
    commands: &mut Commands<'_, '_>,
    asset_server: &Res<'_, AssetServer>,
    event: &SpawnEntity,
    template: &impl ShipTemplateBase,
    axis_length: f32,
) -> Entity {
    commands
        .spawn((
            Transform {
                translation: event.position,
                rotation: event.rotation,
                scale: event.scale,
            },
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            PendingShipMesh {
                gltf_handle: asset_server.load::<Gltf>(event.mesh_template_path.clone()),
            },
            DebugAxesEligible::new(event.id.clone(), axis_length),
            // Physics components: mass and inertia from template JSON (ADR-0014)
            RigidBody::new(template.mass().value, template.inertia_scale()),
            FlightAssist,
            CollisionShape(
                shape_from_json(template.collision_shape(), 1.0)
                    .map_err(|e| tracing::error!("collision shape invalid: {}", e))
                    .expect("collision shape must be valid (ADR-0013)"),
            ),
            CollisionLayersComponent::new(delta_v_types::collision::layers::SHIP),
            // Health component for damage model (M4)
            Health::new(template.health().value),
            // Targetable for targeting/navigation menu (M6 step 11a)
            Targetable,
        ))
        .id()
}

/// Adds Weapon components to the ship entity from the template.
#[allow(clippy::expect_used)] // INVARIANT: validated by delta-v-json upstream (ADR-0013, ADR-0040)
fn add_weapon_components(
    commands: &mut Commands<'_, '_>,
    ship_entity: Entity,
    template: &impl ShipTemplateBase,
) {
    for (i, weapon_ref) in template.weapons().iter().enumerate() {
        // Load weapon definition
        let weapon_def = load_weapon_definition(&weapon_ref.name)
            .expect("weapon definition must exist (ADR-0013)");
        let weapon_def: WeaponTemplateJson = serde_json::from_value(weapon_def)
            .expect("weapon definition deserialization must succeed (ADR-0040)");

        // Load projectile definition referenced by weapon
        let projectile_def = load_projectile_definition(&weapon_def.projectile_template)
            .expect("projectile definition must exist (ADR-0013)");
        let projectile_def: ProjectileDefinitionJson = serde_json::from_value(projectile_def)
            .expect("projectile definition deserialization must succeed (ADR-0040)");

        // Validate weapon sound file exists (ADR-0013: no silent fallbacks).
        if let Some(ref sound) = weapon_def.sound {
            let path = format!("assets/audio/{sound}");
            if !std::path::Path::new(&path).exists() {
                tracing::warn!(
                    "[audio] weapon sound file not found: {} (referenced in template)",
                    path
                );
            }
        }
        commands.entity(ship_entity).insert(Weapon {
            slot: u32::try_from(i).expect("weapon slot index fits in u32"),
            cooldown: 0.0,
            weapon_name: weapon_ref.name.clone(),
            projectile_speed: projectile_def.speed.value,
            damage: projectile_def.damage.value,
            fire_rate: weapon_def.fire_rate.value,
            lifetime: projectile_def.lifetime.value,
            projectile_radius: projectile_def.radius.value,
            sound: weapon_def.sound.clone(),
        });
    }
}

/// Loads propulsion configuration from thruster definitions.
#[allow(clippy::expect_used)] // INVARIANT: validated by delta-v-json upstream (ADR-0013, ADR-0040)
fn load_propulsion_config(template: &impl ShipTemplateBase) -> PropulsionConfig {
    let active_main_thruster_index = 0_usize;
    let main_thruster_names = template.main_thruster_names().to_vec();
    let main_thruster_name = main_thruster_names
        .get(active_main_thruster_index)
        .expect("at least one main thruster must be defined (schema minItems: 1)");

    let main_thruster_def = load_main_thruster_definition(main_thruster_name)
        .expect("main thruster definition must exist (ADR-0013)");
    let main_thruster_def: MainThrusterDefinitionJson = serde_json::from_value(main_thruster_def)
        .expect("main thruster definition deserialization must succeed (ADR-0040)");

    let maneuvering_thruster_name = template.maneuvering_thruster_name().to_string();
    let maneuvering_thruster_def = load_maneuvering_thruster_definition(&maneuvering_thruster_name)
        .expect("maneuvering thruster definition must exist (ADR-0013)");
    let maneuvering_thruster_def: ManeuveringThrusterDefinitionJson =
        serde_json::from_value(maneuvering_thruster_def)
            .expect("maneuvering thruster definition deserialization must succeed (ADR-0040)");

    PropulsionConfig {
        max_forward_thrust: main_thruster_def.max_forward_thrust.value,
        max_backward_thrust: main_thruster_def.max_backward_thrust.value,
        max_torque: maneuvering_thruster_def.max_torque.value,
        max_strafe_thrust: maneuvering_thruster_def.max_strafe_thrust.value,
        rotation_ramp_ticks: maneuvering_thruster_def.rotation_ramp_ticks,
        thrust_sound: main_thruster_def.sound,
    }
}

/// Adds the Propulsion component to the ship entity.
fn add_propulsion_component(
    commands: &mut Commands<'_, '_>,
    ship_entity: Entity,
    template: &impl ShipTemplateBase,
    propulsion_config: &PropulsionConfig,
) {
    let main_thruster_names = template.main_thruster_names().to_vec();
    let maneuvering_thruster_name = template.maneuvering_thruster_name().to_string();
    let active_main_thruster_index = 0_usize;

    commands.entity(ship_entity).insert(Propulsion {
        main_thruster_names,
        maneuvering_thruster_name,
        max_forward_thrust: propulsion_config.max_forward_thrust,
        max_backward_thrust: propulsion_config.max_backward_thrust,
        max_torque: propulsion_config.max_torque,
        max_strafe_thrust: propulsion_config.max_strafe_thrust,
        rotation_ramp_ticks: propulsion_config.rotation_ramp_ticks,
        active_main_thruster_index,
    });
}

/// Marker component for a pending ship entity waiting for its mesh to load.
#[derive(Component)]
pub struct PendingShipMesh {
    /// Handle to the glTF asset being loaded.
    pub gltf_handle: Handle<Gltf>,
}

impl crate::mesh_attachment::PendingMesh for PendingShipMesh {
    fn gltf_handle(&self) -> &Handle<Gltf> {
        &self.gltf_handle
    }
}
