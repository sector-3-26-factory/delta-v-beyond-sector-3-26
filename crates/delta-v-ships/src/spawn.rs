// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Ship entity spawning from templates.
//!
//! Per ADR-0038 (entity template system), ships are spawned from templates
//! loaded and validated by delta-v-json. Templates define
//! cameras, physical properties (mass, inertia), and propulsion configuration.
//! All meshes come from glTF files (ADR-0019).
//!
//! Per ADR-0014, all gameplay values (mass, thrust, torque) come from JSON
//! — never from Rust constants. The template `Value` is deserialized into
//! [`PlayerShipTemplate`] via `serde_json::from_value` (a one-liner per ADR-0040).
//!
//! See also ADR-0005 (plugin architecture) and ADR-0006 (coordinate system).

use crate::cockpit::CockpitOverlayResource;
use crate::ship_templates::{
    MainThrusterTemplate, ManeuveringThrusterTemplate, PlayerShipTemplate, ShipPropulsionConfig,
    StaticShipTemplate,
};
use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_core::{
    ChaseCameraOffset, DebugAxesEligible, FlightAssist, Health, PlayerShipEntity, SpawnEntity,
    Weapon,
};
use delta_v_physics::{CollisionLayersComponent, CollisionShape, RigidBody};
use delta_v_spawn::collision::shape_from_json;
use delta_v_types::collision::layers;

/// Marker component for a pending ship entity waiting for its mesh to load.
#[derive(Component)]
pub struct PendingShipMesh {
    /// Handle to the glTF asset being loaded.
    gltf_handle: Handle<Gltf>,
}

impl delta_v_spawn::mesh_attachment::PendingMesh for PendingShipMesh {
    fn gltf_handle(&self) -> &Handle<Gltf> {
        &self.gltf_handle
    }
}

/// Spawns ship entities in response to `SpawnEntity` events.
///
/// Listens for events with `entity_type` matching known ship types:
/// - `"player_controlled_ship"`: Player-controlled ship
/// - `"ship"`: Non-player ship (static/NPC)
/// - `"npc_ship"`: NPC-controlled ship (future)
///
/// The event's `template` field contains validated template JSON from delta-v-json.
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_ship(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    mut events: MessageReader<'_, '_, SpawnEntity>,
) {
    for event in events.read() {
        match event.entity_type.as_str() {
            "player_controlled_ship" => spawn_player_ship(&mut commands, &asset_server, event),
            "ship" => spawn_static_ship(&mut commands, &asset_server, event),
            "npc_ship" => {
                // NPC ships: future implementation
                log::warn!("NPC ship spawning not yet implemented");
            }
            // Other entity types (e.g. "asteroid") are handled by other plugins.
            // Silently skip — a single plugin cannot know whether another plugin
            // will handle the event.
            _ => {}
        }
    }
}

/// Deserializes the template JSON into a [`PlayerShipTemplate`] struct.
///
/// Per ADR-0040, the template `Value` has already been validated and
/// filled with schema defaults by `delta-v-json`, so deserialization
/// into the struct is a one-liner.
#[allow(clippy::expect_used)] // INVARIANT: template validated by delta-v-json; cannot fail
fn deserialize_template(event: &SpawnEntity) -> PlayerShipTemplate {
    serde_json::from_value(event.template.clone())
        .expect("template deserialization must succeed (validated by delta-v-json, ADR-0040)")
}

/// Spawns camera entities for each available camera definition.
///
/// Only cameras with `available: true` are spawned as child entities
/// of the ship. Positions and targets are scaled by the entity scale.
fn spawn_cameras(
    commands: &mut Commands<'_, '_>,
    ship_entity: Entity,
    template: &PlayerShipTemplate,
    scale: f32,
) {
    for (name, camera) in [
        ("cockpit", &template.cameras.cockpit),
        ("chase", &template.cameras.chase),
        ("rear", &template.cameras.rear),
        ("front", &template.cameras.front),
        ("left", &template.cameras.left),
        ("right", &template.cameras.right),
        ("top", &template.cameras.top),
        ("bottom", &template.cameras.bottom),
    ] {
        if camera.available {
            let position =
                Vec3::new(camera.position.x, camera.position.y, camera.position.z) * scale;
            let target = Vec3::new(camera.target.x, camera.target.y, camera.target.z) * scale;
            commands.entity(ship_entity).with_children(|parent| {
                let _camera_entity = parent.spawn((
                    Transform::from_translation(position).looking_at(target, Vec3::Y),
                    delta_v_core::CameraFollow {
                        target: ship_entity,
                        offset: position,
                    },
                ));
                log::debug!("spawned {name} camera at {position:?}");
            });
        }
    }
}

