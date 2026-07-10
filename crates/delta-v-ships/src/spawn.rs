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
use crate::ship_templates::{PlayerShipTemplate, ShipPropulsionConfig, StaticShipTemplate};
use bevy::prelude::*;
use delta_v_core::{ActiveCameraName, CameraName, PlayerShipEntity, RenderLayer, SpawnEntity};
use delta_v_spawn::template_extraction::png_dimensions;
use delta_v_spawn::{ShipTemplateBase, build_physical_ship};
use delta_v_types::{BoundingBoxJson, CollisionShapeJson, PhysicalQuantityJson, WeaponReference};

/// Spawns ship entities in response to `SpawnEntity` events.
///
/// Listens for events with `entity_type` matching known ship types:
/// - `"player_controlled_ship"`: Player-controlled ship
/// - `"ship"`: Non-player ship (static/NPC)
/// - `"npc_ship"`: NPC-controlled ship (future)
///
/// The event's `template` field contains validated template JSON from delta-v-json.
// INVARIANT: MessageReader::read returns events by value; pass by value is idiomatic.
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
                tracing::warn!("NPC ship spawning not yet implemented");
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
/// All ship cameras render on `Layer(0)` with `order: 0`.
/// The first available camera in the order gets the `ActiveMainCamera` marker.
fn spawn_cameras(
    commands: &mut Commands<'_, '_>,
    ship_entity: Entity,
    template: &PlayerShipTemplate,
    scale: f32,
) {
    let active_camera_name = "cockpit";
    for (name, camera) in [
        ("cockpit", &template.cameras.cockpit),
        ("front", &template.cameras.front),
        ("rear", &template.cameras.rear),
        ("left", &template.cameras.left),
        ("right", &template.cameras.right),
        ("top", &template.cameras.top),
        ("bottom", &template.cameras.bottom),
        ("drone", &template.cameras.drone),
    ] {
        if camera.available {
            let position =
                Vec3::new(camera.position.x, camera.position.y, camera.position.z) * scale;
            let target = Vec3::new(camera.target.x, camera.target.y, camera.target.z) * scale;
            commands.entity(ship_entity).with_children(|parent| {
                let mut camera_entity = parent.spawn((
                    Camera3d::default(),
                    Camera {
                        order: 0,
                        is_active: name == active_camera_name,
                        ..default()
                    },
                    Transform::from_translation(position).looking_at(target, Vec3::Y),
                    RenderLayer::Gameplay.render_layers(),
                    CameraName(name),
                ));
                if name == active_camera_name {
                    camera_entity.insert(delta_v_core::ActiveMainCamera);
                }
                tracing::debug!("spawned {name} camera at {position:?} on layer 0");
            });
        }
    }
}

