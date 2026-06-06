// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Sectors, worlds, and hyperspace gates.
//!
//! This crate owns the world definition format and loader. It reads
//! `*.world.json` files, validates them against their JSON Schema
//! (ADR-0012), fills schema defaults via `delta-v-json` (ADR-0038),
//! and inserts a [`WorldDefResource`] during
//! [`AppState::LoadingWorld`] (ADR-0018).
//!
//! See ADR-0019 (Asset pipeline) and ADR-0020 (Save and load format).

#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![warn(clippy::all, clippy::pedantic)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

pub mod asteroid_spawner;
pub mod error;
pub mod events;
pub mod loader;
pub mod resources;
pub mod template_loader;
pub mod world_def;

#[cfg(test)]
#[path = "loader_tests.rs"]
mod loader_tests;

#[cfg(test)]
#[path = "template_loader_tests.rs"]
mod template_loader_tests;

pub use error::WorldError;
pub use events::SpawnEntity;
pub use resources::WorldDefResource;
pub use world_def::WorldDef;

use asteroid_spawner::{attach_asteroid_meshes, spawn_asteroid_system};
use bevy::prelude::*;
use delta_v_core::{AppState, WorldSpawnSet};
use serde_json::Value;

use crate::loader::load_default_world;
use crate::template_loader::load_template;
use crate::template_loader::resolve_template_path;
use world_def::EntitySpawn;

/// World plugin: loads and validates the world definition.
///
/// Systems run in [`AppState::LoadingWorld`]. On success the plugin
/// inserts [`WorldDefResource`] and transitions to [`AppState::SpawningEntities`].
/// On failure the application panics with a descriptive message
/// (ADR-0013).
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEntity>()
            .add_systems(OnEnter(AppState::LoadingWorld), load_world_system)
            // Asteroid spawning runs in Update during SpawningEntities
            .add_systems(
                Update,
                spawn_asteroid_system
                    .in_set(WorldSpawnSet::SpawnAsteroids)
                    .run_if(in_state(AppState::SpawningEntities)),
            )
            // Attach asteroid meshes in InGame once glTF assets are loaded.
            .add_systems(
                Update,
                attach_asteroid_meshes.run_if(in_state(AppState::InGame)),
            );
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Loads the default world definition and emits `SpawnEntity` events.
///
/// Reads the world JSON, validates it, inserts [`WorldDefResource`],
/// and emits a `SpawnEntity` event for each entity in the world.
///
/// The `entity_type` is derived from the template's `entity_type` field
/// (per ADR-0038, the template declares its own type).
///
/// Transitions to [`AppState::SpawningEntities`] so domain plugins can
/// spawn entities in dependency order (per ADR-0038).
///
/// # Panics
///
/// Panics if the world file or any template file cannot be loaded or validated.
/// This is intentional per ADR-0013 (no silent fallbacks).
fn load_world_system(
    mut commands: Commands<'_, '_>,
    mut events: EventWriter<'_, SpawnEntity>,
    mut next: ResMut<'_, NextState<AppState>>,
) {
    // INVARIANT: a missing or invalid world file is a hard startup
    // error (ADR-0013). The panic is intentional; no recovery is possible.
    #[allow(clippy::panic)]
    let world = load_default_world().unwrap_or_else(|e| {
        panic!("fatal: failed to load world: {e}");
    });
    log::info!("world loaded: {}", world.name);

    // Validate: exactly one entity must have player_controlled: true
    let player_controlled_count = world
        .entities
        .iter()
        .filter(|e| e.player_controlled)
        .count();
    assert!(
        player_controlled_count == 1,
        "fatal: exactly one entity must have player_controlled: true, found {player_controlled_count}"
    );

    // Emit SpawnEntity events for each entity in the world.
    // Per ADR-0038, domain plugins listen for these events and spawn
    // entities based on `entity_type`, in dependency order via WorldSpawnSet.
    for entity_spawn in &world.entities {
        let spawn_event = build_spawn_event(entity_spawn);
        events.send(spawn_event);
    }

    commands.insert_resource(WorldDefResource(world));
    next.set(AppState::SpawningEntities);
}

