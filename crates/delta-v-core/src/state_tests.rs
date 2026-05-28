// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for [`crate::state::AppState`].

use crate::state::AppState;

#[test]
fn app_state_default_is_boot() {
    assert_eq!(AppState::default(), AppState::Boot);
}

#[test]
fn app_state_variants_are_not_equal() {
    assert_ne!(AppState::Boot, AppState::LoadingDefaults);
    assert_ne!(AppState::LoadingDefaults, AppState::LoadingWorld);
    assert_ne!(AppState::LoadingWorld, AppState::InGame);
}
