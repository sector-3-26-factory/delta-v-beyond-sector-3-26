// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Shared test harness for Bevy integration tests.
//!
//! This crate is a **dev-only** library: it is listed as a
//! `dev-dependency` by every crate that runs `harness = false`
//! integration tests, and it is never linked into production builds.
//!
//! # Why a separate crate?
//!
//! The harness helpers (`check`, `press_action`, `settle_render`,
//! `run_harness`) are shared across crates. Duplicating them in every
//! test file would invite drift; putting them in a shared module inside
//! one crate would make that crate's tests depend on another crate's
//! private helpers. A dedicated `delta-v-test-support` crate is the
//! clean solution, and ADR-0021 explicitly contemplates it:
//!
//! > "A `delta-v-test-support` (internal-only) crate may emerge if shared
//! > test fixtures grow; not yet."
//!
//! # Dependency discipline
//!
//! This crate **MUST NOT** depend on `delta-v-app` or any domain crate.
//! Doing so would create a cycle:
//!
//! ```text
//! delta-v-ui (dev) -> delta-v-test-support -> delta-v-app -> delta-v-ui
//! ```
//!
//! Instead, test functions call `delta_v_app::build_app` in their own
//! crate's dev-dependency context and pass the resulting `App` to the
//! helpers exported here. This crate only needs `bevy`,
//! `leafwing-input-manager`, and `delta-v-types`.

#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::cargo
)]
#![allow(clippy::multiple_crate_versions)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use delta_v_types::LogicalAction;
use image::GenericImageView;
use leafwing_input_manager::prelude::ActionState;

static TEST_PASSED: AtomicBool = AtomicBool::new(true);

/// Mark the test as failed (called from `check`).
pub fn fail(message: &str) {
    eprintln!("FAIL: {message}");
    TEST_PASSED.store(false, Ordering::SeqCst);
}

/// Assert `condition`, logging failure instead of panicking so the app
/// can keep running on the main thread and we can collect all failures.
pub fn check(condition: bool, message: &str) {
    if !condition {
        fail(message);
    }
}

/// Helper: press action and run app update.
///
/// Releases the action first and runs an update in between so that custom
/// edge-detection resources (e.g. `NavigationMenuToggleState.prev_pressed`)
/// are cleared before the next press. Without this, pressing the same
/// action twice in a row fails because the action stays `Pressed` and the
/// toggle state never sees a fresh rising edge.
///
/// # Panics
///
/// Panics if the `ActionState<LogicalAction>` resource is not present.
///
/// # Safety
///
/// The `ActionState<LogicalAction>` resource is inserted unconditionally by
/// `delta-v-app::build_app` (the leafwing-input-manager plugin) before any
/// test function runs. `press_action` is only ever called from test functions
/// that have already invoked `make_app` -> `build_app` -> `settle_render`,
/// so the resource is guaranteed to exist. There is no code path in this
/// crate that constructs an `App` without it, and the resource is never
/// removed or replaced between calls.
#[allow(clippy::expect_used)]
pub fn press_action(app: &mut App, action: LogicalAction) {
    {
        let mut query = app.world_mut().query::<&mut ActionState<LogicalAction>>();
        // SAFETY: `ActionState<LogicalAction>` is inserted by
        // `delta-v-app::build_app` before any test runs and is never removed.
        let mut action_state = query
            .single_mut(app.world_mut())
            .expect("ActionState not found");
        action_state.release(&action);
    }
    app.update();
    {
        let mut query = app.world_mut().query::<&mut ActionState<LogicalAction>>();
        // SAFETY: same invariant as above — the resource is present for the
        // lifetime of the shared `App`.
        let mut action_state = query
            .single_mut(app.world_mut())
            .expect("ActionState not found");
        action_state.press(&action);
    }
    app.update();
}

