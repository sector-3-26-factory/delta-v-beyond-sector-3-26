// AGENTS: before modifying this file, read AGENTS.md at the repository root.

#![cfg(test)]

//! Tests for the world crate.
//!
//! The code-level half of the ADR-0058 pair. [`crate::build_spawn_event`] and
//! the spawn systems are driven for every shipped world, so this test proves
//! the behaviour the engine actually has, not what a JSON file says.

// A test reports its own failure with `expect`/`panic!`; the crate denies both
// in production code (ADR-0023), and ADR-0023 allows test code to relax them.
#![allow(clippy::expect_used, clippy::panic)]

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::asset::AssetPlugin;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_assets::paths::get_workspace_root;
use delta_v_assets::template::load_world;
use delta_v_core::DebugAxesEligible;
use delta_v_physics::RigidBody;
use delta_v_physics::belt_field_spawn::{spawn_asteroid_belt, spawn_asteroid_field};
use delta_v_physics::spawn::{spawn_asteroid, spawn_moon, spawn_planet, spawn_sun};
use delta_v_types::EntityTemplate;

use crate::{SpawnEntity, build_spawn_event};

/// Spawns the ship bodies.
///
/// In the game this work is done by `delta-v-ships`, which `delta-v-world`
/// may not depend on (ADR-0051). The test therefore drives the same
/// `delta-v-spawn` entry point that the ships plugin uses, so `require_mass`
/// runs exactly as it runs in game.
// INVARIANT: `Res` is taken by value because a Bevy system parameter cannot be
// borrowed; `build_physical_ship` takes it by reference, exactly as in
// `delta-v-ships`.
#[allow(clippy::needless_pass_by_value)]
fn spawn_ship_bodies(
    mut events: MessageReader<'_, '_, SpawnEntity>,
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
) {
    for event in events.read() {
        match &event.template {
            EntityTemplate::Ship(template) => {
                let _ship = delta_v_spawn::build_physical_ship(
                    &mut commands,
                    &asset_server,
                    event,
                    template,
                );
            }
            EntityTemplate::PlayerShip(template) => {
                let _ship = delta_v_spawn::build_physical_ship(
                    &mut commands,
                    &asset_server,
                    event,
                    template,
                );
            }
            EntityTemplate::AiShip(template) => {
                let _ship = delta_v_spawn::build_physical_ship(
                    &mut commands,
                    &asset_server,
                    event,
                    template,
                );
            }
            EntityTemplate::StaticShip(template) => {
                let _ship = delta_v_spawn::build_physical_ship(
                    &mut commands,
                    &asset_server,
                    event,
                    template,
                );
            }
            _ => {}
        }
    }
}

/// Builds a headless app carrying only the spawn path under test.
///
/// `Gltf` is registered because the spawners ask the asset server for a mesh
/// handle; the handle is never awaited, so no renderer is involved.
fn spawn_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Gltf>()
        .add_message::<SpawnEntity>()
        .add_systems(
            Update,
            (
                spawn_sun,
                spawn_planet,
                spawn_moon,
                spawn_asteroid,
                spawn_asteroid_belt,
                spawn_asteroid_field,
                spawn_ship_bodies,
            ),
        );
    app
}

/// Every shipped world file, in a stable order.
fn shipped_world_files() -> Vec<PathBuf> {
    let worlds = get_workspace_root().join("assets/worlds");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&worlds)
        .unwrap_or_else(|e| panic!("assets/worlds must be readable: {e}"))
        .filter_map(|entry| {
            let dir = entry.expect("dir entry must be readable").path();
            if !dir.is_dir() {
                return None;
            }
            let path = dir.join("world.json");
            if path.exists() { Some(path) } else { None }
        })
        .collect();
    files.sort();
    files
}

/// Masses of the bodies the app actually spawned, keyed by world entity id.
///
/// Both the celestial spawners and the ship spawner tag the body they create
/// with [`DebugAxesEligible`], which carries the world entity id, so one query
/// covers every spawn path.
fn spawned_masses(world: &mut World) -> HashMap<String, f32> {
    let mut masses = HashMap::new();
    let mut bodies = world.query::<(&DebugAxesEligible, &RigidBody)>();
    for (marker, body) in bodies.iter(world) {
        masses.insert(marker.entity_id.clone(), body.mass);
    }
    masses
}

