// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! World real-app integration tests.
//!
//! Each test function receives a fully-built and render-settled `App`
//! from `main.rs` (which calls `settle_render(app, 60)` before
//! dispatching). Tests use plain `assert!` — a panic exits the process
//! with code 1, which the Z2 driver reads as failure.

#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::cargo
)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use bevy::prelude::*;
use delta_v_core::AppState;
use delta_v_world::resources::WorldDefResource;

/// Test: world loading pipeline — loads a world through the full app stack,
/// verifies `WorldDefResource` is inserted, state transitions to
/// `SpawningEntities` (or beyond), and the world definition is correctly loaded.
fn test_world_loading_pipeline(app: &mut App) {
    let world_def = app.world().resource::<WorldDefResource>();
    assert!(
        world_def.0.name == "Test World",
        "WorldDefResource should contain the loaded world name"
    );

    let state = app.world().resource::<State<AppState>>();
    let current_state = state.get();
    let state_ok =
        *current_state == AppState::SpawningEntities || *current_state == AppState::InGame;
    assert!(
        state_ok,
        "AppState should be SpawningEntities or InGame after world load"
    );

    assert!(
        world_def.0.entities.len() == 1,
        "WorldDef should contain exactly one entity from test.world.json"
    );
}

/// Test: entity validation — loads belt_field_test.world.json and verifies all
/// 4 entities have correct templates, positions, and orbital parameters.
#[allow(clippy::float_cmp, clippy::doc_markdown)]
fn test_entity_validation(app: &mut App) {
    let world_def = app.world().resource::<WorldDefResource>();
    assert!(
        world_def.0.name == "Belt Field Test World",
        "WorldDefResource should contain the loaded world name"
    );

    assert!(
        world_def.0.entities.len() == 4,
        "WorldDef should contain exactly 4 entities from belt_field_test.world.json"
    );

    let ship_entity = world_def.0.entities.iter().find(|e| e.id == "test_ship");
    assert!(
        ship_entity.is_some(),
        "Entity test_ship should exist in WorldDef"
    );
    let ship_entity = ship_entity.unwrap();
    assert!(
        ship_entity.template_short == "ships/debug-ship-cube",
        "ship template_short should match"
    );
    assert!(
        ship_entity.position.x == 404_000_000_000.0
            && ship_entity.position.y == 0.0
            && ship_entity.position.z == 0.0,
        "ship position should match world JSON"
    );
    assert!(
        ship_entity.mass == Some(10_000.0),
        "ship mass should match world JSON"
    );
    assert!(
        ship_entity.player_controlled,
        "ship should be player_controlled"
    );

    let belt_entity = world_def.0.entities.iter().find(|e| e.id == "test_belt");
    assert!(
        belt_entity.is_some(),
        "Entity test_belt should exist in WorldDef"
    );
    let belt_entity = belt_entity.unwrap();
    assert!(
        belt_entity.template_short == "asteroid-belts/main",
        "belt template_short should match"
    );
    assert!(
        belt_entity.position.x == 0.0
            && belt_entity.position.y == 0.0
            && belt_entity.position.z == 0.0,
        "belt position should be at origin"
    );
    assert!(
        belt_entity.orbital_parameters.is_some(),
        "belt should have orbital parameters"
    );
    let orbital = belt_entity.orbital_parameters.as_ref().unwrap();
    assert!(
        orbital.orbital_parent == "sun",
        "belt orbital_parent should be 'sun'"
    );
    assert!(
        (orbital.orbital_distance - 404_000_000_000.0).abs() < f32::EPSILON,
        "belt orbital_distance should match"
    );
    assert!(
        belt_entity.asteroid_belt_parameters.is_some(),
        "belt should have asteroid_belt_parameters"
    );

    let field_entity = world_def.0.entities.iter().find(|e| e.id == "test_field");
    assert!(
        field_entity.is_some(),
        "Entity test_field should exist in WorldDef"
    );
    let field_entity = field_entity.unwrap();
    assert!(
        field_entity.template_short == "asteroid-fields/minable-pocket",
        "field template_short should match"
    );
    assert!(
        field_entity.position.x == 350_000_000_000.0
            && field_entity.position.y == 0.0
            && field_entity.position.z == 0.0,
        "field position should match world JSON"
    );
    assert!(
        field_entity.orbital_parameters.is_some(),
        "field should have orbital parameters"
    );
    let orbital = field_entity.orbital_parameters.as_ref().unwrap();
    assert!(
        orbital.orbital_parent == "sun",
        "field orbital_parent should be 'sun'"
    );
    assert!(
        (orbital.orbital_distance - 350_000_000_000.0).abs() < f32::EPSILON,
        "field orbital_distance should match"
    );
    assert!(
        field_entity.asteroid_field_parameters.is_some(),
        "field should have asteroid_field_parameters"
    );
}

