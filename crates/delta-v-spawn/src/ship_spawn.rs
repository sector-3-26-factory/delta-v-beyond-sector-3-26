// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Shared ship spawning utilities.
//!
//! Provides common functionality for spawning ship entities (player, NPC, static)
//! from templates. This avoids duplication across domain crates.
//!
//! See ADR-0047 (centralized spawning) and ADR-0051 (dependency hierarchy).

use crate::collision::scale_collision_shape;
use crate::template_extraction::resolve_mass;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_assets::resolve_sound_path;
use delta_v_assets::template::{
    load_main_thruster_definition, load_maneuvering_thruster_definition,
    load_projectile_definition, load_weapon_definition,
};
use delta_v_core::{
    DebugAxesEligible, EntityType, FlightAssist, Health, Propulsion, SelectedWeapon, SpawnEntity,
    Targetable, Weapon, WorldEntityId,
};
use delta_v_physics::{CollisionLayersComponent, CollisionShape, RigidBody};
use delta_v_types::{PropulsionConfig, ShipTemplateBase};

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
    // Collision shape is already validated and converted to runtime type.
    let _collision_shape_data = scale_collision_shape(template.collision_shape(), scale);

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

    // Initialize SelectedWeapon resource with max_weapons_count
    commands.insert_resource(SelectedWeapon {
        index: 0,
        max_weapons_count: template.max_weapons_count(),
    });

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
    // Extract uniform scale from event (use max of x, y, z for uniform scaling).
    let scale = event.scale.x.max(event.scale.y).max(event.scale.z);

    // Resolve mass: use override if present, otherwise use template mass.
    // Mass is NOT scaled - it is used as-is or overridden.
    let mass = resolve_mass(template.mass(), event.mass);

    // Derive mesh path from template path using filesystem functions.
    // The mesh is always at mesh.glb in the template's directory.
    let mesh_path = event.mesh_path();

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
                gltf_handle: asset_server.load::<Gltf>(&mesh_path),
            },
            DebugAxesEligible::new(event.id.clone(), axis_length),
            // Physics components: mass and inertia from template JSON (ADR-0014)
            RigidBody::new(mass, template.inertia_scale()),
            FlightAssist,
            CollisionShape(scale_collision_shape(template.collision_shape(), scale)),
            CollisionLayersComponent::new(delta_v_types::collision::layers::SHIP),
            // Health component for damage model (M4)
            Health::new(template.health()),
            // Targetable for targeting/navigation menu (M6 step 11a)
            Targetable,
        ))
        .id()
}

/// Adds Weapon components to the ship entity as children from the template.
///
/// Each weapon is spawned as a child entity with a Weapon component.
/// This allows multiple weapons to be stored and selected.
#[allow(clippy::expect_used)] // INVARIANT: validated by delta-v-json upstream (ADR-0013, ADR-0040)
fn add_weapon_components(
    commands: &mut Commands<'_, '_>,
    ship_entity: Entity,
    template: &impl ShipTemplateBase,
) {
    for (i, weapon_name) in template.weapons().iter().enumerate() {
        // Load weapon definition (already converted to runtime type with SI units)
        let weapon_def =
            load_weapon_definition(weapon_name).expect("weapon definition must exist (ADR-0013)");

        // Load projectile definition (already converted to runtime type with SI units)
        let projectile_def = load_projectile_definition(&weapon_def.projectile_template)
            .expect("projectile definition must exist (ADR-0013)");

        // Resolve sound paths at spawn time for performance
        let weapon_dir = format!("assets/components/weapons/{weapon_name}");
        let projectile_dir = format!(
            "assets/components/projectiles/{}",
            weapon_def.projectile_template
        );

        let fire_sound = resolve_sound_path(&weapon_dir, "fire");
        let hit_sound = resolve_sound_path(&projectile_dir, "hit");

        // Spawn weapon as a child entity
        commands.entity(ship_entity).with_children(|parent| {
            parent.spawn(Weapon {
                slot: u32::try_from(i).expect("weapon slot index fits in u32"),
                cooldown: 0.0,
                weapon_name: weapon_name.clone(),
                projectile_speed: projectile_def.speed,
                damage: projectile_def.damage,
                fire_rate: weapon_def.fire_rate,
                lifetime: projectile_def.lifetime,
                projectile_radius: projectile_def.radius,
                fire_sound,
                hit_sound,
                projectile_template: weapon_def.projectile_template.clone(),
            });
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

    // Load main thruster definition (already converted to runtime type with SI units)
    let main_thruster_def = load_main_thruster_definition(main_thruster_name)
        .expect("main thruster definition must exist (ADR-0013)");

    let maneuvering_thruster_name = template.maneuvering_thruster_name().to_string();
    // Load maneuvering thruster definition (already converted to runtime type with SI units)
    let maneuvering_thruster_def = load_maneuvering_thruster_definition(&maneuvering_thruster_name)
        .expect("maneuvering thruster definition must exist (ADR-0013)");

    // Resolve thrust sound path at spawn time for performance
    let thruster_dir = format!("assets/components/propulsion/main-thrusters/{main_thruster_name}");
    let thrust_sound = resolve_sound_path(&thruster_dir, "thrust");

    PropulsionConfig {
        max_forward_thrust: main_thruster_def.max_forward_thrust,
        max_backward_thrust: main_thruster_def.max_backward_thrust,
        max_torque: maneuvering_thruster_def.max_torque,
        max_strafe_thrust: maneuvering_thruster_def.max_strafe_thrust,
        rotation_ramp_ticks: maneuvering_thruster_def.rotation_ramp_ticks,
        thrust_sound,
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
    let max_propulsions_count = template.max_propulsions_count();

    commands.entity(ship_entity).insert(Propulsion {
        main_thruster_names,
        maneuvering_thruster_name,
        max_forward_thrust: propulsion_config.max_forward_thrust,
        max_backward_thrust: propulsion_config.max_backward_thrust,
        max_torque: propulsion_config.max_torque,
        max_strafe_thrust: propulsion_config.max_strafe_thrust,
        rotation_ramp_ticks: propulsion_config.rotation_ramp_ticks,
        active_main_thruster_index,
        max_propulsions_count,
    });
}

/// Marker component for a pending ship entity waiting for its mesh to load.
///
/// Used by the ship spawn system in delta-v-ships. Implements the
/// [`PendingMesh`] trait for the generic mesh attachment system.
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
