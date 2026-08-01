// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Input map construction from keybindings JSON.
//!
//! Converts `KeybindingsResource` into an `InputMap<LogicalAction>` that maps
//! `LogicalAction` to physical keys/buttons.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::input::KeybindingsResource;
use delta_v_types::LogicalAction;

/// Builds an `InputMap<LogicalAction>` from the loaded keybindings.
///
/// Multi-key bindings (chords) are bound using [`ButtonlikeChord`] so that
/// all keys must be pressed together to trigger the action. Single-key
/// bindings are bound directly.
pub fn build_input_map(keybindings: &KeybindingsResource) -> InputMap<LogicalAction> {
    use leafwing_input_manager::user_input::ButtonlikeChord;

    let mut input_map = InputMap::<LogicalAction>::default();

    for action in LogicalAction::all() {
        let name = action.as_str();
        let Some(bindings) = keybindings.0.get(name) else {
            continue;
        };

        let key_codes: Vec<KeyCode> = bindings
            .keyboard
            .iter()
            .filter_map(|k| parse_key_code(k.as_str()))
            .collect();

        match key_codes.as_slice() {
            [] => {}
            [single] => {
                input_map = input_map.with(*action, *single);
                tracing::debug!(
                    "[build_input_map] action '{}' bound to single key: {:?}",
                    name,
                    single
                );
            }
            keys => {
                input_map = input_map.with(*action, ButtonlikeChord::new(keys.to_vec()));
                tracing::debug!(
                    "[build_input_map] action '{}' bound to chord: {:?}",
                    name,
                    keys
                );
            }
        }

        if let Some(ref gamepad) = bindings.gamepad_button
            && let Some(button) = parse_gamepad_button(gamepad)
        {
            input_map = input_map.with(*action, button);
        }
    }

    input_map
}

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
        "KeyN" => Some(KeyCode::KeyN),
        "KeyT" => Some(KeyCode::KeyT),
        "Digit1" => Some(KeyCode::Digit1),
        "Digit2" => Some(KeyCode::Digit2),
        "Digit3" => Some(KeyCode::Digit3),
        "Digit4" => Some(KeyCode::Digit4),
        "Digit5" => Some(KeyCode::Digit5),
        "Digit6" => Some(KeyCode::Digit6),
        "Digit7" => Some(KeyCode::Digit7),
        "Digit8" => Some(KeyCode::Digit8),
        "Digit9" => Some(KeyCode::Digit9),
        "Digit0" => Some(KeyCode::Digit0),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "ControlLeft" => Some(KeyCode::ControlLeft),
        "ShiftLeft" => Some(KeyCode::ShiftLeft),
        "AltLeft" => Some(KeyCode::AltLeft),
        "ArrowUp" => Some(KeyCode::ArrowUp),
        "ArrowDown" => Some(KeyCode::ArrowDown),
        "ArrowLeft" => Some(KeyCode::ArrowLeft),
        "ArrowRight" => Some(KeyCode::ArrowRight),
        "Space" => Some(KeyCode::Space),
        "Tab" => Some(KeyCode::Tab),
        _ => {
            tracing::warn!("keybindings: unknown key name '{name}' (ignored)");
            None
        }
    }
}

fn parse_gamepad_button(name: &str) -> Option<GamepadButton> {
    match name {
        "GamepadSouth" => Some(GamepadButton::South),
        "GamepadNorth" => Some(GamepadButton::North),
        "GamepadEast" => Some(GamepadButton::East),
        "GamepadWest" => Some(GamepadButton::West),
        "GamepadStart" => Some(GamepadButton::Start),
        "GamepadSelect" => Some(GamepadButton::Select),
        _ => {
            tracing::warn!("keybindings: unknown gamepad button '{name}' (ignored)");
            None
        }
    }
}