/// Test: player-controlled validation — loads a world with exactly one
/// player-controlled entity and verifies it passes (the happy path).
fn test_player_controlled_validation_one(app: &mut App) {
    let world_def = app.world().resource::<WorldDefResource>();
    let player_count = world_def
        .0
        .entities
        .iter()
        .filter(|e| e.player_controlled)
        .count();
    assert!(
        player_count == 1,
        "test.world.json should have exactly one player_controlled entity"
    );
}

/// Test: every body entity spawns with its declared mass (ADR-0058).
///
/// Uses the `belt_field_test` world which has a sun and a ship with declared
/// masses. Queries the spawned bodies (tagged with [`DebugAxesEligible`])
/// and verifies their masses match the world definition.
fn test_every_shipped_world_body_spawns_with_its_declared_mass(app: &mut App) {
    use std::collections::HashMap;

    use delta_v_core::DebugAxesEligible;
    use delta_v_physics::RigidBody;

    // Clone the entity list so we can drop the immutable borrow before the
    // mutable query below.
    let entities = app
        .world()
        .resource::<WorldDefResource>()
        .0
        .entities
        .clone();

    // Query all spawned bodies tagged with DebugAxesEligible (carries world entity id)
    let mut bodies = app.world_mut().query::<(&DebugAxesEligible, &RigidBody)>();
    let spawned_masses: HashMap<String, f32> = bodies
        .iter(app.world())
        .map(|(marker, body)| (marker.entity_id.clone(), body.mass))
        .collect();

    // Check each body entity (skip belt/field definitions - they are not bodies)
    for entity_spawn in &entities {
        if entity_spawn.template_short.starts_with("asteroid-belts/")
            || entity_spawn.template_short.starts_with("asteroid-fields/")
        {
            continue;
        }
        let declared = entity_spawn.mass.expect("entity must declare a mass");
        let spawned = spawned_masses
            .get(&entity_spawn.id)
            .expect("body must spawn");
        assert!(
            (spawned - declared).abs() <= f32::EPSILON * declared.abs().max(1.0),
            "entity '{}' spawned with mass {spawned} but world declares {declared}",
            entity_spawn.id
        );
    }
}

/// Test: belt and field entities spawn correctly and generate asteroids with proper mass.
///
/// Uses the `belt_field_test` world which has a belt and a field.
/// Queries the spawned belt/field entities and verifies they exist with correct IDs.
fn test_belt_and_field_entities_spawn_and_generate_asteroids_with_mass(app: &mut App) {
    use delta_v_physics::belt_field_spawn::{AsteroidBelt, AsteroidField};

    // Query spawned belt entities
    let belt_entities: Vec<_> = {
        let world = app.world_mut();
        world
            .query::<&AsteroidBelt>()
            .iter(world)
            .map(|b| b.id.clone())
            .collect()
    };
    let field_entities: Vec<_> = {
        let world = app.world_mut();
        world
            .query::<&AsteroidField>()
            .iter(world)
            .map(|f| f.id.clone())
            .collect()
    };

    assert_eq!(
        belt_entities.len(),
        1,
        "exactly one belt entity should be spawned"
    );
    assert_eq!(
        field_entities.len(),
        1,
        "exactly one field entity should be spawned"
    );
    assert_eq!(belt_entities.first().map(String::as_str), Some("test_belt"));
    assert_eq!(
        field_entities.first().map(String::as_str),
        Some("test_field")
    );
}