/// Builds a `SpawnEntity` event for a world entity spawn definition.
fn build_spawn_event(entity_spawn: &EntitySpawn) -> SpawnEntity {
    let template_short = &entity_spawn.template;

    // Determine entity_type and load template.
    // For player_controlled ships: load player_controlled_ship.json and merge with ship.json.
    // For asteroids: load asteroid.json directly.
    // For other ships: load ship.json directly.
    let (entity_type, template_path, merged_template, mesh_template_path) =
        if entity_spawn.player_controlled {
            load_player_controlled_ship(template_short)
        } else if template_short.starts_with("asteroids/") {
            load_asteroid(template_short)
        } else {
            load_ship(template_short)
        };

    let pos = Vec3::new(
        entity_spawn.position.x,
        entity_spawn.position.y,
        entity_spawn.position.z,
    );
    let rot = Quat::from_xyzw(
        entity_spawn.rotation.x,
        entity_spawn.rotation.y,
        entity_spawn.rotation.z,
        entity_spawn.rotation.w,
    );
    let scale = Vec3::new(
        entity_spawn.scale.x,
        entity_spawn.scale.y,
        entity_spawn.scale.z,
    );

    SpawnEntity::new(
        entity_spawn.id.clone(),
        entity_type,
        merged_template,
        template_path,
        mesh_template_path,
        pos,
    )
    .with_rotation(rot)
    .with_scale(scale)
}

/// Loads a player-controlled ship template by merging `player_controlled_ship.json` with `ship.json`.
fn load_player_controlled_ship(template_short: &str) -> (String, String, Value, String) {
    let entity_type = "player_controlled_ship";
    let template_path = resolve_template_path(template_short, entity_type);
    #[allow(clippy::panic)]
    let template = match load_template(&template_path, entity_type) {
        Ok(t) => t,
        Err(e) => panic!("fatal: failed to load template '{template_short}': {e}"),
    };

    // Load the co-located ship.json for mesh and base properties
    let ship_template_path = resolve_template_path(template_short, "ship");
    #[allow(clippy::panic)]
    let ship_template = match load_template(&ship_template_path, "ship") {
        Ok(t) => t,
        Err(e) => {
            panic!("fatal: failed to load co-located ship template '{template_short}': {e}")
        }
    };

    // Merge: start with ship template, overlay from player_controlled_ship
    let mut merged = ship_template;
    if let (Some(merged_obj), Some(player_obj)) = (merged.as_object_mut(), template.as_object()) {
        for (key, value) in player_obj {
            // Skip entity_type (we keep it as player_controlled_ship)
            if key != "entity_type" {
                merged_obj.insert(key.clone(), value.clone());
            }
        }
        // Ensure entity_type is player_controlled_ship
        merged_obj.insert(
            "entity_type".to_string(),
            serde_json::Value::String("player_controlled_ship".to_string()),
        );
    }

    (
        entity_type.to_string(),
        template_path,
        merged,
        ship_template_path,
    )
}

/// Loads an asteroid template.
fn load_asteroid(template_short: &str) -> (String, String, Value, String) {
    let entity_type = "asteroid";
    let template_path = resolve_template_path(template_short, entity_type);
    // Mesh is at mesh.glb in the same directory as the template
    let mesh_path = template_path.replace("asteroid.json", "mesh.glb");
    #[allow(clippy::panic)]
    let template = match load_template(&template_path, entity_type) {
        Ok(t) => t,
        Err(e) => panic!("fatal: failed to load template '{template_short}': {e}"),
    };
    (entity_type.to_string(), template_path, template, mesh_path)
}

/// Loads a regular ship template.
fn load_ship(template_short: &str) -> (String, String, Value, String) {
    let entity_type = "ship";
    let template_path = resolve_template_path(template_short, entity_type);
    #[allow(clippy::panic)]
    let template = match load_template(&template_path, entity_type) {
        Ok(t) => t,
        Err(e) => panic!("fatal: failed to load template '{template_short}': {e}"),
    };
    let mesh_path = template_path.clone();
    (entity_type.to_string(), template_path, template, mesh_path)
}
