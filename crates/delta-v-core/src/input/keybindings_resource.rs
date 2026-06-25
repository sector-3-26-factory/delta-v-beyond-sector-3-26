// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Keybindings resource for input map construction.
//!
//! [`KeybindingsResource`] is defined in `delta-v-core` (not in
//! `delta-v-config`) so that [`crate::input::input_map::build_input_map`]
//! can read it without creating a crate-dependency cycle.
//!
//! The actual JSON loading and validation happens in `delta-v-config`,
//! which inserts this resource during startup.

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
/// [`crate::AppState::LoadingDefaults`]. The inner map is keyed by the
/// `snake_case` action name strings returned by
/// [`crate::LogicalAction::as_str`].
#[derive(Resource)]
pub struct KeybindingsResource(pub HashMap<String, ActionBindings>);
