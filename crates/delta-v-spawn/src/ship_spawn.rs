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
use delta_v_assets::template::{load_projectile_definition, load_weapon_definition};
use delta_v_core::{
    DebugAxesEligible, EntityType, FlightAssist, Health, SpawnEntity, Targetable, Weapon,
    WorldEntityId,
};
use delta_v_physics::{CollisionLayersComponent, CollisionShape, RigidBody};
use delta_v_types::{
    BoundingBoxJson, CollisionShapeJson, PhysicalQuantityJson, ProjectileDefinitionJson,
    WeaponReference, WeaponTemplateJson,
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
}

/// Builds the base physical ship entity with common components.
///
/// This function creates the ship entity with physics, collision, health, and weapons.
/// It is called by all ship spawn functions (player, static, AI) to avoid duplication.
///
/// # Arguments
/// * `commands` - Bevy commands for entity spawning
/// * `asset_server` - Asset server for loading glTF meshes
/// * `event` - `SpawnEntity` event containing position, rotation, scale, entity ID, and `mesh_template_path`
/// * `template` - Ship template implementing [`ShipTemplateBase`] with physical properties
///
/// # Returns
/// The spawned ship entity ID.
///
/// # Panics
/// Panics if the collision shape from the template is invalid. This should never happen
/// because templates are validated by delta-v-json before spawning (ADR-0013).
// INVARIANT: Indexing is safe (weapons iter), expect used after JSON validation.
#[allow(
    clippy::option_if_let_else,
    clippy::indexing_slicing,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]
pub fn build_physical_ship(
    commands: &mut Commands<'_, '_>,
    asset_server: &Res<'_, AssetServer>,
    event: &SpawnEntity,
    template: &impl ShipTemplateBase,
) -> Entity {
    // Extract uniform scale from event (use max of x, y, z for uniform scaling).
    let scale = event.scale.x.max(event.scale.y).max(event.scale.z);

    // Compute debug axes length from the bounding box in the template JSON, scaled.
    let half_extent = Vec3::new(
        (template.bounding_box().max.x - template.bounding_box().min.x) / 2.0,
        (template.bounding_box().max.y - template.bounding_box().min.y) / 2.0,
        (template.bounding_box().max.z - template.bounding_box().min.z) / 2.0,
    );
    let axis_length = half_extent.x.max(half_extent.y).max(half_extent.z) * 2.0 * scale;

    tracing::debug!(
        "build_physical_ship: axis_length={axis_length:.1} from bounding_box in template (scale={scale})"
    );

    // Build the ship entity spawn command.
    // Use delta-v-spawn for collision shape conversion (ADR-0047).
    let collision_shape_data = shape_from_json(template.collision_shape(), scale)
        .map_err(|e| tracing::error!("collision shape invalid: {}", e))
        .expect("collision shape must be valid (ADR-0013)");

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
            PendingShipMesh {
                gltf_handle: asset_server.load::<Gltf>(event.mesh_template_path.clone()),
            },
            DebugAxesEligible::new(event.id.clone(), axis_length),
            // Physics components: mass and inertia from template JSON (ADR-0014)
            RigidBody::new(template.mass().value, template.inertia_scale()),
            FlightAssist,
            CollisionShape(collision_shape_data),
            CollisionLayersComponent::new(delta_v_types::collision::layers::SHIP),
            // Health component for damage model (M4)
            Health::new(template.health().value),
            // Targetable for targeting/navigation menu (M6 step 11a)
            Targetable,
        ))
        .id();

    // Entity type and ID for navigation list display (inserted separately to avoid tuple limit)
    commands.entity(ship_entity).insert((
        EntityType(template.entity_type().to_string()),
        WorldEntityId(event.id.clone()),
    ));

    // Add Weapon components from template (M4).
    // Per ADR-0014, all gameplay values come from JSON.
    // Weapons reference weapon definitions by name, which in turn reference projectile definitions.
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
            slot: i as u32,
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

    ship_entity
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
