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

pub mod resources;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

pub use delta_v_core::{AppState, SpawnEntity};
pub use delta_v_types::{EntitySpawn, WorldDef};
pub use resources::WorldDefResource;
pub use resources::WorldPath;

use bevy::prelude::*;
use delta_v_assets::template::{load_template_by_kind, load_world};
use delta_v_physics::check_collision;
use delta_v_physics::collision::CollisionShape;
use delta_v_types::collision::scale_collision_shape;

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
/// Panics if the world file or any template file cannot be loaded or validated.
/// This is intentional per ADR-0013 (no silent fallbacks).
#[allow(clippy::needless_pass_by_value)]
fn load_world_system(
    world_path: Res<'_, WorldPath>,
    mut commands: Commands<'_, '_>,
    mut events: MessageWriter<'_, SpawnEntity>,
    mut next: ResMut<'_, NextState<AppState>>,
) {
    // ADR-0058: asteroid belts and fields are region definitions, not solid
    // bodies. They generate asteroids within their volume but are not
    // themselves collidable — there is plenty of empty space between the
    // asteroids they spawn. We skip them in the collision check entirely.
    const NON_SOLID_TYPES: &[&str] = &["asteroid_belt", "asteroid_field"];

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

    // Single pass: load each entity's template, check for collisions with
    // already-loaded entities, then emit the SpawnEntity event immediately.
    // Per ADR-0049, template loading goes through the unified delta-v-assets
    // entry point. Per ADR-0013, any collision is a hard error.
    //
    // We use `check_collision` from delta-v-physics (SAT for box-box,
    // exact for sphere-sphere and sphere-box) rather than a naive
    // center-inside-shape test, which misses cases where two boxes
    // overlap without either center being inside the other.
    //
    // The template is loaded once here and passed directly to
    // `build_spawn_event`, which stores it on the entity spawn and
    // forwards it into the `SpawnEntity` event — no second load.

    #[allow(clippy::indexing_slicing)]
    let mut spawned: Vec<(String, Vec3, Quat, CollisionShape)> = Vec::new();

    for entity_spawn in &mut world.entities {
        // Load the template via the unified entry point (ADR-0049).
        // The template path is carried on the LoadedTemplate wrapper.
        let loaded_template = {
            #[allow(clippy::panic)]
            load_template_by_kind(
                &entity_spawn.template_short,
                entity_spawn.player_controlled,
                entity_spawn.ai_task.as_ref(),
            )
            .unwrap_or_else(|e| {
                panic!(
                    "fatal: failed to load template for entity '{}': {e}",
                    entity_spawn.id
                )
            })
        };

        let entity_type = loaded_template.template.entity_type();

        // Belts and fields are not solid — skip the collision check for them.
        if NON_SOLID_TYPES.contains(&entity_type) {
            let spawn_event = build_spawn_event(entity_spawn, loaded_template);
            events.write(spawn_event);
            continue;
        }

        // Apply the entity's scale to the template collision shape so the
        // overlap check uses world-space dimensions, not the template's.
        let shape_data = scale_collision_shape(
            loaded_template.template.collision_shape(),
            entity_spawn.scale.length(),
        );
        let shape = CollisionShape(shape_data);

        let pos = Vec3::new(
            entity_spawn.position.x,
            entity_spawn.position.y,
            entity_spawn.position.z,
        );
        let rot = entity_spawn.rotation;

        // Check against every already-loaded solid entity.
        // Uses the actual rotation from the world definition, not identity,
        // so rotated bodies are checked correctly.
        for (other_id, other_pos, other_rot, other_shape) in &spawned {
            assert!(
                check_collision(pos, rot, &shape, *other_pos, *other_rot, other_shape).is_none(),
                "fatal: entities '{}' and '{}' have overlapping collision shapes. \
                 Entities cannot occupy the same space.",
                entity_spawn.id,
                other_id
            );
        }

        spawned.push((entity_spawn.id.clone(), pos, rot, shape));

        // No collision: emit the spawn event now, reusing the template
        // we just loaded instead of loading it a second time.
        let spawn_event = build_spawn_event(entity_spawn, loaded_template);
        events.write(spawn_event);
    }

    commands.insert_resource(WorldDefResource(world));
    next.set(AppState::SpawningEntities);
}

/// Builds a `SpawnEntity` event for a world entity spawn definition.
///
/// The template is passed in as a [`LoadedTemplate`] — the caller is
/// responsible for loading it (in the collision pass) so it isn't loaded
/// a second time here.
///
/// # Panics
///
/// Panics if template loading fails. This is intentional per ADR-0013 (no silent fallbacks).
#[allow(clippy::expect_used, clippy::panic)]
fn build_spawn_event(
    entity_spawn: &EntitySpawn,
    loaded_template: delta_v_types::LoadedTemplate,
) -> SpawnEntity {
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

    let mut spawn_event = SpawnEntity::new(entity_spawn.id.clone(), loaded_template, pos)
        .with_rotation(rot)
        .with_scale(scale);

    // Pass through the AI task if present.
    if let Some(ref ai_task) = entity_spawn.ai_task {
        let task_str = match ai_task {
            delta_v_types::AiTask::Patrol => "patrol",
        };
        spawn_event = spawn_event.with_ai_task(task_str.to_string());
    }

    // Pass through the mass override if present.
    // Mass is NOT scaled - it is used as-is or overridden.
    if let Some(mass) = entity_spawn.mass {
        spawn_event = spawn_event.with_mass(mass);
    }

    // Pass through orbital parameters if present.
    if let Some(orbital_params) = entity_spawn.orbital_parameters.clone() {
        spawn_event = spawn_event.with_orbital_parameters(orbital_params);
    }

    // Pass through asteroid belt parameters if present.
    if let Some(belt_params) = entity_spawn.asteroid_belt_parameters.clone() {
        spawn_event.asteroid_belt_parameters = Some(belt_params);
    }

    // Pass through asteroid field parameters if present.
    if let Some(field_params) = entity_spawn.asteroid_field_parameters.clone() {
        spawn_event.asteroid_field_parameters = Some(field_params);
    }

    spawn_event
}
