// AGENTS: before modifying this file, read AGENTS.md at the repository root.
// allow-default: Bevy requires Default on States enums for state-machine initialisation; this is not a config type and carries no silent-fallback risk.

//! Application-level state machine.
//!
//! Defines the top-level [`AppState`] used by every plugin to scope its
//! systems to the correct phase of the application lifecycle.
//!
//! See ADR-0018 (State management).

use bevy::prelude::*;

/// Top-level application state.
///
/// Transitions follow the path:
/// `Boot -> LoadingDefaults -> LoadingWorld -> SpawningEntities -> InGame`
/// for the default development flow (per ADR-0038).
///
/// `MainMenu` and `Paused` are reserved for later milestones.
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    /// Initial frame; no plugin has completed setup yet.
    ///
    /// The application enters this state automatically on startup.
    /// `CorePlugin` immediately transitions to [`AppState::LoadingDefaults`]
    /// on the first `OnEnter(Boot)` system.
    #[default]
    Boot,

    /// Loading default configuration files (keybindings, settings, etc.).
    ///
    /// `ConfigPlugin` runs its loaders here and transitions to
    /// [`AppState::LoadingWorld`] when all resources are ready.
    LoadingDefaults,

    /// World JSON has been parsed and validated.
    ///
    /// `WorldPlugin` loads the world definition and emits `SpawnEntity`
    /// events. Transitions to [`AppState::SpawningEntities`] when the
    /// world is loaded.
    LoadingWorld,

    /// Entities are being spawned from templates.
    ///
    /// Domain plugins listen for `SpawnEntity` events and spawn entities
    /// in dependency order (suns, planets, moons, stations, ships).
    /// Transitions to [`AppState::InGame`] when complete (per ADR-0038).
    SpawningEntities,

    /// Simulation is running; the player has control.
    ///
    /// All gameplay systems (input, physics, camera follow, HUD) run
    /// in this state.
    InGame,

    /// Skirmish is over (win or lose).
    ///
    /// Entered when the player is destroyed or all enemy ships are
    /// destroyed. For M5, only logs the result; M6 will add HUD,
    /// restart, etc.
    SkirmishOver,
}