/// Inserts player-specific resources after the ship entity is spawned.
///
/// Stores the player ship entity ID, chase camera offset, propulsion
/// configuration, and cockpit overlay resource.
fn insert_player_resources(
    commands: &mut Commands<'_, '_>,
    ship_entity: Entity,
    template: &PlayerShipTemplate,
    main: &MainThrusterTemplate,
    maneuvering: &ManeuveringThrusterTemplate,
    active_index: usize,
    scale: f32,
) {
    let chase_offset = Vec3::new(
        template.cameras.chase.position.x,
        template.cameras.chase.position.y,
        template.cameras.chase.position.z,
    ) * scale;
    commands.insert_resource(PlayerShipEntity(ship_entity));
    commands.insert_resource(ChaseCameraOffset(chase_offset));
    commands.insert_resource(ShipPropulsionConfig {
        max_forward_thrust: main.max_forward_thrust.value,
        max_backward_thrust: main.max_backward_thrust.value,
        max_torque: maneuvering.max_torque.value,
        max_strafe_thrust: maneuvering.max_strafe_thrust.value,
        active_main_thruster_index: active_index,
        rotation_ramp_ticks: maneuvering.rotation_ramp_ticks,
    });
    commands.insert_resource(CockpitOverlayResource {
        stations: template.cockpit.stations.clone(),
    });
}

/// Spawns the player-controlled ship from a template event.
///
/// Template is validated by delta-v-json; structure is guaranteed.
/// Mass, inertia, and propulsion values are read from the template JSON
/// per ADR-0014 (gameplay values in JSON, not Rust constants).
///
/// All 8 cameras are defined in the template with position, target, and availability.
/// Only cameras with `available: true` are spawned as camera entities.
///
/// Debug axes length is computed from the bounding box stored in the template JSON.
#[allow(
    clippy::option_if_let_else,
    clippy::indexing_slicing,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]
fn spawn_player_ship(
    commands: &mut Commands<'_, '_>,
    asset_server: &Res<'_, AssetServer>,
    event: &SpawnEntity,
) {
    // Deserialize template JSON into typed struct (ADR-0040 one-liner).
    let template = deserialize_template(event);

    // Extract propulsion values from the active main thruster.
    let active_index = 0_usize; // M2: single active thruster
    let main = &template.propulsion.main_thrusters[active_index];
    let maneuvering = &template.propulsion.maneuvering_thruster;

    // Derive mesh path from the mesh template path (always mesh.glb in the template directory).
    let mesh_path = event.mesh_template_path.replace("ship.json", "mesh.glb");

    // Queue glTF mesh load.
    let gltf_handle = asset_server.load::<Gltf>(&mesh_path);

    // Extract uniform scale from event (use max of x, y, z for uniform scaling).
    let scale = event.scale.x.max(event.scale.y).max(event.scale.z);

    // Compute debug axes length from the bounding box in the template JSON, scaled.
    // The bounding_box is already in the template JSON (computed by tooling).
    let half_extent = Vec3::new(
        (template.bounding_box.max.x - template.bounding_box.min.x) / 2.0,
        (template.bounding_box.max.y - template.bounding_box.min.y) / 2.0,
        (template.bounding_box.max.z - template.bounding_box.min.z) / 2.0,
    );
    let axis_length = half_extent.x.max(half_extent.y).max(half_extent.z) * 2.0 * scale;

    log::debug!(
        "spawn_player_ship: axis_length={axis_length:.1} from bounding_box in template (scale={scale})"
    );

    // Build the ship entity spawn command.
    // Use delta-v-spawn for collision shape conversion (ADR-0047).
    let collision_shape_data = shape_from_json(&template.collision_shape, scale)
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
            PendingShipMesh { gltf_handle },
            DebugAxesEligible::new(event.id.clone(), axis_length),
            // Physics components: mass and inertia from template JSON (ADR-0014)
            RigidBody::new(template.mass.value, template.inertia_scale),
            FlightAssist,
            CollisionShape(collision_shape_data),
            CollisionLayersComponent::new(layers::SHIP),
            // Health component for damage model (M4)
            Health::new(template.health.value),
        ))
        .id();

    // Add Weapon components from template (M4).
    // Per ADR-0014, all gameplay values come from JSON.
    for (i, weapon_json) in template.weapons.iter().enumerate() {
        commands.entity(ship_entity).insert(Weapon {
            slot: i as u32,
            cooldown: 0.0,
            projectile_speed: weapon_json.projectile_speed.value,
            damage: weapon_json.damage.value,
            fire_rate: weapon_json.fire_rate.value,
            lifetime: weapon_json.lifetime.value,
            projectile_radius: weapon_json.projectile_radius.value,
        });
    }
    // Spawn cameras for each available camera definition, scaled by the entity scale.
    spawn_cameras(commands, ship_entity, &template, scale);

    // Store player ship ID, chase camera offset, propulsion config, and cockpit overlay.
    insert_player_resources(
        commands,
        ship_entity,
        &template,
        main,
        maneuvering,
        active_index,
        scale,
    );

    log::info!(
        "player controlled ship spawned at position ({:.1}, {:.1}, {:.1}) from {} (mass={}kg, forward_thrust={}N, backward_thrust={}N)",
        event.position.x,
        event.position.y,
        event.position.z,
        mesh_path,
        template.mass.value,
        main.max_forward_thrust.value,
        main.max_backward_thrust.value,
    );
}

