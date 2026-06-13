// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! System sets for entity spawning order.
//!
//! These sets ensure entities are spawned in dependency order:
//! suns before planets, planets before moons, etc.
//!
//! See ADR-0038 (Entity template system) and ADR-0018 (State management).

use bevy::prelude::*;

/// System sets for controlling the order of entity spawning.
///
/// Each set represents a category of entities or post-processing task.
/// Systems are ordered via `.after()` to respect dependencies:
///
/// ```ignore
/// SunSpawner -> PlanetSpawner -> MoonSpawner -> StationSpawner ->
/// ShipSpawner -> NpcSpawner -> MarkDebugAxes
/// ```
///
/// All spawning and post-processing happens in `OnEnter(AppState::SpawningEntities)`.
/// Per ADR-0005 (plugin architecture), `MarkDebugAxes` decouples debug visualization
/// from domain plugins.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorldSpawnSet {
    /// Spawn suns and stars. No dependencies.
    SpawnSuns,

    /// Spawn planets. Depends on: `SpawnSuns` (for orbital mechanics).
    SpawnPlanets,

    /// Spawn moons. Depends on: `SpawnPlanets`.
    SpawnMoons,

    /// Spawn space stations. Depends on: `SpawnPlanets`, `SpawnMoons`.
    SpawnStations,

    /// Spawn asteroids and debris. Depends on: (none, but runs after stations).
    SpawnAsteroids,

    /// Spawn player-controlled and NPC ships. Runs last in domain spawning.
    SpawnShips,

    /// Spawn AI-driven NPC entities. Depends on: `SpawnShips`.
    SpawnNpcs,

    /// Mark eligible entities with debug axes. Runs after all domain spawning.
    /// Per ADR-0005, this decouples debug visualization from domain plugins.
    MarkDebugAxes,
}
