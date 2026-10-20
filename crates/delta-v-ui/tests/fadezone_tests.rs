// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Screenshot-based visual regression test for window fade zones.
//!
//! Registered as a `[[test]]` target with `harness = false` in Cargo.toml so
//! that `cargo test --test ui_fadezone_tests` runs `main()` on the **main
//! thread**, which is required by Bevy's `WinitPlugin` (see
//! plans/testframework.md step 8).
//!
//! The `harness = false` setting makes the test runner skip the `#[test]`
//! harness and execute the binary's `main()` directly. The harness itself
//! lives in the `delta-v-test-support` crate so it can be reused across
//! crates without duplication. This file only contains the test functions.
//!
//!   xvfb-run -a cargo test --test `ui_fadezone_tests` -p delta-v-ui
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

use std::path::Path;

use bevy::prelude::*;
use delta_v_app::{AppConfig, build_app};
use delta_v_test_support::{
    ScreenshotCaptureState, ScreenshotTestConfig, check, run_screenshot_test,
};
use delta_v_ui::window::components::WindowRoot;
use delta_v_ui::window::{UiTheme, WindowConfig, spawn_window};

/// Test: a dedicated scrollable window with enough dummy text to overflow the
/// content area shows visible top and bottom fade zones.
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
fn test_fade_zones_visible(
    mut commands: Commands<'_, '_>,
    roots: Query<(), With<WindowRoot>>,
    names: Query<&Name>,
    mut exit_events: MessageWriter<AppExit>,
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

/// Spawn the test window through a one-shot system so we get real
/// `Res<AssetServer>` / `Res<UiTheme>` handles alongside `Commands`.
/// The system writes the spawned root entity into a resource so the test
/// function can read it back after `app.update()`.
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

/// Entry point — runs on the main thread so `WinitPlugin` can initialize.
///
/// Exits 0 on success, 1 on any assertion failure.
fn main() {
    let mut app = delta_v_test_support::make_app(200, || {
        build_app(AppConfig {
            asset_root_suffix: "crates/delta-v-ui/tests/fixtures/assets".to_string(),
            world_name: Some("test".to_string()),
            log_filter: Some("warn,delta_v=debug".to_string()),
        })
    });

    app.init_resource::<FadeTestWindowRoot>();
    app.init_resource::<ScreenshotCaptureState>();
    app.add_systems(Update, spawn_fade_test_window_system);
    app.add_systems(Update, test_fade_zones_visible);

    let exit = app.run();
    let code = i32::from(!exit.is_success());
    std::process::exit(code);
}
