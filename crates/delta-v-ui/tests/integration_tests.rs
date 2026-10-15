// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Integration tests for keybindings menu.
//!
//! See ADR-0021 (Testing strategy) and ADR-0060 (Test framework single source of truth).

// Test code is allowed to use expect/unwrap/indexing per ADR-0023.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use bevy::prelude::*;
use delta_v_app::{AppConfig, build_app};
use delta_v_ui::overlays::keybindings_menu::resources::KeybindingsMenuOpen;

/// Integration test: verify `KeybindingsMenuOpen` resource can be set and read.
///
/// This test verifies the basic resource update behavior.
#[test]
fn test_keybindings_menu_open_resource() {
    let mut app = build_app(AppConfig {
        is_headless: true,
        asset_root_suffix: "tests/fixtures/assets".to_string(),
        world_name: None,
        log_filter: Some("warn,delta_v=debug".to_string()),
    });

    // Add the KeybindingsMenuOpen resource
    app.insert_resource(KeybindingsMenuOpen(false));

    // Verify initial state
    let menu_open = app.world().get_resource::<KeybindingsMenuOpen>().unwrap();
    assert!(!menu_open.0, "menu should start closed");

    // Simulate opening the menu
    app.world_mut().resource_mut::<KeybindingsMenuOpen>().0 = true;
    let menu_open = app.world().get_resource::<KeybindingsMenuOpen>().unwrap();
    assert!(menu_open.0, "menu should be open after setting to true");

    // Simulate closing the menu
    app.world_mut().resource_mut::<KeybindingsMenuOpen>().0 = false;
    let menu_open = app.world().get_resource::<KeybindingsMenuOpen>().unwrap();
    assert!(!menu_open.0, "menu should be closed after setting to false");
}
