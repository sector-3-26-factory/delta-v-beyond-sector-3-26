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

use std::path::PathBuf;

use bevy::prelude::*;
use delta_v_core::AppState;

use crate::loader::load_default_world;
use crate::template_loader::load_template;

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

    // Emit SpawnEntity events for each entity in the world.
    // Per ADR-0038, domain plugins listen for these events and spawn
    // entities based on `entity_type`, in dependency order via WorldSpawnSet.
    for entity_spawn in &world.entities {
        // Load and validate the template file per ADR-0038.
        // The entity_type is derived from the template itself by reading
        // the raw JSON first, then validating against the correct schema.
        // If template loading fails, this is a hard error (ADR-0013).
        #[allow(clippy::panic, clippy::indexing_slicing)]
        let template_path = crate::template_loader::resolve_template_path(&entity_spawn.template);
        #[allow(clippy::panic)]
        let (entity_type, template, mesh_template_path) =
            match load_template_and_extract_type(&template_path) {
                Ok(result) => result,
                Err(e) => panic!(
                    "fatal: failed to load template '{}': {}",
                    entity_spawn.template, e
                ),
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

        let spawn_event = SpawnEntity::new(
            entity_spawn.id.clone(),
            entity_type,
            template,
            template_path,
            mesh_template_path,
            pos,
        )
        .with_rotation(rot)
        .with_scale(scale);

        events.send(spawn_event);
    }

    commands.insert_resource(WorldDefResource(world));
    next.set(AppState::SpawningEntities);
}

/// Loads a template and extracts the `entity_type` from it.
///
/// The `entity_type` is determined by reading the raw JSON file first
/// and extracting the `entity_type` field. The template is then validated
/// against the schema matching its declared type. Unknown entity types
/// are a hard error (ADR-0013 — no silent fallbacks).
///
/// Returns a tuple of (`entity_type`, `template_value`, `mesh_template_path`).
///
/// # Errors
///
/// Returns [`WorldError`] if:
/// - The file cannot be read or parsed
/// - The template is missing the `entity_type` field
/// - The `entity_type` is not a known/supported type
/// - The template fails schema validation
#[allow(clippy::expect_used)] // INVARIANT: CARGO_MANIFEST_DIR always set by cargo; workspace structure fixed
fn load_template_and_extract_type(
    template_path: &str,
) -> Result<(String, serde_json::Value, String), WorldError> {
    // Read the raw JSON to determine entity_type before validation.
    // Per ADR-0038, the template declares its own entity_type.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("CARGO_MANIFEST_DIR parent (crates dir) must exist")
        .parent()
        .expect("workspace root must exist");
    let assets_root = workspace_root.join("assets");
    let template_file = assets_root.join(template_path);

    let raw_json = std::fs::read_to_string(&template_file).map_err(|e| WorldError::Io {
        path: template_file.clone(),
        source: e,
    })?;
    let raw_value: serde_json::Value =
        serde_json::from_str(&raw_json).map_err(|e| WorldError::Parse {
            path: template_file.clone(),
            source: e,
        })?;

    // Extract entity_type from raw JSON — missing is a hard error.
    let entity_type = raw_value
        .get("entity_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WorldError::Schema {
            path: template_file.clone(),
            pointer: "/entity_type".to_string(),
            reason: "template missing 'entity_type' field".to_string(),
        })?
        .to_string();

    // Validate against the correct schema based on the declared entity_type.
    // Unknown entity types are a hard error — no silent fallbacks (ADR-0013).
    match entity_type.as_str() {
        "player_controlled_ship" => {
            // Extract the ship_template path from the raw JSON and resolve it to a full path.
            let ship_template_short = raw_value
                .get("ship_template")
                .and_then(|v| v.as_str())
                .ok_or_else(|| WorldError::Schema {
                    path: template_file.clone(),
                    pointer: "/ship_template".to_string(),
                    reason: "player_controlled_ship template missing 'ship_template' field"
                        .to_string(),
                })?;
            let mesh_template_path =
                crate::template_loader::resolve_template_path(ship_template_short);
            let template = load_template(template_path, &entity_type)?;
            Ok((entity_type, template, mesh_template_path))
        }
        "ship" => {
            let template = load_template(template_path, &entity_type)?;
            // For standalone ships, the mesh is in the template's own directory.
            Ok((entity_type, template, template_path.to_string()))
        }
        other => Err(WorldError::Schema {
            path: template_file,
            pointer: "/entity_type".to_string(),
            reason: format!(
                "unknown entity_type '{other}'. Supported types: player_controlled_ship, ship"
            ),
        }),
    }
}
