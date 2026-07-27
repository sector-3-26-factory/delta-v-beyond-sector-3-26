// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for ship spawning.
//!
//! See ADR-0021 (Testing strategy).

// Test code is allowed to use expect/unwrap/indexing per ADR-0023.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use bevy::app::App;
use bevy::asset::AssetPlugin;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_core::{PlayerShipEntity, SpawnEntity};
use delta_v_types::{
    AsteroidTemplate, BoundingBox, CameraDefinition, CockpitDefinition, CollisionShapeData,
    CollisionShapeType, EntityTemplate, PlayerShipTemplate, ShipCamerasTemplate,
    ShipPropulsionTemplate, StaticShipTemplate,
};

use crate::ship_templates::ShipPropulsionConfig;
use crate::spawn::spawn_ship;

/// Creates a minimal Bevy app for testing ship spawning.
///
/// Sets up the asset plugin (needed for glTF loading), registers
/// the `SpawnEntity` event, and adds the `spawn_ship` system.
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: "assets".to_string(),
        ..default()
    });
    app.add_message::<SpawnEntity>();
    app.init_asset::<Gltf>();
    app.add_systems(Update, spawn_ship);
    app
}

/// Builds a minimal `SpawnEntity` event for a player-controlled ship.
///
/// Uses a template `EntityTemplate` that matches the merged `player_controlled_ship`
/// structure (ship properties + cameras + `bounding_box` + `health`).
fn make_player_ship_event() -> SpawnEntity {
    let template = EntityTemplate::PlayerShip(PlayerShipTemplate {
        mass: 10_000.0,
        inertia_scale: 1.0,
        bounding_box: BoundingBox {
            min: Vec3::new(-1.0, -1.0, -1.0),
            max: Vec3::new(1.0, 1.0, 1.0),
        },
        collision_shape: CollisionShapeData {
            shape_type: CollisionShapeType::Box {
                half_extents: Vec3::new(1.0, 1.0, 1.0),
            },
            offset: Vec3::ZERO,
        },
        propulsion: ShipPropulsionTemplate {
            main_thruster_names: vec!["chemical-main".to_string()],
            maneuvering_thruster: "rcs-standard".to_string(),
        },
        cameras: ShipCamerasTemplate {
            cockpit: CameraDefinition {
                position: Vec3::new(0.0, 0.5, -0.2),
                target: Vec3::new(0.0, 0.5, -10.0),
                available: true,
            },
            drone: CameraDefinition {
                position: Vec3::new(0.0, 2.0, 5.0),
                target: Vec3::new(0.0, 0.0, 0.0),
                available: true,
            },
            rear: CameraDefinition {
                position: Vec3::new(0.0, 1.0, 4.0),
                target: Vec3::new(0.0, 1.0, -10.0),
                available: true,
            },
            front: CameraDefinition {
                position: Vec3::new(0.0, 0.5, -2.0),
                target: Vec3::new(0.0, 0.5, -10.0),
                available: true,
            },
            left: CameraDefinition {
                position: Vec3::new(-3.0, 1.0, 0.0),
                target: Vec3::new(10.0, 1.0, 0.0),
                available: true,
            },
            right: CameraDefinition {
                position: Vec3::new(3.0, 1.0, 0.0),
                target: Vec3::new(-10.0, 1.0, 0.0),
                available: true,
            },
            top: CameraDefinition {
                position: Vec3::new(0.0, 2.0, 0.0),
                target: Vec3::new(0.0, -10.0, 0.0),
                available: true,
            },
            bottom: CameraDefinition {
                position: Vec3::new(0.0, -2.0, 0.0),
                target: Vec3::new(0.0, 10.0, 0.0),
                available: true,
            },
        },
        weapons: vec![],
        max_weapons_count: 2,
        max_propulsions_count: 2,
        health: 100.0,
        cockpit: CockpitDefinition { stations: vec![] },
    });

    SpawnEntity::new(
        "test_player_ship".to_string(),
        template,
        "templates/ships/space-fighter-comrade1280/player_controlled_ship.json".to_string(),
        Vec3::new(1.0, 2.0, 3.0),
    )
}

/// A `SpawnEntity` event with an unknown `entity_type` should not panic
/// and should not spawn any entities.
#[test]
fn test_unknown_entity_type_does_not_spawn() {
    let mut app = create_test_app();

    let template = EntityTemplate::Asteroid(AsteroidTemplate {
        entity_type: "unknown_type".to_string(),
        mass: 1000.0,
        bounding_box: BoundingBox {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(1.0, 1.0, 1.0),
        },
        collision_shape: CollisionShapeData {
            shape_type: CollisionShapeType::Sphere { radius: 1.0 },
            offset: Vec3::ZERO,
        },
        is_gravity_source: false,
    });

    let event = SpawnEntity::new(
        "unknown_entity".to_string(),
        template,
        "templates/unknown/template.json".to_string(),
        Vec3::ZERO,
    );

    // Send the event.
    app.world_mut().write_message(event);
    // Run the spawn system.
    app.update();

    // No entities should have been spawned.
    let entity_count = app.world_mut().entities().len();
    // The spawn system should have logged a warning but not spawned anything.
    assert!(
        entity_count == 0,
        "unknown entity_type should not spawn any entities, found {entity_count}"
    );
}