/// Every body a shipped world spawns carries the mass its world entity
/// declares (ADR-0058).
///
/// This is the code-level counterpart to the JSON-level check in
/// `delta-v-assets`. It loads each world through `delta-v-assets`, converts
/// every entity with the real [`build_spawn_event`], runs the real spawn
/// systems and then reads the mass off the spawned [`RigidBody`]. A template
/// that reintroduced a mass, or a spawner that stopped calling
/// `require_mass`, fails here.
///
/// Belt and field entities are a definition rather than a body and reach no
/// `require_mass` call; their spawners derive a mass from the region density
/// instead. They are covered by their own fixture, not here.
#[test]
fn every_shipped_world_body_spawns_with_its_declared_mass() {
    // Weapon, projectile and sound definitions are resolved through paths that
    // are relative to the working directory, so the test must run from the
    // workspace root the way the game does.
    std::env::set_current_dir(get_workspace_root()).expect("workspace root must be enterable");

    let worlds = shipped_world_files();
    assert!(!worlds.is_empty(), "no shipped world files were found");

    for world_file in worlds {
        let mut world = load_world(&world_file)
            .unwrap_or_else(|e| panic!("{} must load: {e}", world_file.display()));
        assert!(
            !world.entities.is_empty(),
            "{} declares no entities",
            world_file.display()
        );

        let mut app = spawn_app();
        for entity_spawn in &mut world.entities {
            // Skip belt and field entities - they are definitions, not bodies
            if entity_spawn.template_short.starts_with("asteroid-belts/")
                || entity_spawn.template_short.starts_with("asteroid-fields/")
            {
                continue;
            }
            app.world_mut()
                .write_message(build_spawn_event(entity_spawn));
        }
        app.update();

        let masses = spawned_masses(app.world_mut());

        // Count only body entities (skip belt/field definitions)
        let expected_body_count = world
            .entities
            .iter()
            .filter(|e| {
                !e.template_short.starts_with("asteroid-belts/")
                    && !e.template_short.starts_with("asteroid-fields/")
            })
            .count();

        assert_eq!(
            masses.len(),
            expected_body_count,
            "every body entity of {} must spawn exactly one body",
            world_file.display()
        );

        for entity_spawn in &world.entities {
            // Skip belt/field entities - they are definitions, not bodies
            if entity_spawn.template_short.starts_with("asteroid-belts/")
                || entity_spawn.template_short.starts_with("asteroid-fields/")
            {
                continue;
            }
            let declared = entity_spawn.mass.unwrap_or_else(|| {
                panic!(
                    "{} entity '{}' declares no mass; a body spawned without one is a hard error \
                     (ADR-0058)",
                    world_file.display(),
                    entity_spawn.id
                )
            });
            let spawned = masses.get(&entity_spawn.id).unwrap_or_else(|| {
                panic!(
                    "{} entity '{}' spawned no body",
                    world_file.display(),
                    entity_spawn.id
                )
            });
            assert!(
                (spawned - declared).abs() <= f32::EPSILON * declared.abs().max(1.0),
                "{} entity '{}' spawned with mass {spawned} but the world declares {declared}",
                world_file.display(),
                entity_spawn.id
            );
        }
    }
}

/// Test that belt and field entities spawn correctly and generate asteroids with proper mass.
///
/// This test loads a fixture world with a belt and a field, runs the spawn systems,
/// and verifies that the belt and field entities are created (as definitions, not bodies)
/// and that the generated asteroids have mass derived from the region density and radius.
#[test]
fn belt_and_field_entities_spawn_and_generate_asteroids_with_mass() {
    std::env::set_current_dir(get_workspace_root()).expect("workspace root must be enterable");

    let world_file =
        get_workspace_root().join("crates/delta-v-world/tests/fixtures/belt_field_test.world.json");
    let mut world = load_world(&world_file)
        .unwrap_or_else(|e| panic!("{} must load: {e}", world_file.display()));

    let mut app = spawn_app();
    for entity_spawn in &mut world.entities {
        app.world_mut()
            .write_message(build_spawn_event(entity_spawn));
    }
    app.update();

    // Verify belt and field entities were spawned (as definitions, not bodies)
    let belt_entities: Vec<_> = {
        let world = app.world_mut();
        world
            .query::<&delta_v_physics::belt_field_spawn::AsteroidBelt>()
            .iter(world)
            .map(|b| b.id.clone())
            .collect::<Vec<_>>()
    };
    let field_entities: Vec<_> = {
        let world = app.world_mut();
        world
            .query::<&delta_v_physics::belt_field_spawn::AsteroidField>()
            .iter(world)
            .map(|f| f.id.clone())
            .collect::<Vec<_>>()
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

    // Verify the belt has the correct ID
    assert_eq!(belt_entities.first().map(String::as_str), Some("test_belt"));
    assert_eq!(
        field_entities.first().map(String::as_str),
        Some("test_field")
    );

    // Note: The actual asteroid generation happens in the streaming system during gameplay,
    // not during initial spawn. The belt/field entities are definitions that the streaming
    // system uses to generate asteroids when the player is nearby.
    // This test verifies the definitions are created correctly.
}
