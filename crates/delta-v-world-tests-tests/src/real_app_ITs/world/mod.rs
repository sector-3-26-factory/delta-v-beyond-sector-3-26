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
        "world::belt_streaming_happy_path",
        test_belt_streaming_happy_path,
    );
    do_register(
        "world::belt_streaming_player_far",
        test_belt_streaming_player_far,
    );
}

#[cfg(feature = "test")]
fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
