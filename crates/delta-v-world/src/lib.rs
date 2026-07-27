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
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
// Multiple crate versions are unavoidable due to Bevy ecosystem dependencies
// pulling in transitive duplicates that cannot be unified without upstream fixes.
#![allow(clippy::multiple_crate_versions)]
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

pub mod error;
pub mod loader;
pub mod resources;
pub mod world_def;

#[cfg(test)]
#[path = "loader_tests.rs"]
mod loader_tests;

pub use delta_v_core::SpawnEntity;
pub use error::WorldError;
pub use resources::WorldDefResource;
pub use resources::WorldPath;
pub use world_def::WorldDef;

use bevy::prelude::*;
use delta_v_assets::template::{
    load_ai_controlled_ship, load_asteroid, load_planet, load_player_controlled_ship, load_ship,
    load_sun,
};
use delta_v_core::AppState;

use crate::loader::load_world;
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
        app.add_message::<SpawnEntity>()
            .add_systems(OnEnter(AppState::LoadingWorld), load_world_system)
            // Setup scene lighting as fallback for worlds without suns.
            // Runs after world is loaded but before entities are spawned.
            .add_systems(OnEnter(AppState::SpawningEntities), setup_scene_lighting);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Sets up scene lighting as a fallback for worlds without suns.
///
/// Per ADR-0007, this runs in `Update` during `AppState::SpawningEntities`.
/// If the world has at least one sun entity, no fallback lighting is added
/// since the sun will provide its own `PointLight`.
#[allow(clippy::needless_pass_by_value)]
fn setup_scene_lighting(
    mut commands: Commands<'_, '_>,
    world_def: Option<Res<'_, WorldDefResource>>,
) {
    // Check if the world has any suns - if so, skip fallback lighting
    let has_sun = world_def.as_ref().is_some_and(|w| {
        w.0.entities
            .iter()
            .any(|e| matches!(&e.template, Some(delta_v_types::EntityTemplate::Sun(_))))
    });

    if !has_sun {
        delta_v_spawn::lighting::setup_scene_lighting(&mut commands);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Loads the world definition and emits `SpawnEntity` events.
///
/// Reads the world JSON (from [`WorldPath`] resource), validates it,
/// inserts [`WorldDefResource`], and emits a `SpawnEntity` event for
/// each entity in the world.
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
#[allow(clippy::needless_pass_by_value)]
fn load_world_system(
    world_path: Res<'_, WorldPath>,
    mut commands: Commands<'_, '_>,
    mut events: MessageWriter<'_, SpawnEntity>,
    mut next: ResMut<'_, NextState<AppState>>,
) {
    // INVARIANT: a missing or invalid world file is a hard startup
    // error (ADR-0013). The panic is intentional; no recovery is possible.
    #[allow(clippy::panic)]
    let mut world = load_world(world_path.as_ref()).unwrap_or_else(|e| {
        panic!("fatal: failed to load world: {e}");
    });
    tracing::info!("world loaded: {}", world.name);

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
    // Load templates for each entity and emit SpawnEntity events.
    // Per ADR-0038, domain plugins listen for these events and spawn
    // entities based on `entity_type`, in dependency order via WorldSpawnSet.
    for entity_spawn in &mut world.entities {
        let spawn_event = build_spawn_event(entity_spawn);
        events.write(spawn_event);
    }

    commands.insert_resource(WorldDefResource(world));
    next.set(AppState::SpawningEntities);
}

/// Builds a `SpawnEntity` event for a world entity spawn definition.
///
/// # Panics
///
/// Panics if template loading fails. This is intentional per ADR-0013 (no silent fallbacks).
#[allow(clippy::expect_used, clippy::panic)]
fn build_spawn_event(entity_spawn: &mut EntitySpawn) -> SpawnEntity {
    let template_short = &entity_spawn.template_short;

    // Load template based on entity type.
    // For player_controlled ships: load player_controlled_ship.json and merge with ship.json.
    // For AI-controlled ships: load ai_controlled_ship.json and merge with ship.json.
    // For asteroids: load asteroid.json directly.
    // For suns: load sun.json directly.
    // For planets: load planet.json directly.
    // For other ships: load ship.json directly.
    // INVARIANT: template loading must succeed (ADR-0013).
    let (template_path, merged_template) = if entity_spawn.player_controlled {
        // INVARIANT: player_controlled_ship template is required by schema (ADR-0013)
        load_player_controlled_ship(template_short)
            .expect("player_controlled_ship template must load successfully")
    } else if entity_spawn.ai_task.is_some() {
        // INVARIANT: ai_controlled_ship template is required by schema (ADR-0013)
        load_ai_controlled_ship(template_short)
            .expect("ai_controlled_ship template must load successfully")
    } else if template_short.starts_with("asteroids/") {
        // INVARIANT: asteroid template is required by schema (ADR-0013)
        load_asteroid(template_short).expect("asteroid template must load successfully")
    } else if template_short.starts_with("suns/") {
        // INVARIANT: sun template is required by schema (ADR-0013)
        load_sun(template_short).expect("sun template must load successfully")
    } else if template_short.starts_with("planets/") {
        // INVARIANT: planet template is required by schema (ADR-0013)
        load_planet(template_short).expect("planet template must load successfully")
    } else {
        // INVARIANT: ship template is required by schema (ADR-0013)
        load_ship(template_short).expect("ship template must load successfully")
    };

    // Store the loaded template in the entity spawn
    entity_spawn.template = Some(merged_template.clone());

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

    let mut spawn_event =
        SpawnEntity::new(entity_spawn.id.clone(), merged_template, template_path, pos)
            .with_rotation(rot)
            .with_scale(scale);

    // Pass through the AI task if present.
    if let Some(ref ai_task) = entity_spawn.ai_task {
        let task_str = match ai_task {
            delta_v_types::AiTaskJson::Patrol => "patrol",
        };
        spawn_event = spawn_event.with_ai_task(task_str.to_string());
    }

    // Pass through the mass override if present.
    // Mass is NOT scaled - it is used as-is or overridden.
    if let Some(ref mass) = entity_spawn.mass {
        spawn_event = spawn_event.with_mass(mass.value);
    }

    spawn_event
}