/// Inserts player-specific resources after the ship entity is spawned.
///
/// Stores the player ship entity ID, camera configuration, propulsion
/// configuration, cockpit overlay resource, and ship sounds.
fn insert_player_resources(
    commands: &mut Commands<'_, '_>,
    ship_entity: Entity,
    template_path: &str,
    template: &PlayerShipTemplate,
    propulsion_config: ShipPropulsionConfig,
) {
    commands.insert_resource(PlayerShipEntity(ship_entity));
    commands.insert_resource(propulsion_config);
    // Initialize ActiveCameraName resource to track the current camera.
    // The cockpit camera is the default active camera.
    commands.insert_resource(ActiveCameraName("cockpit".to_string()));
    // Convert template file path to asset directory path.
    // The template_path is a file path like "templates/ships/space-fighter-comrade1280/player_controlled_ship.json".
    // We need the directory path relative to the assets/ root: "templates/ships/space-fighter-comrade1280".
    // The asset server loads from "assets/" + directory_path + "/" + texture_name.
    let cockpit_dir = template_path
        .rsplit_once('/')
        .map_or(template_path, |(d, _)| d);
    // Read the first station's texture dimensions from the PNG file.
    // Slot coordinates are in texture pixel space and must be scaled to viewport percentages.
    // Per ADR-0013, missing or invalid PNG is a hard error — no silent fallback.
    let (texture_width, texture_height) =
        template.cockpit.stations.first().map_or((1.0, 1.0), |s| {
            let path = format!("assets/{}/{}", cockpit_dir, s.texture);
            #[allow(clippy::expect_used)]
            png_dimensions(&path).expect("failed to read cockpit texture PNG dimensions (ADR-0013)")
        });
    commands.insert_resource(CockpitOverlayResource {
        template_path: cockpit_dir.to_string(),
        stations: template.cockpit.stations.clone(),
        texture_width,
        texture_height,
    });
    // Insert ShipSounds resource from template.
    // Schema provides default {} for sounds, so template.sounds is always present.
    // Audio systems handle None values by skipping playback.
    let ship_sounds = template.sounds.clone();
    commands.insert_resource(ship_sounds);
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
// INVARIANT: Indexing is safe (active_main_thruster_index=0, weapons iter), expect used after JSON validation.
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

    // Build the physical ship (common components: physics, collision, health, weapons).
    let (ship_entity, propulsion_config) =
        build_physical_ship(commands, asset_server, event, &template);

    // Convert to ShipPropulsionConfig for resource insertion.
    let ship_propulsion_config = ShipPropulsionConfig {
        max_forward_thrust: propulsion_config.max_forward_thrust,
        max_backward_thrust: propulsion_config.max_backward_thrust,
        max_torque: propulsion_config.max_torque,
        max_strafe_thrust: propulsion_config.max_strafe_thrust,
        active_main_thruster_index: 0,
        rotation_ramp_ticks: propulsion_config.rotation_ramp_ticks,
    };

    // Spawn cameras for each available camera definition, scaled by the entity scale.
    let scale = event.scale.x.max(event.scale.y).max(event.scale.z);
    spawn_cameras(commands, ship_entity, &template, scale);

    // Store player ship ID, propulsion config, and cockpit overlay.
    insert_player_resources(
        commands,
        ship_entity,
        &event.template_path,
        &template,
        ship_propulsion_config,
    );

    tracing::info!(
        "player controlled ship spawned at position ({:.1}, {:.1}, {:.1}) from {} (mass={}kg)",
        event.position.x,
        event.position.y,
        event.position.z,
        event.mesh_template_path,
        template.mass.value,
    );
}

/// Spawns a non-player ship from a template event.
///
/// This is for `entity_type: "ship"` templates — ships that don't require
/// player input, cameras, or propulsion configuration.
///
/// Template is validated by delta-v-json; structure is guaranteed.
// INVARIANT: Indexing is safe (weapons iter), expect used after JSON validation.
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

    // Build the physical ship (common components: physics, collision, health, weapons).
    let (_ship_entity, _propulsion_config) =
        build_physical_ship(commands, asset_server, event, &template);

    tracing::info!(
        "static ship spawned at position ({:.1}, {:.1}, {:.1}) from {} (mass={}kg)",
        event.position.x,
        event.position.y,
        event.position.z,
        event.mesh_template_path,
        template.mass.value,
    );
}

/// Implement `ShipTemplateBase` for `PlayerShipTemplate`
impl ShipTemplateBase for PlayerShipTemplate {
    fn mass(&self) -> &PhysicalQuantityJson {
        &self.mass
    }
    fn inertia_scale(&self) -> f32 {
        self.inertia_scale
    }
    fn bounding_box(&self) -> &BoundingBoxJson {
        &self.bounding_box
    }
    fn collision_shape(&self) -> &CollisionShapeJson {
        &self.collision_shape
    }
    fn health(&self) -> &PhysicalQuantityJson {
        &self.health
    }
    fn weapons(&self) -> &[WeaponReference] {
        &self.weapons
    }
    fn entity_type(&self) -> &'static str {
        "player_controlled_ship"
    }
    fn main_thruster_names(&self) -> &[String] {
        &self.propulsion.main_thruster_names
    }
    fn maneuvering_thruster_name(&self) -> &str {
        &self.propulsion.maneuvering_thruster
    }
}

/// Implement `ShipTemplateBase` for `StaticShipTemplate`
impl ShipTemplateBase for StaticShipTemplate {
    fn mass(&self) -> &PhysicalQuantityJson {
        &self.mass
    }
    fn inertia_scale(&self) -> f32 {
        self.inertia_scale
    }
    fn bounding_box(&self) -> &BoundingBoxJson {
        &self.bounding_box
    }
    fn collision_shape(&self) -> &CollisionShapeJson {
        &self.collision_shape
    }
    fn health(&self) -> &PhysicalQuantityJson {
        &self.health
    }
    fn weapons(&self) -> &[WeaponReference] {
        &self.weapons
    }
    fn entity_type(&self) -> &'static str {
        "ship"
    }
    fn main_thruster_names(&self) -> &[String] {
        &self.propulsion.main_thruster_names
    }
    fn maneuvering_thruster_name(&self) -> &str {
        &self.propulsion.maneuvering_thruster
    }
}
