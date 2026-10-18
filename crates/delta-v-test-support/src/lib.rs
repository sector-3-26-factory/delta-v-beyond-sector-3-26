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

use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;
use delta_v_types::LogicalAction;
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
/// sub-app on the first `update()`), then runs 20 updates to let the
/// render graph settle. Subsequent calls skip the `finish()`/`cleanup()`
/// step and only run the update loop, so it is safe to call defensively
/// from test helpers without worrying about whether the caller already
/// settled the render graph.
///
/// The `finish()`/`cleanup()` step is guarded by a thread-local flag
/// because Bevy's `WinitPlugin` event loop is thread-local: it can only
/// be created once per thread, so calling `cleanup()` on a second `App`
/// in the same thread panics with `RecreationAttempt`.
pub fn settle_render(app: &mut App) {
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
    for _ in 0..20 {
        app.update();
    }
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
/// `build_app` and `settle_render` are called by the caller before
/// invoking this function; this helper only owns the pass/fail bookkeeping
/// and the shared `App`.
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