/// Drive the render graph so render-dependent resources are available.
///
/// Bevy's default runner (`run_once`) calls `finish()` -> `cleanup()` ->
/// `update()`. `Plugin::finish()` is where `RenderDevice` is inserted into
/// the render world (via `RenderResources::unpack_into`). Calling
/// `app.update()` alone skips `finish()`, so `Res<RenderDevice>` is not
/// yet available and render systems panic.
///
/// This function is **idempotent**: the first call runs `finish()` +
/// `cleanup()` to set up the render thread (which consumes the `RenderApp`
/// sub-app on the first `update()`), then runs `frames` `update()` calls.
/// Subsequent calls skip the `finish()`/`cleanup()` step and only run the
/// update loop, so it is safe to call defensively from test helpers
/// without worrying about whether the caller already settled the render
/// graph.
///
/// The `finish()`/`cleanup()` step is guarded by a thread-local flag
/// because Bevy's `WinitPlugin` event loop is thread-local: it can only
/// be created once per thread, so calling `cleanup()` on a second `App`
/// in the same thread panics with `RecreationAttempt`.
///
/// `frames` is the number of `app.update()` invocations to run. A single
/// update is NOT enough to settle the framebuffer under software rendering:
/// the render graph drains asynchronously, so the scene needs many ticks to
/// converge. Each test controls its own `frames` value so the settle depth
/// is a per-test knob rather than a global constant. Wall-clock sleep does
/// NOT help here: only `update()` advances the render graph.
pub fn settle_render(app: &mut App, frames: u32) {
    use std::cell::Cell;
    thread_local! {
        static RENDER_SETTLED: Cell<bool> = const { Cell::new(false) };
    }
    let needs_init = RENDER_SETTLED.with(|c| !c.get());
    if needs_init {
        app.finish();
        app.cleanup();
        RENDER_SETTLED.with(|c| c.set(true));
    }
    for _ in 0..frames {
        app.update();
    }
}

/// Build the test `App` and settle its render graph.
///
/// This is the **centralized** app factory for every `harness = false`
/// integration test target. The one and only `App` per target is built
/// here; `asset_root_suffix` and `world_name` are the only things that may
/// differ between targets (production uses `"assets"` / `"default"`, tests
/// use `"tests/fixtures/assets"` / `"test"`). The `log_filter` is the same
/// for every target.
///
/// The function always returns the `App` for the caller to run via
/// `app.run()`. The caller must register any work (e.g. a test system that
/// calls `app.exit()`) before calling `app.run()`.
///
/// `build_app` is called by the test target itself (it owns
/// `delta_v_app::build_app` via its own dev-dependency). This helper only
/// owns `settle_render`.
///
/// `frames` is the number of `app.update()` invocations `settle_render`
/// runs after `finish()`/`cleanup()`. A single update is NOT enough to
/// settle the framebuffer under software rendering; each test passes its
/// own value so the settle depth is a per-test knob.
///
/// # Panics
///
/// Panics if `settle_render` panics (e.g. `RecreationAttempt` on a second
/// `App` in the same thread — prevented by the thread-local guard).
pub fn make_app<F>(frames: u32, build: F) -> App
where
    F: FnOnce() -> bevy::prelude::App,
{
    let mut app = build();
    settle_render(&mut app, frames);
    app
}

