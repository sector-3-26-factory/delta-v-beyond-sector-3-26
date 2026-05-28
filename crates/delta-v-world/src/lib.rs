// See AGENTS.md
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

pub mod error;
pub mod events;
pub mod loader;
pub mod resources;
pub mod world_def;

#[cfg(test)]
#[path = "loader_tests.rs"]
mod loader_tests;

pub use error::WorldError;
pub use events::SpawnEntity;
pub use resources::WorldDefResource;
pub use world_def::WorldDef;

use bevy::prelude::*;
use delta_v_core::AppState;

use crate::loader::load_default_world;

/// World plugin: loads and validates the world definition.
///
/// Systems run in [`AppState::LoadingWorld`]. On success the plugin
/// inserts [`WorldDefResource`] and transitions to [`AppState::InGame`].
/// On failure the application panics with a descriptive message
/// (ADR-0013).
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEntity>()
            .add_systems(OnEnter(AppState::LoadingWorld), load_world_system);
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
/// Transitions to [`AppState::SpawningEntities`] so domain plugins can
/// spawn entities in dependency order (per ADR-0038).
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

    // Emit SpawnEntity events for each entity in the world.
    // Per ADR-0038, domain plugins listen for these events and spawn
    // entities based on entity_type, in dependency order via WorldSpawnSet.
    for entity_spawn in &world.entities {
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

        // For M1, template_data is empty. Templates are loaded by domain
        // plugins from the template path (future enhancement per ADR-0038).
        let template = serde_json::json!({});

        let spawn_event = SpawnEntity::new(entity_spawn.entity_type.clone(), template, pos)
            .with_rotation(rot)
            .with_scale(scale);

        events.send(spawn_event);
    }

    commands.insert_resource(WorldDefResource(world));
    next.set(AppState::SpawningEntities);
}
