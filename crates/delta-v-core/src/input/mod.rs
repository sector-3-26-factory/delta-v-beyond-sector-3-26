// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Logical input actions, the `ActiveActions` resource, and the input
//! translation / logging systems.
//!
//! Gameplay code reads [`ActiveActions`]; it never reads raw keys directly.
//! The translation from raw `KeyCode` to [`LogicalAction`] is driven by the
//! [`KeybindingsResource`] loaded by `ConfigPlugin`.
//!
//! See ADR-0011 (Keybindings configuration) and ADR-0017 (Fixed timestep).

use std::collections::BTreeSet;

use bevy::prelude::*;

pub mod keybindings_resource;
/// Input system set ordering (`InputSet`).
pub mod sets;

pub use delta_v_types::LogicalAction;
pub use keybindings_resource::{ActionBindings, KeybindingsResource};
pub use sets::InputSet;

// ---------------------------------------------------------------------------
// ActiveActions resource
// ---------------------------------------------------------------------------

/// The set of logical actions currently held down this fixed tick.
///
/// Populated by [`input_translation_system`]; read by gameplay systems.
///
/// `BTreeSet` is used (not `HashSet`) to guarantee stable iteration order
/// for determinism (ADR-0017).
// allow-default: Bevy requires Default on resources for init_resource. This
// resource starts empty and is populated each tick; it is not a config type.
#[derive(Resource, Default, Debug)]
pub struct ActiveActions(pub BTreeSet<LogicalAction>);

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Translates raw keyboard input into [`ActiveActions`].
///
/// Runs in `FixedUpdate`, gated to [`crate::AppState::InGame`] (ADR-0017).
/// Reads [`KeybindingsResource`] to map `KeyCode` name strings to
/// [`LogicalAction`]s.
#[allow(clippy::needless_pass_by_value)]
pub fn input_translation_system(
    keyboard: Res<'_, ButtonInput<KeyCode>>,
    keybindings: Res<'_, KeybindingsResource>,
    mut active: ResMut<'_, ActiveActions>,
) {
    active.0.clear();

    for &action in LogicalAction::all() {
        let name = action.as_str();
        let Some(bindings) = keybindings.0.get(name) else {
            continue;
        };
        let pressed = bindings
            .keyboard
            .iter()
            .all(|key_name| parse_key_code(key_name).is_some_and(|kc| keyboard.pressed(kc)));
        if pressed {
            active.0.insert(action);
        }
    }
}

/// Logs active actions at DEBUG level each fixed tick (ADR-0015, M1-O6).
///
/// This system is the M1 proof-of-pipeline. It remains registered in
/// subsequent milestones but is silent in release builds because DEBUG is
/// filtered out (ADR-0015).
#[allow(clippy::needless_pass_by_value)]
pub fn input_log_system(active: Res<'_, ActiveActions>) {
    if !active.0.is_empty() {
        tracing::debug!("active actions: {:?}", active.0);
    }
}

// ---------------------------------------------------------------------------
// Key-code parsing
// ---------------------------------------------------------------------------

/// Maps a Bevy `KeyCode` variant name string to the enum variant.
///
/// Covers all keys used in the default keybindings. Unknown names
/// emit a `WARN` and return `None` (no panic; the action is simply absent).
fn parse_key_code(name: &str) -> Option<KeyCode> {
    match name {
        "KeyW" => Some(KeyCode::KeyW),
        "KeyA" => Some(KeyCode::KeyA),
        "KeyS" => Some(KeyCode::KeyS),
        "KeyD" => Some(KeyCode::KeyD),
        "KeyQ" => Some(KeyCode::KeyQ),
        "KeyE" => Some(KeyCode::KeyE),
        "KeyR" => Some(KeyCode::KeyR),
        "KeyF" => Some(KeyCode::KeyF),
        "KeyC" => Some(KeyCode::KeyC),
        "F2" => Some(KeyCode::F2),
        "ControlLeft" => Some(KeyCode::ControlLeft),
        "AltLeft" => Some(KeyCode::AltLeft),
        "ArrowUp" => Some(KeyCode::ArrowUp),
        "ArrowDown" => Some(KeyCode::ArrowDown),
        "ArrowLeft" => Some(KeyCode::ArrowLeft),
        "ArrowRight" => Some(KeyCode::ArrowRight),
        "Space" => Some(KeyCode::Space),
        _ => {
            tracing::warn!("keybindings: unknown key name '{name}' (ignored)");
            None
        }
    }
}
