// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Integration tests for keybindings menu.
//!
//! See ADR-0021 (Testing strategy).

// Test code is allowed to use expect/unwrap/indexing per ADR-0023.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use bevy::app::App;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;

use delta_v_ui::overlays::keybindings_menu::resources::KeybindingsMenuOpen;

/// Integration test: verify `KeybindingsMenuOpen` resource can be set and read.
///
/// This test verifies the basic resource update behavior.
#[test]
fn test_keybindings_menu_open_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: "assets".to_string(),
        ..default()
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
