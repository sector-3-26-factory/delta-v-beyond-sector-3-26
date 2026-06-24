// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Input map construction from keybindings JSON.
//!
//! Uses `leafwing-input-manager` for robust input handling including
//! modifier key chords and gamepad support.

pub mod input_map;
pub mod keybindings_resource;

pub use input_map::build_input_map;
pub use keybindings_resource::KeybindingsResource;
pub use leafwing_input_manager::prelude::{ActionState, InputManagerPlugin, InputMap};