/// Run a battery of tests against a single shared `App`.
///
/// Bevy's `WinitPlugin` permits only one `EventLoop` per thread, and
/// `build_app` uses the full `DefaultPlugins` (ADR-0060 forbids stripping
/// it down to `MinimumPlugins`). A `harness = false` `[[test]]` target runs
/// one `main()` on the main thread, so a single target can host only one
/// `App`. Tests therefore share it; each test is responsible for asserting
/// its own preconditions and leaving the app in a state that does not
/// interfere with the next test. Tests that require a fresh `App` must live
/// in their own `[[test]]` target (separate process = separate thread).
///
/// `make_app` (which calls `build_app` + `settle_render`) is called by the
/// caller before invoking this function; this helper only owns the
/// pass/fail bookkeeping and the shared `App`.
///
/// # Panics
///
/// Panics if any test function panics.
pub fn run_harness(app: &mut App, tests: &[TestSpec]) {
    let mut failed = 0;
    for (name, test_fn) in tests {
        // Reset the global state before each test so failures don't leak
        // between tests.
        TEST_PASSED.store(true, Ordering::SeqCst);
        test_fn(app);
        let passed = TEST_PASSED.load(Ordering::SeqCst);
        if passed {
            println!("PASS: {name}");
        } else {
            eprintln!("FAIL: {name}");
            failed += 1;
        }
    }

    if failed == 0 {
        println!("ALL {} TESTS PASSED", tests.len());
        std::process::exit(0);
    } else {
        eprintln!("{failed} TEST(S) FAILED");
        std::process::exit(1);
    }
}
/// A single named test in a harness run: its display name and the function
/// that drives it against the shared `App`.
pub type TestSpec = (&'static str, fn(&mut App));

// ---------------------------------------------------------------------------
// Screenshot-based visual regression testing
// ---------------------------------------------------------------------------
//
// ADR-0060 forbids stripping the plugin stack down to `MinimumPlugins`, so the
// test app renders to a real window on the virtual X display provided by
// `xvfb-run`. We capture that framebuffer with Bevy's built-in `Screenshot`
// API (`bevy::render::view::screenshot`) and compare the result against a
// committed golden image.
//
// This crate MUST NOT depend on `delta-v-app` (cycle). The screenshot helpers
// are pure `std` + `image` + `bevy`, so they live here and are called from the
// test target that already owns the `App`.

/// Resolve the workspace root directory.
///
/// Cargo sets `CARGO_MANIFEST_DIR` to the directory containing the crate's
/// `Cargo.toml`, but does **not** expose the workspace root. This walks up
/// from `CARGO_MANIFEST_DIR` until it finds the directory whose `Cargo.toml`
/// has a `[workspace]` table — that is the workspace root.
///
/// Used to write test artifacts (screenshots, diff images) under the shared
/// workspace `target/` directory, which `.gitignore` already ignores, rather
/// than under a crate-local `target/` that is *not* ignored and would leak
/// build artifacts into version control.
///
/// # Panics
///
/// Panics if no workspace root can be found (the crate is not part of a
/// workspace).
#[allow(
    clippy::panic,
    reason = "workspace_root is only called from test targets; a missing workspace root means the test environment is broken and there is no graceful recovery"
)]
pub fn workspace_root() -> &'static Path {
    use std::sync::OnceLock;
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut dir = manifest.as_path();
        loop {
            let cargo_toml = dir.join("Cargo.toml");
            if cargo_toml.is_file() {
                let contents = fs::read_to_string(&cargo_toml).unwrap_or_else(|e| {
                    // SAFETY: this is the only read of the workspace root
                    // Cargo.toml; if it is unreadable the test environment
                    // is broken and there is no graceful recovery.
                    panic!("failed to read {}: {e}", cargo_toml.display())
                });
                if contents.contains("[workspace]") {
                    return dir.to_path_buf();
                }
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => {
                    // SAFETY: the crate is part of a workspace by
                    // construction (its Cargo.toml has [workspace] in a
                    // parent directory); if we walked all the way up
                    // without finding one, the environment is broken and
                    // there is no graceful recovery.
                    panic!(
                        "could not find workspace root: walked up from {} without finding a Cargo.toml with [workspace]",
                        manifest.display()
                    )
                }
            }
        }
    })
}

/// Directory where screenshot tests write captures and diff images.
///
/// Resolves to `<workspace_root>/target/<suffix>/screenshots`, i.e. under the
/// shared workspace `target/` that `.gitignore` ignores, so the artifacts
/// never leak into version control regardless of which crate's test target
/// runs them.
///
/// `suffix` distinguishes screenshot tests belonging to different `[[test]]`
/// targets (or different crates), so captures from one target never collide
/// with another's. Pass the test target name, e.g. `"ui_fadezone_tests"`.
///
/// The returned `PathBuf` is **not** opened or created; callers must
/// create the directory themselves (e.g. via `fs::create_dir_all`).
#[must_use]
pub fn screenshot_dir(suffix: &str) -> PathBuf {
    workspace_root().join(format!("target/{suffix}/screenshots"))
}