/// A `SpawnEntity` event with `npc_ship` type should not panic
/// (NPC ships are not yet implemented).
#[test]
fn test_npc_ship_does_not_panic() {
    let mut app = create_test_app();

    let template = EntityTemplate::StaticShip(StaticShipTemplate {
        mass: 10_000.0,
        inertia_scale: 1.0,
        bounding_box: BoundingBox {
            min: Vec3::new(-1.0, -1.0, -1.0),
            max: Vec3::new(1.0, 1.0, 1.0),
        },
        collision_shape: CollisionShapeData {
            shape_type: CollisionShapeType::Box {
                half_extents: Vec3::new(1.0, 1.0, 1.0),
            },
            offset: Vec3::ZERO,
        },
        propulsion: ShipPropulsionTemplate {
            main_thruster_names: vec!["chemical-main".to_string()],
            maneuvering_thruster: "rcs-standard".to_string(),
        },
        weapons: vec![],
        max_weapons_count: 2,
        max_propulsions_count: 2,
        health: 100.0,
    });

    let event = SpawnEntity::new(
        "npc_ship".to_string(),
        template,
        "templates/ships/npc/template.json".to_string(),
        Vec3::ZERO,
    );

    app.world_mut().write_message(event);
    // Should not panic — NPC ships log a warning.
    app.update();
}

/// Verifies that the `SpawnEntity` event can be created with the
/// expected fields and that the builder methods work correctly.
#[test]
fn test_spawn_event_builder() {
    let template = EntityTemplate::Asteroid(AsteroidTemplate {
        entity_type: "test".to_string(),
        mass: 1000.0,
        bounding_box: BoundingBox {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(1.0, 1.0, 1.0),
        },
        collision_shape: CollisionShapeData {
            shape_type: CollisionShapeType::Sphere { radius: 1.0 },
            offset: Vec3::ZERO,
        },
        is_gravity_source: false,
    });

    let event = SpawnEntity::new(
        "test_ship".to_string(),
        template,
        "templates/test/template.json".to_string(),
        Vec3::new(10.0, 20.0, 30.0),
    )
    .with_rotation(Quat::from_xyzw(0.0, 1.0, 0.0, 0.0))
    .with_scale(Vec3::new(2.0, 2.0, 2.0));

    assert_eq!(event.id, "test_ship");
    assert_eq!(event.position, Vec3::new(10.0, 20.0, 30.0));
    assert_eq!(event.rotation, Quat::from_xyzw(0.0, 1.0, 0.0, 0.0));
    assert_eq!(event.scale, Vec3::new(2.0, 2.0, 2.0));
}

/// Verifies that a `player_controlled_ship` event with a valid template
/// produces the expected `PlayerShipEntity` and `ShipPropulsionConfig`
/// resources after the spawn system runs.
///
/// Note: This test does NOT verify the glTF mesh loading (which requires
/// the actual mesh file on disk). It verifies that the spawn system
/// processes the event and creates the expected ECS entities and resources.
#[test]
fn test_player_ship_spawn_creates_resources() {
    let mut app = create_test_app();

    let event = make_player_ship_event();
    app.world_mut().write_message(event);

    // Run the system — it will attempt to load the glTF mesh, which
    // will fail silently (the mesh file may not exist in the test
    // environment), but the entity and resources should still be created.
    app.update();

    // Verify that the PlayerShipEntity resource was inserted.
    let player_entity = app.world().get_resource::<PlayerShipEntity>();
    assert!(
        player_entity.is_some(),
        "PlayerShipEntity resource should be inserted after spawning"
    );

    // Verify that the ShipPropulsionConfig resource was inserted.
    let propulsion = app.world().get_resource::<ShipPropulsionConfig>();
    assert!(
        propulsion.is_some(),
        "ShipPropulsionConfig resource should be inserted after spawning"
    );

    // Verify the propulsion values match the template.
    // Use approximate comparison for f32 values (clippy::float_cmp).
    let propulsion = propulsion.unwrap();
    let eps = 0.001;
    assert!(
        (propulsion.max_forward_thrust - 100_000.0).abs() < eps,
        "max_forward_thrust should be 100000, got {}",
        propulsion.max_forward_thrust
    );
    assert!(
        (propulsion.max_backward_thrust - 40_000.0).abs() < eps,
        "max_backward_thrust should be 40000, got {}",
        propulsion.max_backward_thrust
    );
    assert!(
        (propulsion.max_torque - 50_000.0).abs() < eps,
        "max_torque should be 50000, got {}",
        propulsion.max_torque
    );
    assert!(
        (propulsion.max_strafe_thrust - 50_000.0).abs() < eps,
        "max_strafe_thrust should be 50000, got {}",
        propulsion.max_strafe_thrust
    );
    assert_eq!(propulsion.active_main_thruster_index, 0);
}

/// Verifies that the player ship entity is spawned at the correct position
/// from the event.
#[test]
fn test_player_ship_spawn_position() {
    let mut app = create_test_app();

    let event = make_player_ship_event();
    let expected_pos = event.position;
    app.world_mut().write_message(event);
    app.update();

    // The PlayerShipEntity resource should reference an entity at the
    // expected position.
    let player_entity = app.world().get_resource::<PlayerShipEntity>().unwrap();
    let entity_ref = app.world().get_entity(player_entity.0);
    assert!(
        entity_ref.is_ok(),
        "player ship entity should exist in the world"
    );

    let transform = entity_ref.unwrap().get::<Transform>();
    assert!(
        transform.is_some(),
        "player ship entity should have a Transform"
    );
    assert_eq!(transform.unwrap().translation, expected_pos);
}
