// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Integration tests for world plugin — belt/field fixture tests.
//!
//! Registered as a `[[test]]` target with `harness = false` in Cargo.toml so
//! that `cargo test --test world_belt_field_tests` runs `main()` on the **main
//! thread**, which is required by Bevy's `WinitPlugin` (see
//! plans/testframework.md step 8).
//!
//!   cargo test --test `world_belt_field_tests` -p delta-v-world
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
use delta_v_test_framework::{check, run_harness};
use delta_v_world::resources::WorldDefResource;

/// Test: entity validation — loads belt_field_test.world.json and verifies all
/// 3 entities have correct templates, positions, and orbital parameters.
fn test_entity_validation(app: &mut App) {
    let world_def = app.world().resource::<WorldDefResource>();
    check(
        world_def.0.name == "Belt Field Test World",
        "WorldDefResource should contain the loaded world name",
    );

    // Verify state transitioned to SpawningEntities or beyond
    let state = app.world().resource::<State<AppState>>();
    let current_state = state.get();
    let state_ok =
        *current_state == AppState::SpawningEntities || *current_state == AppState::InGame;
    check(
        state_ok,
        "AppState should be SpawningEntities or InGame after world load",
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

/// Build the test `App` via `build_app` for the belt/field fixture.
fn build_belt_field_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-world/tests/fixtures/assets".to_string(),
        world_name: Some("belt_field_test".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
}

/// Test: belt sectors are loaded when the player ship is on the belt's
/// orbital ring (the happy path).
///
/// This exercises the full pipeline: world JSON → template loading →
/// spawn systems → streaming system. The player ship is at 404M km
/// (on the belt's orbital ring), so the streaming system should load
/// 3 belt sectors (index-1, index, index+1).
fn test_belt_streaming_happy_path(app: &mut App) {
    use delta_v_physics::belt_field_streaming::LoadedSectors;

    // Run several fixed updates so the streaming system has a chance to
    // run. The streaming system runs in `FixedUpdate` during
    // `AppState::InGame`.
    delta_v_test_framework::run_fixed_updates(app, 5);

    let loaded = app.world().resource::<LoadedSectors>();
    check(
        !loaded.belt_sectors.is_empty(),
        "belt sectors should be loaded when player is on the belt ring",
    );
    check(
        loaded.belt_sectors.len() == 3,
        "exactly 3 belt sectors should be loaded (index-1, index, index+1)",
    );
    for (id, _) in &loaded.belt_sectors {
        check(
            id == "test_belt",
            "loaded sector should belong to test_belt",
        );
    }
}

/// Test: belt sectors are NOT loaded when the player ship is far from
/// the belt (outside streaming distance).
///
/// The player ship is moved to 300M km (well outside the belt's
/// streaming distance of 100M km), so no sectors should load.
fn test_belt_streaming_player_far(app: &mut App) {
    use delta_v_core::PlayerShipEntity;
    use delta_v_physics::belt_field_streaming::LoadedSectors;

    // Move the player ship far from the belt (300M km, outside
    // streaming distance of 100M km).
    let player = app.world().resource::<PlayerShipEntity>().0;
    if let Ok(mut entity) = app.world_mut().get_entity_mut(player) {
        let pos = Vec3::new(3.0e11, 0.0, 0.0);
        entity.insert(GlobalTransform::from_translation(pos));
        entity.insert(Transform::from_translation(pos));
    }

    // Run several fixed updates so the streaming system can evaluate.
    delta_v_test_framework::run_fixed_updates(app, 5);

    let loaded = app.world().resource::<LoadedSectors>();
    check(
        loaded.belt_sectors.is_empty(),
        "no belt sectors should be loaded when player is far from the belt",
    );
}

/// Entry point — runs on the main thread so `WinitPlugin` can initialize.
fn main() {
    let mut app = delta_v_test_framework::make_app(60, build_belt_field_app);
    run_harness(
        &mut app,
        &[
            ("entity_validation", test_entity_validation),
            ("belt_streaming_happy_path", test_belt_streaming_happy_path),
            ("belt_streaming_player_far", test_belt_streaming_player_far),
        ],
    );
}
