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
/// `Boot -> LoadingDefaults -> LoadingWorld -> InGame`
/// for the default development flow.
///
/// `MainMenu` and `Paused` are reserved for later milestones and must
/// not be transitioned into during M1.
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    /// Initial frame; no plugin has completed setup yet.
    ///
    /// The application enters this state automatically on startup.
    /// `CorePlugin` immediately transitions to [`AppState::LoadingDefaults`]
    /// on the first `OnEnter(Boot)` system.
    #[default]
    Boot,

    /// Loading default world and configuration files.
    ///
    /// `ConfigPlugin` runs its loaders here and transitions to
    /// [`AppState::LoadingWorld`] when all resources are ready.
    LoadingDefaults,

    /// World JSON has been parsed; entities are being spawned.
    ///
    /// `WorldPlugin` inserts [`crate::WorldDefResource`] (once it exists)
    /// and triggers spawning. Transitions to [`AppState::InGame`] when
    /// the scene is ready.
    LoadingWorld,

    /// Simulation is running; the player has control.
    ///
    /// All gameplay systems (input, physics, camera follow, HUD) run
    /// in this state.
    InGame,
}