/// Spawn a screenshot capture of the primary window that saves to `path`.
///
/// Uses Bevy's built-in `Screenshot` API
/// (`bevy::render::view::screenshot`). The `Screenshot` plugin is part of
/// `DefaultPlugins` (registered by
/// `bevy_render::view::window::WindowPlugin`), so it is already wired up by
/// `delta-v-app::build_app`.
///
/// The capture completes asynchronously: the spawned entity carries an
/// `observe(save_to_disk(path))` observer that fires when the GPU readback
/// finishes and writes the PNG. The PNG therefore does not exist on the frame
/// the capture is spawned — callers running inside `app.run()` (e.g. the
/// fadezone test) must poll for the file to appear on subsequent frames before
/// comparing it with [`compare_screenshot_files`].
///
/// The target directory is created if it does not exist.
///
/// # Panics
///
/// Panics if the target directory cannot be created.
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "screenshot capture fails the test outright"
)]
pub fn spawn_screenshot(commands: &mut Commands<'_, '_>, path: &Path) {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).expect("failed to create screenshot directory");
    // The observer closure must be `'static` (Bevy observers are `'static`),
    // so the path is cloned into an owned `PathBuf` rather than borrowed.
    #[allow(
        clippy::unnecessary_to_owned,
        reason = "observer closures are 'static; path must be owned"
    )]
    let path = path.to_path_buf();
    // Do NOT insert `Capturing` here: Bevy's `extract_screenshots` system only
    // processes screenshot entities *without* `Capturing` and inserts it
    // itself. Pre-inserting it makes the renderer skip the capture entirely,
    // so the PNG is never written and any poll for it hangs forever.
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}

/// Result of comparing a freshly captured screenshot against a golden image.
pub enum ScreenshotCompare {
    /// No golden image exists yet — the capture was written as the new one.
    Bootstrapped,
    /// Both images exist and are pixel-identical.
    Match,
    /// Both images exist but differ; `actual` was copied to `diff_dir`.
    Mismatch {
        /// Number of pixels that differ between the two images.
        diff_pixels: u64,
    },
}

/// Compare two already-written PNG files pixel-for-pixel (tolerance 0).
///
/// Takes the paths of two images that have already been captured to disk — it
/// does not drive the `App`. Intended for tests running inside `app.run()` that
/// spawn a screenshot capture on an earlier frame and poll for the PNG to
/// appear on a later frame before calling this.
///
/// # Panics
///
/// Panics if either image cannot be decoded.
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "screenshot capture fails the test outright"
)]
#[must_use]
pub fn compare_screenshot_files(expected_path: &Path, actual_path: &Path) -> ScreenshotCompare {
    let expected = image::open(expected_path).unwrap_or_else(|e| {
        panic!(
            "failed to open golden image {}: {e}",
            expected_path.display()
        )
    });
    let actual = image::open(actual_path).unwrap_or_else(|e| {
        panic!(
            "failed to open captured screenshot {}: {e}",
            actual_path.display()
        )
    });

    if expected.dimensions() != actual.dimensions() {
        let msg = format!(
            "screenshot dimensions differ: expected {}x{}, got {}x{}",
            expected.width(),
            expected.height(),
            actual.width(),
            actual.height()
        );
        fail(&msg);
        return ScreenshotCompare::Mismatch { diff_pixels: 0 };
    }

    let diff_pixels = expected
        .pixels()
        .zip(actual.pixels())
        .filter(|(a, b)| a != b)
        .count() as u64;

    if diff_pixels == 0 {
        ScreenshotCompare::Match
    } else {
        let msg = format!(
            "screenshot differs from golden image by {diff_pixels} px (tolerance 0 px); \
             actual screenshot written to {}",
            actual_path.display()
        );
        fail(&msg);
        ScreenshotCompare::Mismatch { diff_pixels }
    }
}

/// Tracks the screenshot-capture state across the multiple frames a test
/// system runs.
///
/// It records whether the capture has been spawned and whether this is a
/// bootstrap run (no golden image yet). Both decisions must persist across
/// frames — recomputing `bootstrap` every frame would flip to non-bootstrap
/// the moment the PNG lands in `expected_path`.
// allow-default: ScreenshotCaptureState is a test-only Bevy Resource, not JSON-backed.
// ADR-0013/ADR-0039 Default ban applies only to JSON-deserialized types.
#[derive(Resource, Default)]
pub struct ScreenshotCaptureState {
    /// Whether the screenshot capture has been spawned.
    pub spawned: bool,
    /// Whether this is a bootstrap run (no golden image exists yet).
    pub bootstrap: bool,
}

