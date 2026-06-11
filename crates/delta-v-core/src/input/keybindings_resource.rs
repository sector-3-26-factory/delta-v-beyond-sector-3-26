// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Loaded and validated keybindings.
//!
//! [`KeybindingsResource`] is defined in `delta-v-core` (not in
//! `delta-v-config`) so that [`crate::input::input_translation_system`]
//! can read it without creating a crate-dependency cycle.
//!
//! Dependency direction (ADR-0002):
//! - `delta-v-config` → `delta-v-core`   (uses `AppState`, inserts this resource)
//! - `delta-v-core`   (defines & reads this resource; does NOT depend on delta-v-config)
//!
//! `ConfigPlugin` in `delta-v-config` inserts this resource during
//! [`crate::AppState::LoadingDefaults`].
//!
//! See ADR-0011 (Keybindings configuration).

use std::collections::HashMap;

use bevy::prelude::*;

/// Per-action keyboard (and optional gamepad) bindings.
///
/// This is a minimal mirror of `delta_v_config::ActionBindings`, owned
/// here so `delta-v-core` does not depend on `delta-v-config`.
#[derive(Debug, Clone)]
pub struct ActionBindings {
    /// Bevy `KeyCode` variant name strings (e.g. `"KeyW"`) that trigger
    /// this action.
    pub keyboard: Vec<String>,
    /// Optional gamepad button name. `None` means no gamepad binding.
    pub gamepad_button: Option<String>,
}

/// Loaded and validated keybindings.
///
/// Inserted as a Bevy resource by `ConfigPlugin` during
/// [`crate::AppState::LoadingDefaults`]. All gameplay systems that need
/// to query which keys are bound to which actions read this resource.
///
/// The inner map is keyed by the `snake_case` action name strings
/// returned by [`crate::LogicalAction::as_str`].
///
/// The value is guaranteed to have passed schema validation (ADR-0012)
/// and to contain no silent fallbacks (ADR-0013).
#[derive(Resource)]
pub struct KeybindingsResource(pub HashMap<String, ActionBindings>);
