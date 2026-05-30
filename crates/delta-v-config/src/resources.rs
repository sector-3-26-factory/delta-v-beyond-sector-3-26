// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Re-exports [`delta_v_core::KeybindingsResource`].
//!

//! `KeybindingsResource` is defined in `delta-v-core` so that
//! `delta-v-core`'s input translation system can read it without creating a
//! crate-dependency cycle. `ConfigPlugin` converts the loaded `Keybindings`
//! struct into `delta_v_core::KeybindingsResource` and inserts it during
//! [`delta_v_core::AppState::LoadingDefaults`].
//!
//! See ADR-0011 (Keybindings) and ADR-0002 (Repository layout).

// Re-export so that existing `use delta_v_config::KeybindingsResource` paths
// continue to resolve without change.
pub use delta_v_core::KeybindingsResource;
