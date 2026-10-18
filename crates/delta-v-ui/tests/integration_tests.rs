// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Integration tests for UI menus.
//!
//! Registered as a `[[test]]` target with `harness = false` in Cargo.toml so
//! that `cargo test --test ui_integration_tests` runs `main()` on the **main
//! thread**, which is required by Bevy's `WinitPlugin` (see
//! plans/testframework.md step 8).
//!
//! The `harness = false` setting makes the test runner skip the `#[test]`
//! harness and execute the binary's `main()` directly. The harness itself
//! lives in the `delta-v-test-support` crate so it can be reused across
//! crates without duplication. This file only contains the test functions.
//!
//!   xvfb-run -a cargo test --test `ui_integration_tests` -p delta-v-ui
//!
//! See ADR-0021 (Testing strategy) and ADR-0060 (Test framework single
//! source of truth).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_app::{AppConfig, build_app};
use delta_v_test_support::{check, press_action, run_harness, settle_render};
use delta_v_types::LogicalAction;
use delta_v_ui::overlays::keybindings_menu::{
    components::KeybindingsMenuRoot, resources::KeybindingsMenuOpen,
};
use delta_v_ui::overlays::navigation_menu::{
    components::NavigationMenuRoot, resources::NavigationMenuOpen,
};

/// Build the app and settle the render graph before running assertions.
///
/// Called once in `main()`; the resulting `App` is shared across all tests
/// in this target. Bevy's `WinitPlugin` permits only one `EventLoop` per
/// thread, so a single `harness = false` target can host only one `App`.
fn make_app() -> App {
    let mut app = build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-ui/tests/fixtures/assets".to_string(),
        world_name: Some("test".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    });
    settle_render(&mut app);
    app
}

/// Test: F1 toggles the keybindings menu open and closed.
fn test_keybindings_menu_toggle(app: &mut App) {
    // Verify menu starts closed
    let menu_resource = app.world().resource::<KeybindingsMenuOpen>();
    check(!menu_resource.0, "keybindings menu should start closed");
    check(
        app.world_mut()
            .query_filtered::<Entity, With<KeybindingsMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 0,
        "keybindings menu UI should not exist initially",
    );

    // Press F1 to open
    press_action(app, LogicalAction::ToggleKeybindingsMenu);

    // Verify menu opened
    let menu_resource = app.world().resource::<KeybindingsMenuOpen>();
    check(
        menu_resource.0,
        "keybindings menu should be open after press",
    );
    check(
        app.world_mut()
            .query_filtered::<Entity, With<KeybindingsMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 1,
        "keybindings menu UI should be spawned",
    );

    // Press F1 again to close
    press_action(app, LogicalAction::ToggleKeybindingsMenu);

    // Verify menu closed
    let menu_resource = app.world().resource::<KeybindingsMenuOpen>();
    check(
        !menu_resource.0,
        "keybindings menu should be closed after second press",
    );
    check(
        app.world_mut()
            .query_filtered::<Entity, With<KeybindingsMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 0,
        "keybindings menu UI should be despawned",
    );
}

/// Test: N toggles the navigation menu open and closed.
fn test_navigation_menu_toggle(app: &mut App) {
    // Verify menu starts closed
    let menu_resource = app.world().resource::<NavigationMenuOpen>();
    check(!menu_resource.0, "navigation menu should start closed");
    check(
        app.world_mut()
            .query_filtered::<Entity, With<NavigationMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 0,
        "navigation menu UI should not exist initially",
    );

    // Press N to open
    press_action(app, LogicalAction::ToggleNavigationMenu);

    // Verify menu opened
    let menu_resource = app.world().resource::<NavigationMenuOpen>();
    check(
        menu_resource.0,
        "navigation menu should be open after press",
    );
    check(
        app.world_mut()
            .query_filtered::<Entity, With<NavigationMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 1,
        "navigation menu UI should be spawned",
    );

    // Press N again to close
    press_action(app, LogicalAction::ToggleNavigationMenu);

    // Verify menu closed
    let menu_resource = app.world().resource::<NavigationMenuOpen>();
    check(
        !menu_resource.0,
        "navigation menu should be closed after second press",
    );
    check(
        app.world_mut()
            .query_filtered::<Entity, With<NavigationMenuRoot>>()
            .iter(app.world_mut())
            .count()
            == 0,
        "navigation menu UI should be despawned",
    );
}

/// Entry point — runs on the main thread so `WinitPlugin` can initialize.
///
/// Exits 0 on success, 1 on any assertion failure.
fn main() {
    let mut app = make_app();
    run_harness(
        &mut app,
        &[
            ("keybindings_menu_toggle", test_keybindings_menu_toggle),
            ("navigation_menu_toggle", test_navigation_menu_toggle),
        ],
    );
}
