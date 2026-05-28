// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Deserialisation types for `keybindings.json`.
//!
//! See ADR-0011 (Keybindings configuration) and ADR-0012 (JSON schema
//! validation). These types have no `Default` impl: every instance must
//! come from a validated JSON load (ADR-0013, ADR-0014).

use std::collections::HashMap;

use serde::Deserialize;

/// Loaded keybindings: a map from logical action name to its bindings.
///
/// The logical action names are the `snake_case` strings defined in
/// `LogicalAction::as_str()` in `delta-v-core`.
#[derive(Debug, Deserialize)]
pub struct Keybindings {
    /// Map from logical action name to its input bindings.
    pub actions: HashMap<String, ActionBindings>,
}

/// Input bindings for a single logical action.
#[derive(Debug, Deserialize)]
pub struct ActionBindings {
    /// Bevy `KeyCode` variant names that trigger this action.
    ///
    /// The string form (e.g. `"KeyW"`) is parsed by
    /// `delta-v-core`'s `parse_key_code` helper at runtime.
    pub keyboard: Vec<String>,

    /// Optional gamepad button name. `None` means no gamepad binding.
    pub gamepad_button: Option<String>,
}