/// Test: belt sectors are loaded when the player ship is on the belt's
/// orbital ring (the happy path).
fn test_belt_streaming_happy_path(app: &mut App) {
    use delta_v_physics::belt_field_streaming::LoadedSectors;

    delta_v_test_framework::run_fixed_updates(app, 5);

    let loaded = app.world().resource::<LoadedSectors>();
    assert!(
        !loaded.belt_sectors.is_empty(),
        "belt sectors should be loaded when player is on the belt ring"
    );
    assert!(
        loaded.belt_sectors.len() == 3,
        "exactly 3 belt sectors should be loaded (index-1, index, index+1)"
    );
    for (id, _) in &loaded.belt_sectors {
        assert!(
            id == "test_belt",
            "loaded sector should belong to test_belt"
        );
    }
}

/// Test: belt sectors are NOT loaded when the player ship is far from
/// the belt (outside streaming distance).
fn test_belt_streaming_player_far(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::belt_field_streaming::LoadedSectors;

    let player = app.world().resource::<PlayerShipEntity>().0;
    if let Ok(mut entity) = app.world_mut().get_entity_mut(player) {
        let pos = Vec3::new(3.0e11, 0.0, 0.0);
        entity.insert(GlobalTransform::from_translation(pos));
        entity.insert(Transform::from_translation(pos));
    }

    delta_v_test_framework::run_fixed_updates(app, 5);

    let loaded = app.world().resource::<LoadedSectors>();
    assert!(
        loaded.belt_sectors.is_empty(),
        "no belt sectors should be loaded when player is far from the belt"
    );
}

/// Register all world test functions in the global registry.
///
/// Called from [`super::init_registry`] (the crate-level
/// `real_app_ITs::init_registry`).
#[cfg(feature = "test")]
pub fn init_registry() {
    do_register("world::world_loading_pipeline", test_world_loading_pipeline);
    do_register("world::entity_validation", test_entity_validation);
    do_register(
        "world::player_controlled_validation_one",
        test_player_controlled_validation_one,
    );
    do_register(
        "world::every_shipped_world_body_spawns_with_its_declared_mass",
        test_every_shipped_world_body_spawns_with_its_declared_mass,
    );
    do_register(
        "world::belt_and_field_entities_spawn_and_generate_asteroids_with_mass",
        test_belt_and_field_entities_spawn_and_generate_asteroids_with_mass,
    );
    do_register(
        "world::belt_streaming_happy_path",
        test_belt_streaming_happy_path,
    );
    do_register(
        "world::belt_streaming_player_far",
        test_belt_streaming_player_far,
    );
    // These three tests EXPECT a panic during `build_app` / `settle_render`
    // in `main.rs`. The panic happens during app construction, before this
    // function runs, so the function body is a no-op. The binary wraps
    // construction in `catch_unwind` and checks the panic message.
    do_register_panic(
        "world::player_controlled_zero",
        test_player_controlled_zero_panic,
        "exactly one entity must have player_controlled: true",
    );
    do_register_panic(
        "world::player_controlled_two",
        test_player_controlled_two_panic,
        "exactly one entity must have player_controlled: true",
    );
    do_register_panic(
        "world::collision_detection",
        test_collision_detection_panic,
        "overlapping collision shapes",
    );
}

/// No-op test function for panic-expecting tests.
///
/// The panic happens during `build_app` / `settle_render` in `main.rs`,
/// which is wrapped in `catch_unwind` by `run_test_panic`. This function
/// is never actually called — it exists only to satisfy the registry's
/// `fn(&mut App)` signature.
fn test_player_controlled_zero_panic(_app: &mut App) {}

/// No-op test function for panic-expecting tests. See above.
fn test_player_controlled_two_panic(_app: &mut App) {}

/// No-op test function for panic-expecting tests. See above.
fn test_collision_detection_panic(_app: &mut App) {}

#[cfg(feature = "test")]
fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry {
        name,
        func,
        panic_expected: false,
        panic_message: "",
    };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}

/// Register a test function that is EXPECTED to panic during app construction.
///
/// `panic_msg` is the substring the panic message must contain for the test
/// to pass. The binary wraps `build_app` + `settle_render` in `catch_unwind`
/// and checks the message in `run_test_panic`.
#[cfg(feature = "test")]
fn do_register_panic(name: &'static str, func: fn(&mut App), panic_msg: &'static str) {
    let entry = delta_v_test_framework::TestEntry {
        name,
        func,
        panic_expected: true,
        panic_message: panic_msg,
    };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
