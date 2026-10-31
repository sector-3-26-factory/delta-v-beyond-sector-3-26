// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! UI real-app integration tests.
//!
//! Menu-toggle tests are registered in the Z2 registry and driven by
//! `main.rs` (which calls `settle_render(app, 60)` before dispatching).
//! Tests use plain `assert!` — a panic exits the process with code 1,
//! which the Z2 driver reads as failure.
//!
//! The fadezone screenshot test (`fadezone`) is NOT registered here:
//! it runs inside `app.run()` (it needs `MessageWriter<AppExit>` and
//! `ResMut<ScreenshotCaptureState>` inside the `Update` schedule), which
//! is incompatible with the Z2 `fn(&mut App)` model. It lives as a
//! separate `harness = false` `[[test]]` target in this crate's
//! `Cargo.toml`.

#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::cargo
)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use bevy::prelude::*;
use delta_v_types::LogicalAction;
use delta_v_ui::overlays::keybindings_menu::{
    components::KeybindingsMenuRoot, resources::KeybindingsMenuOpen,
};
use delta_v_ui::overlays::navigation_menu::{
    components::NavigationMenuRoot, resources::NavigationMenuOpen,
};

/// Test: F1 toggles the keybindings menu open and closed.
fn test_keybindings_menu_toggle(app: &mut App) {
    // Verify menu starts closed
    let menu_resource = app.world().resource::<KeybindingsMenuOpen>();
    assert!(!menu_resource.0, "keybindings menu should start closed");
    assert!(
        app.world_mut()
            .query_filtered::<Entity, With<KeybindingsMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 0,
        "keybindings menu UI should not exist initially"
    );

    // Press F1 to open
    delta_v_test_framework::press_and_update(app, LogicalAction::ToggleKeybindingsMenu);

    // Verify menu opened
    let menu_resource = app.world().resource::<KeybindingsMenuOpen>();
    assert!(
        menu_resource.0,
        "keybindings menu should be open after press"
    );
    assert!(
        app.world_mut()
            .query_filtered::<Entity, With<KeybindingsMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 1,
        "keybindings menu UI should be spawned"
    );

    // Press F1 again to close
    delta_v_test_framework::press_and_update(app, LogicalAction::ToggleKeybindingsMenu);

    // Verify menu closed
    let menu_resource = app.world().resource::<KeybindingsMenuOpen>();
    assert!(
        !menu_resource.0,
        "keybindings menu should be closed after second press"
    );
    assert!(
        app.world_mut()
            .query_filtered::<Entity, With<KeybindingsMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 0,
        "keybindings menu UI should be despawned"
    );
}

/// Test: N toggles the navigation menu open and closed.
fn test_navigation_menu_toggle(app: &mut App) {
    // Verify menu starts closed
    let menu_resource = app.world().resource::<NavigationMenuOpen>();
    assert!(!menu_resource.0, "navigation menu should start closed");
    assert!(
        app.world_mut()
            .query_filtered::<Entity, With<NavigationMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 0,
        "navigation menu UI should not exist initially"
    );

    // Press N to open
    delta_v_test_framework::press_and_update(app, LogicalAction::ToggleNavigationMenu);

    // Verify menu opened
    let menu_resource = app.world().resource::<NavigationMenuOpen>();
    assert!(
        menu_resource.0,
        "navigation menu should be open after press"
    );
    assert!(
        app.world_mut()
            .query_filtered::<Entity, With<NavigationMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 1,
        "navigation menu UI should be spawned"
    );

    // Press N again to close
    delta_v_test_framework::press_and_update(app, LogicalAction::ToggleNavigationMenu);

    // Verify menu closed
    let menu_resource = app.world().resource::<NavigationMenuOpen>();
    assert!(
        !menu_resource.0,
        "navigation menu should be closed after second press"
    );
    assert!(
        app.world_mut()
            .query_filtered::<Entity, With<NavigationMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 0,
        "navigation menu UI should be despawned"
    );
}

/// Register all UI test functions in the global registry.
///
/// Called from [`super::init_registry`] (the crate-level
/// `real_app_ITs::init_registry`).
#[cfg(feature = "test")]
pub fn init_registry() {
    do_register("ui::keybindings_menu_toggle", test_keybindings_menu_toggle);
    do_register("ui::navigation_menu_toggle", test_navigation_menu_toggle);
}

#[cfg(feature = "test")]
fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
