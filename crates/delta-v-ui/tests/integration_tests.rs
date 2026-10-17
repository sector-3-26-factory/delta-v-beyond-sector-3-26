// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Integration tests for UI menus.
//!
//! Registered as a `[[test]]` target with `harness = false` in Cargo.toml so
//! that `cargo test --test ui_integration_tests` runs `main()` on the **main
//! thread**, which is required by Bevy's `WinitPlugin` (see
//! plans/testframework.md step 8).
//!
//! The `harness = false` setting makes the test runner skip the `#[test]`
//! harness and execute the binary's `main()` directly. This is the canonical
//! invocation:
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
use delta_v_types::LogicalAction;
use delta_v_ui::overlays::keybindings_menu::{
    components::KeybindingsMenuRoot, resources::KeybindingsMenuOpen,
};
use delta_v_ui::overlays::navigation_menu::{
    components::NavigationMenuRoot, resources::NavigationMenuOpen,
};
use leafwing_input_manager::prelude::ActionState;

/// Tracks whether any assertion has failed.
///
/// SAFETY INVARIANT: this `static mut` is only ever touched from the single
/// OS main thread that `harness = false` guarantees (see the module doc
/// comment). No rayon, no `std::thread::spawn`, no Bevy worker thread — the
/// whole test body runs synchronously on the main thread, so there is no
/// data race and no reordering hazard. Any future change that introduces
/// concurrent access MUST first replace this with an `AtomicBool` or a
/// `Mutex`.
static mut TEST_PASSED: bool = true;

/// Mark the test as failed (called from `check`).
fn fail(message: &str) {
    eprintln!("FAIL: {message}");
    // SAFETY: single main-thread owner (see `TEST_PASSED` invariant).
    unsafe {
        TEST_PASSED = false;
    }
}

/// Assert `condition`, logging failure instead of panicking so the app can
/// keep running on the main thread and we can collect all failures.
fn check(condition: bool, message: &str) {
    if !condition {
        fail(message);
    }
}

/// Helper: press action and run app update.
///
/// Releases the action first and runs an update in between so that custom
/// edge-detection resources (e.g. `NavigationMenuToggleState.prev_pressed`)
/// are cleared before the next press. Without this, pressing the same action
/// twice in a row fails because the action stays `Pressed` and the toggle
/// state never sees a fresh rising edge.
fn press_action(app: &mut App, action: LogicalAction) {
    {
        let mut query = app.world_mut().query::<&mut ActionState<LogicalAction>>();
        let mut action_state = query
            .single_mut(app.world_mut())
            .expect("ActionState not found");
        action_state.release(&action);
    }
    app.update();
    {
        let mut query = app.world_mut().query::<&mut ActionState<LogicalAction>>();
        let mut action_state = query
            .single_mut(app.world_mut())
            .expect("ActionState not found");
        action_state.press(&action);
    }
    app.update();
}

/// Run the menu-toggle integration test.
fn run_test() {
    let mut app = build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-ui/tests/fixtures/assets".to_string(),
        world_name: Some("test".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    });

    // Bevy's default runner (`run_once`) calls `finish()` -> `cleanup()` ->
    // `update()`. `Plugin::finish()` is where `RenderDevice` is inserted
    // into the render world (via `RenderResources::unpack_into`). Calling
    // `app.update()` alone skips `finish()`, so `Res<RenderDevice>` is not
    // yet available and render systems panic. We call `finish()` here, then
    // `cleanup()` once to set up the render thread (which consumes the
    // RenderApp on the first `update()`). Subsequent `update()` calls drive
    // the render graph. (See plans/testframework.md step 9.)
    app.finish();
    app.cleanup();
    for _ in 0..20 {
        app.update();
    }

    // ===== Test Keybindings Menu (F1) =====
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
    press_action(&mut app, LogicalAction::ToggleKeybindingsMenu);

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
    press_action(&mut app, LogicalAction::ToggleKeybindingsMenu);

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

    // ===== Test Navigation Menu (N) =====
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
    press_action(&mut app, LogicalAction::ToggleNavigationMenu);

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
    press_action(&mut app, LogicalAction::ToggleNavigationMenu);

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
    run_test();

    // SAFETY: single main-thread owner (see `TEST_PASSED` invariant).
    unsafe {
        if TEST_PASSED {
            println!("ALL TESTS PASSED");
            std::process::exit(0);
        } else {
            eprintln!("SOME TESTS FAILED");
            std::process::exit(1);
        }
    }
}