/// Configuration for a screenshot test run.
pub struct ScreenshotTestConfig {
    /// Directory containing the golden (expected) images.
    pub expected_dir: PathBuf,
    /// Name of the expected/golden image file (e.g., "`fade_zones_visible.png`").
    pub expected_filename: &'static str,
    /// Suffix for the diff directory (e.g., "`ui_fadezone_tests`").
    pub diff_suffix: &'static str,
    /// Human-readable test name for log messages (e.g., "`fade_zones_visible`").
    pub test_name: &'static str,
}

/// Run the screenshot capture lifecycle inside a Bevy system.
///
/// This function encapsulates the common pattern used by visual regression tests:
/// 1. On first call, detect whether this is a bootstrap run (golden image missing).
/// 2. Spawn a screenshot capture to the appropriate path (golden dir for bootstrap,
///    diff dir for normal runs).
/// 3. Poll for the PNG to appear on subsequent frames.
/// 4. On bootstrap: exit success (the capture becomes the new golden image).
/// 5. On normal run: compare against golden image, exit success on match, exit
///    failure on mismatch.
///
/// The caller is responsible for:
/// - Adding `ScreenshotCaptureState` as a resource to the app (via `init_resource`).
/// - Calling this function from a system that runs in the `Update` schedule.
/// - Running any test-specific assertions *before* calling this function.
///
/// # Parameters
///
/// - `commands`: Bevy commands for spawning the screenshot entity.
/// - `exit_events`: Message writer for `AppExit` events.
/// - `capture_state`: Mutable access to the `ScreenshotCaptureState` resource.
/// - `config`: Configuration for this screenshot test.
///
/// # Panics
///
/// Panics if the screenshot directory cannot be created or if image comparison
/// fails to decode either image.
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "screenshot capture fails the test outright"
)]
pub fn run_screenshot_test(
    commands: &mut Commands<'_, '_>,
    exit_events: &mut MessageWriter<'_, AppExit>,
    capture_state: &mut ResMut<'_, ScreenshotCaptureState>,
    config: &ScreenshotTestConfig,
) {
    let expected_path = config.expected_dir.join(config.expected_filename);
    let diff_dir = screenshot_dir(config.diff_suffix);
    let actual_path = diff_dir.join(config.expected_filename);

    // Bootstrap decision is taken once, on the first frame the system runs,
    // and persisted in `capture_state.bootstrap`: once the PNG lands in
    // `expected_path` it would suddenly exist and we could no longer tell the
    // two runs apart if we recomputed `bootstrap` every frame.
    if !capture_state.spawned {
        capture_state.bootstrap = !expected_path.exists();
    }
    let bootstrap = capture_state.bootstrap;

    // Bootstrap: no golden image yet — the capture becomes the new golden image.
    // Otherwise: capture the actual screenshot into the diff directory.
    let capture_path: &Path = if bootstrap {
        expected_path.as_path()
    } else {
        actual_path.as_path()
    };

    if !capture_state.spawned {
        // In non-bootstrap runs, delete any stale actual screenshot so we don't
        // compare against an old file while waiting for the new capture to complete.
        if !bootstrap && actual_path.exists() {
            std::fs::remove_file(&actual_path).ok();
        }
        spawn_screenshot(commands, capture_path);
        capture_state.spawned = true;
    }

    // The PNG is written asynchronously; wait for it to appear before acting.
    if !capture_path.exists() {
        return;
    }

    if bootstrap {
        // Bootstrap: the freshly captured screenshot is the new golden image.
        println!(
            "{}: bootstrap — golden image written to {}",
            config.test_name,
            expected_path.display()
        );
        exit_events.write(AppExit::Success);
        return;
    }

    match compare_screenshot_files(&expected_path, &actual_path) {
        ScreenshotCompare::Match => {
            println!("{}: screenshot matches golden image", config.test_name);
            exit_events.write(AppExit::Success);
        }
        ScreenshotCompare::Mismatch { diff_pixels } => {
            eprintln!(
                "FAIL: {}: screenshot differs from golden image by {diff_pixels} pixels; actual written to {}",
                config.test_name,
                actual_path.display()
            );
            std::process::exit(1);
        }
        // `compare_screenshot_files` only ever returns Match or Mismatch.
        ScreenshotCompare::Bootstrapped => unreachable!(
            "compare_screenshot_files returned Bootstrapped; this variant is only produced by the bootstrap path"
        ),
    }
}
