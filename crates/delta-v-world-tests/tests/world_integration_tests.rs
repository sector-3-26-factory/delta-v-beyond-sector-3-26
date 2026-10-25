// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Integration tests for world plugin — real world loading pipeline tests.
//!
//! Registered as a `[[test]]` target with `harness = false` in Cargo.toml so
//! that `cargo test --test world_integration_tests` runs `main()` on the **main
//! thread**, which is required by Bevy's `WinitPlugin` (see
//! plans/testframework.md step 8).
//!
//!   cargo test --test `world_integration_tests` -p delta-v-world
//!
//! See ADR-0019 (Asset pipeline), ADR-0020 (Save and load format), and
//! ADR-0060 (Test framework single source of truth).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    clippy::too_many_lines,
    clippy::doc_markdown,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_app::{AppConfig, build_app};
use delta_v_core::AppState;
use delta_v_test_support::{check, run_harness};
use delta_v_world::resources::WorldDefResource;

/// Test: world loading pipeline — loads a world through the full app stack,
/// verifies `WorldDefResource` is inserted, state transitions to
/// `SpawningEntities` (or beyond), and the world definition is correctly loaded.
fn test_world_loading_pipeline(app: &mut App) {
    // Verify WorldDefResource was inserted
    let world_def = app.world().resource::<WorldDefResource>();
    check(
        world_def.0.name == "Test World",
        "WorldDefResource should contain the loaded world name",
    );

    // Verify state transitioned to SpawningEntities or beyond (InGame, etc.)
    // After 60 fixed updates in make_app, the state may have moved past
    // SpawningEntities to InGame. We just verify it's at least SpawningEntities.
    let state = app.world().resource::<State<AppState>>();
    let current_state = state.get();
    let state_ok =
        *current_state == AppState::SpawningEntities || *current_state == AppState::InGame;
    check(
        state_ok,
        "AppState should be SpawningEntities or InGame after world load",
    );

    // Verify exactly one entity in the world definition
    check(
        world_def.0.entities.len() == 1,
        "WorldDef should contain exactly one entity from test.world.json",
    );
}

/// Test: entity validation — loads belt_field_test.world.json and verifies all
/// 3 entities have correct templates, positions, and orbital parameters.
fn test_entity_validation(app: &mut App) {
    let world_def = app.world().resource::<WorldDefResource>();
    check(
        world_def.0.name == "Belt Field Test World",
        "WorldDefResource should contain the loaded world name",
    );

    // Verify exactly 4 entities in the world definition (sun + ship + belt + field)
    check(
        world_def.0.entities.len() == 4,
        "WorldDef should contain exactly 4 entities from belt_field_test.world.json",
    );

    // Verify entity 1: player ship (debug-ship-cube)
    let ship_entity = world_def.0.entities.iter().find(|e| e.id == "test_ship");
    check(
        ship_entity.is_some(),
        "Entity test_ship should exist in WorldDef",
    );
    let ship_entity = ship_entity.unwrap();
    check(
        ship_entity.template_short == "ships/debug-ship-cube",
        "ship template_short should match",
    );
    check(
        ship_entity.position.x == 404_000_000_000.0
            && ship_entity.position.y == 0.0
            && ship_entity.position.z == 0.0,
        "ship position should match world JSON",
    );
    check(
        ship_entity.mass == Some(10_000.0),
        "ship mass should match world JSON",
    );
    check(
        ship_entity.player_controlled,
        "ship should be player_controlled",
    );

    // Verify entity 2: asteroid belt (asteroid-belts/main)
    let belt_entity = world_def.0.entities.iter().find(|e| e.id == "test_belt");
    check(
        belt_entity.is_some(),
        "Entity test_belt should exist in WorldDef",
    );
    let belt_entity = belt_entity.unwrap();
    check(
        belt_entity.template_short == "asteroid-belts/main",
        "belt template_short should match",
    );
    check(
        belt_entity.position.x == 0.0
            && belt_entity.position.y == 0.0
            && belt_entity.position.z == 0.0,
        "belt position should be at origin",
    );
    check(
        belt_entity.orbital_parameters.is_some(),
        "belt should have orbital parameters",
    );
    let orbital = belt_entity.orbital_parameters.as_ref().unwrap();
    check(
        orbital.orbital_parent == "sun",
        "belt orbital_parent should be 'sun'",
    );
    check(
        (orbital.orbital_distance - 404_000_000_000.0).abs() < f32::EPSILON,
        "belt orbital_distance should match",
    );
    check(
        belt_entity.asteroid_belt_parameters.is_some(),
        "belt should have asteroid_belt_parameters",
    );

    // Verify entity 3: asteroid field (asteroid-fields/minable-pocket)
    let field_entity = world_def.0.entities.iter().find(|e| e.id == "test_field");
    check(
        field_entity.is_some(),
        "Entity test_field should exist in WorldDef",
    );
    let field_entity = field_entity.unwrap();
    check(
        field_entity.template_short == "asteroid-fields/minable-pocket",
        "field template_short should match",
    );
    check(
        field_entity.position.x == 350_000_000_000.0
            && field_entity.position.y == 0.0
            && field_entity.position.z == 0.0,
        "field position should match world JSON",
    );
    check(
        field_entity.orbital_parameters.is_some(),
        "field should have orbital parameters",
    );
    let orbital = field_entity.orbital_parameters.as_ref().unwrap();
    check(
        orbital.orbital_parent == "sun",
        "field orbital_parent should be 'sun'",
    );
    check(
        (orbital.orbital_distance - 350_000_000_000.0).abs() < f32::EPSILON,
        "field orbital_distance should match",
    );
    check(
        field_entity.asteroid_field_parameters.is_some(),
        "field should have asteroid_field_parameters",
    );
}

/// Test: player-controlled validation — loads a world with exactly one
/// player-controlled entity and verifies it passes (the happy path).
fn test_player_controlled_validation_one(app: &mut App) {
    // This test uses the test.world.json fixture which has exactly one
    // player_controlled entity. The validation happens during world load in
    // load_world_system. If the validation passes, we're good.
    let world_def = app.world().resource::<WorldDefResource>();
    let player_count = world_def
        .0
        .entities
        .iter()
        .filter(|e| e.player_controlled)
        .count();
    check(
        player_count == 1,
        "test.world.json should have exactly one player_controlled entity",
    );
}

/// Build the test `App` via `build_app` (Single Source of Truth per ADR-0060).
/// Uses the `test` world fixture (simple, single player ship).
fn build_test_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-world/tests/fixtures/assets".to_string(),
        world_name: Some("test".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
}

/// Build the test `App` via `build_app` for the belt/field fixture.
fn build_belt_field_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-world/tests/fixtures/assets".to_string(),
        world_name: Some("belt_field_test".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
}

/// Entry point — runs on the main thread so `WinitPlugin` can initialize.
fn main() {
    // Test 1: world loading pipeline with simple test fixture
    let mut app1 = delta_v_test_support::make_app(60, build_test_app);
    run_harness(
        &mut app1,
        &[
            ("world_loading_pipeline", test_world_loading_pipeline),
            (
                "player_controlled_validation_one",
                test_player_controlled_validation_one,
            ),
        ],
    );

    // Test 2: entity validation with belt/field fixture
    let mut app2 = delta_v_test_support::make_app(60, build_belt_field_app);
    run_harness(&mut app2, &[("entity_validation", test_entity_validation)]);
}