/// Spawns a non-player ship from a template event.
///
/// This is for `entity_type: "ship"` templates — ships that don't require
/// player input, cameras, or propulsion configuration.
///
/// Template is validated by delta-v-json; structure is guaranteed.
#[allow(
    clippy::option_if_let_else,
    clippy::indexing_slicing,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]
fn spawn_static_ship(
    commands: &mut Commands<'_, '_>,
    asset_server: &Res<'_, AssetServer>,
    event: &SpawnEntity,
) {
    // Deserialize template JSON into typed struct (ADR-0040 one-liner).
    let template: StaticShipTemplate = serde_json::from_value(event.template.clone())
        .expect("template deserialization must succeed (validated by delta-v-json, ADR-0040)");

    // Derive mesh path from the mesh template path (always mesh.glb in the template directory).
    let mesh_path = event.mesh_template_path.replace("ship.json", "mesh.glb");

    // Queue glTF mesh load.
    let gltf_handle = asset_server.load::<Gltf>(&mesh_path);

    // Extract uniform scale from event (use max of x, y, z for uniform scaling).
    let scale = event.scale.x.max(event.scale.y).max(event.scale.z);

    // Compute debug axes length from the bounding box in the template JSON, scaled.
    let half_extent = Vec3::new(
        (template.bounding_box.max.x - template.bounding_box.min.x) / 2.0,
        (template.bounding_box.max.y - template.bounding_box.min.y) / 2.0,
        (template.bounding_box.max.z - template.bounding_box.min.z) / 2.0,
    );
    let axis_length = half_extent.x.max(half_extent.y).max(half_extent.z) * 2.0 * scale;

    log::debug!(
        "spawn_static_ship: axis_length={axis_length:.1} from bounding_box in template (scale={scale})"
    );

    // Build the ship entity spawn command.
    // Use delta-v-spawn for collision shape conversion (ADR-0047).
    let collision_shape_data = shape_from_json(&template.collision_shape, scale)
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
            PendingShipMesh { gltf_handle },
            DebugAxesEligible::new(event.id.clone(), axis_length),
            // Physics components: mass and inertia from template JSON (ADR-0014)
            RigidBody::new(template.mass.value, template.inertia_scale),
            FlightAssist,
            CollisionShape(collision_shape_data),
            // Health component for damage model (M4)
            Health::new(template.health.value),
        ))
        .id();

    // Add Weapon components from template (M4).
    for (i, weapon_json) in template.weapons.iter().enumerate() {
        commands.entity(ship_entity).insert(Weapon {
            slot: i as u32,
            cooldown: 0.0,
            projectile_speed: weapon_json.projectile_speed.value,
            damage: weapon_json.damage.value,
            fire_rate: weapon_json.fire_rate.value,
            lifetime: weapon_json.lifetime.value,
            projectile_radius: weapon_json.projectile_radius.value,
        });
    }

    log::info!(
        "static ship spawned at position ({:.1}, {:.1}, {:.1}) from {} (mass={}kg)",
        event.position.x,
        event.position.y,
        event.position.z,
        mesh_path,
        template.mass.value,
    );
}
