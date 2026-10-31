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
use delta_v_test_framework::{
    ScreenshotCaptureState, ScreenshotTestConfig, check, run_screenshot_test,
};
use delta_v_types::LogicalAction;
use delta_v_ui::overlays::keybindings_menu::{
    components::KeybindingsMenuRoot, resources::KeybindingsMenuOpen,
};
use delta_v_ui::overlays::navigation_menu::{
    components::NavigationMenuRoot, resources::NavigationMenuOpen,
};
use delta_v_ui::window::components::WindowRoot;
use delta_v_ui::window::{UiTheme, WindowConfig, spawn_window};
use std::path::Path;

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

/// Test: a dedicated scrollable window with enough dummy text to overflow the
/// content area shows visible top and bottom fade zones.
///
/// This test drives the full event loop (`app.run()`) because it needs
/// `MessageWriter<AppExit>` and `ResMut<ScreenshotCaptureState>` inside the
/// `Update` schedule. Dispatched via `--test-run` (not `--test-function`),
/// so `main.rs` does NOT call `settle_render` before this function runs.
///
/// Spawns a `spawn_window` directly (not via a menu) and fills it with enough
/// rows of dummy text to overflow the content area, so the bottom fade zone
/// actually fades text out. After settling the render graph we capture the
/// framebuffer with Bevy's built-in `Screenshot` API and compare it
/// pixel-for-pixel against a committed golden image at
/// `tests/expected_results/fade_zones_visible.png`.
///
/// First run: no golden image exists, so the capture is written as the new
/// golden image and the test passes. Review the PNG before committing it.
/// Subsequent runs compare exactly (tolerance 0); on mismatch the test calls
/// `process::exit(1)`, so a refactor or library update that broke the fade
/// zones is caught.
///
/// The window is spawned through a Bevy system (not directly from the test
/// function) because `spawn_window` requires `Res<AssetServer>` and
/// `Res<UiTheme>` alongside `Commands` — a combination only a system can
/// provide.
///
/// Runs inside the event loop via the `Update` schedule; captures the
/// framebuffer and compares it pixel-for-pixel against the committed golden
/// image at `tests/expected_results/fade_zones_visible.png`. Tolerance is 0 px.
fn test_fade_zones_visible(app: &mut App) {
    // allow-default: FadeTestWindowRoot is a test-only Bevy Resource, not JSON-backed.
    // ADR-0013/ADR-0039 Default ban applies only to JSON-deserialized types.
    #[derive(Resource, Default)]
    struct FadeTestWindowRoot(pub Option<Entity>);

    #[allow(
        clippy::needless_pass_by_value,
        reason = "Bevy system parameters: `Res<'_, T>` is passed by value in the \
                  system signature and Bevy auto-borrows the resource at call time; \
                  taking `&Res` here is not a valid system parameter"
    )]
    fn spawn_fade_test_window_system(
        mut commands: Commands<'_, '_>,
        asset_server: Res<'_, AssetServer>,
        theme: Res<'_, UiTheme>,
        mut root: ResMut<'_, FadeTestWindowRoot>,
    ) {
        if root.0.is_some() {
            return;
        }
        let config = WindowConfig {
            title: "Fade Zone Test".to_string(),
            hint: "scroll me".to_string(),
            size: Vec2::new(300.0, 200.0),
            header_height: 20.0,
            animation: None,
            scrollable: true,
            // The border animation advances on wall-clock time, which makes the
            // framebuffer non-deterministic across processes. This test compares
            // screenshots byte-for-byte (tolerance 0), so disable it.
            border_enabled: false,
        };

        root.0 = Some(spawn_window(
            &mut commands,
            &config,
            &asset_server,
            &theme,
            |ui| {
                for i in 0..20 {
                    ui.spawn((
                        Name::new(format!("DummyRow_{i}")),
                        Text::new(format!("dummy text row {i}")),
                        TextFont {
                            font: bevy::prelude::FontSource::Handle(theme.font.clone()),
                            font_size: UiTheme::TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(UiTheme::LABEL_COLOR),
                        TextLayout::default(),
                    ));
                }
            },
        ));
    }

    fn fade_zones_system(
        mut commands: Commands<'_, '_>,
        roots: Query<'_, '_, (), With<WindowRoot>>,
        names: Query<'_, '_, &Name>,
        mut exit_events: MessageWriter<'_, AppExit>,
        mut capture_state: ResMut<'_, ScreenshotCaptureState>,
    ) {
        let root_count = roots.iter().count();
        if root_count == 0 {
            // Window not spawned yet — let spawn_fade_test_window_system run first.
            return;
        }
        check(root_count >= 1, "test window root should exist");
        let fade_count = names
            .iter()
            .filter(|name| {
                let n = name.as_str();
                n == "FadeTop" || n == "FadeBottom"
            })
            .count();
        check(
            fade_count >= 2,
            "both FadeTop and FadeBottom entities should be spawned",
        );

        // Delegate screenshot lifecycle to the shared helper.
        let config = ScreenshotTestConfig {
            expected_dir: Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/expected_results"),
            expected_filename: "fade_zones_visible.png",
            diff_suffix: "ui_fadezone_tests",
            test_name: "fade_zones_visible",
        };
        run_screenshot_test(&mut commands, &mut exit_events, &mut capture_state, &config);
    }

    // Settle the render graph BEFORE registering test systems. This matches
    // the old harness=false flow where make_app(200, ...) called
    // settle_render(app, 200) before the test systems were added. The window
    // must spawn during app.run(), not during settle, otherwise the
    // screenshot capture can't find the window in the new event loop.
    //
    // The thread-local guard in settle_render ensures finish()/cleanup()
    // only runs once; app.run() then drives the full event loop.
    delta_v_test_framework::settle_render(app, 200);

    app.init_resource::<FadeTestWindowRoot>();
    app.init_resource::<ScreenshotCaptureState>();
    app.add_systems(Update, spawn_fade_test_window_system);
    app.add_systems(Update, fade_zones_system);

    let exit = app.run();
    let code = i32::from(!exit.is_success());
    std::process::exit(code);
}

/// Register all UI test functions in the global registry.
///
/// Called from [`super::init_registry`] (the crate-level
/// `real_app_ITs::init_registry`).
///
/// `ui::fade_zones_visible` is registered for `--test-run` (it drives
/// `app.run()` itself); the menu-toggle tests are registered for
/// `--test-function` (they receive a pre-settled `App`).
#[cfg(feature = "test")]
pub fn init_registry() {
    do_register("ui::keybindings_menu_toggle", test_keybindings_menu_toggle);
    do_register("ui::navigation_menu_toggle", test_navigation_menu_toggle);
    do_register("ui::fade_zones_visible", test_fade_zones_visible);
}

#[cfg(feature = "test")]
fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry {
        name,
        func,
        panic_expected: false,
        panic_message: "",
    };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
