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
/// Systems are ordered via `.after()` to respect dependencies. The chain below
/// is the authoritative one — this example records which set actually ran, in
/// which order, so a reordering of the `.chain()` breaks the test:
///
/// ```
/// use bevy::prelude::*;
/// use delta_v_core::spawn::WorldSpawnSet;
///
/// #[derive(Resource, Default)]
/// struct SpawnOrder(Vec<&'static str>);
///
/// let mut app = App::new();
/// app.init_resource::<SpawnOrder>();
/// app.configure_sets(
///     Update,
///     (
///         WorldSpawnSet::SpawnSuns,
///         WorldSpawnSet::SpawnPlanets,
///         WorldSpawnSet::SpawnMoons,
///         WorldSpawnSet::SpawnStations,
///         WorldSpawnSet::SpawnAsteroids,
///         WorldSpawnSet::SpawnShips,
///         WorldSpawnSet::SpawnNpcs,
///         WorldSpawnSet::MarkDebugAxes,
///     )
///         .chain(),
/// );
///
/// app.add_systems(
///     Update,
///     (
///         (|mut o: ResMut<SpawnOrder>| o.0.push("SpawnSuns")).in_set(WorldSpawnSet::SpawnSuns),
///         (|mut o: ResMut<SpawnOrder>| o.0.push("SpawnPlanets")).in_set(WorldSpawnSet::SpawnPlanets),
///         (|mut o: ResMut<SpawnOrder>| o.0.push("SpawnMoons")).in_set(WorldSpawnSet::SpawnMoons),
///         (|mut o: ResMut<SpawnOrder>| o.0.push("SpawnStations")).in_set(WorldSpawnSet::SpawnStations),
///         (|mut o: ResMut<SpawnOrder>| o.0.push("SpawnAsteroids")).in_set(WorldSpawnSet::SpawnAsteroids),
///         (|mut o: ResMut<SpawnOrder>| o.0.push("SpawnShips")).in_set(WorldSpawnSet::SpawnShips),
///         (|mut o: ResMut<SpawnOrder>| o.0.push("SpawnNpcs")).in_set(WorldSpawnSet::SpawnNpcs),
///         (|mut o: ResMut<SpawnOrder>| o.0.push("MarkDebugAxes")).in_set(WorldSpawnSet::MarkDebugAxes),
///     ),
/// );
///
/// app.update();
///
/// let expected = [
///     "SpawnSuns",
///     "SpawnPlanets",
///     "SpawnMoons",
///     "SpawnStations",
///     "SpawnAsteroids",
///     "SpawnShips",
///     "SpawnNpcs",
///     "MarkDebugAxes",
/// ];
/// let actual = &app.world().resource::<SpawnOrder>().0;
/// assert_eq!(actual, &expected);
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
