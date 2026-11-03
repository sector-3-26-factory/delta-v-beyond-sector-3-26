# ADR-0061: Test Framework Architecture

- **Status**: Accepted
- **Date**: 2026-10-29
- **Deciders**: Cute-Donkey
- **Supersedes**: (none)

## Context

Bevy 0.19's `WinitPlugin` caches the `EventLoop` in a process-global static.
Only one `App` with `DefaultPlugins` can exist per process. This forces a
`harness = false` model: one `[[test]]` target per test scenario, each
producing a standalone ~2.1 GB binary that statically embeds the entire
Bevy dependency chain.

`crates/delta-v-physics` alone declares 26 such targets (~55 GB). Across
the workspace: 20+ binaries × 2.1 GB ≈ 42+ GB. The `target/` directory
grew to 259 GB.

Earlier strategies all failed:
- Thread-per-scenario: winit panics on non-main-thread event loop
  creation; `run_on_any_thread: true` only allows creation on any thread,
  not multiple event loops in one process.
- Sequential apps on main thread: `RecreationAttempt` — the global static
  is never released.
- Smaller binaries via profile tweaks: reduces size, not count.
- Subprocess IPC (Section 3 of the POC): works but needs a fragile
  file-based protocol with race conditions.

## Decision

We use a **test function registry** architecture:

1. **One production binary** (`delta-v`) that can run any registered test
   function by name via `--test-function <name>`.
2. **Per-domain test crates** (`crates/delta-v-physics-tests`, etc.) that
   hold test functions compiled into the binary behind a `test` feature
   flag. Each crate depends on its domain crate + `delta-v-app` +
   `delta-v-test-framework`.
3. **The test function registry** — owned by `delta-v-test-framework`, a
   technical crate (only depends on `bevy`, `delta-v-types`, `image`,
   `leafwing-input-manager`). It exposes `TestEntry`, the
   `Mutex<Vec<RegisteredEntry>>` global registry, `register`,
   `all_tests()`, and the `register_test!` macro. Entries are leaked via
   `Box::leak` and registered manually — no `inventory` crate (its
   cross-crate static initialiser is unreliable with `rlib` linkage).
4. **Normal `#[test]` drivers** in each test crate's `tests/` directory
   spawn the binary and check the exit code. No `xvfb` needed for the
   driver itself.

### Crate dependency graph

```
delta-v-test-framework   (depends on: bevy, types, image, leafwing; owns the registry)
    ↑
delta-v-physics-tests  (depends on: physics, test-support, app[test])
    ↑
delta-v binary         (test feature enables physics-tests + test-support)
```

No cycles. `delta-v-test-framework` no longer touches any domain crate.

### Directory structure (per domain)

```
crates/<domain>-tests/
  Cargo.toml
  README.md
  src/
    lib.rs
    real_app_ITs/
      mod.rs                    # init_registry()
      <category>/
        <test_group>/
          mod.rs                # test functions + init_registry()
  tests/
    real_app_ITs/
      <category>/
        <test_group>/
          test_driver.rs        # #[test] that spawns binary
          fixtures/
            assets/
```

Test functions, drivers, and fixtures for a given test live in the same
crate and follow matching directory paths — tight co-location without
scattering across the project.

## Consequences

### Positive
- **One binary** replaces 20+ `harness = false` targets. Disk usage drops
  dramatically.
- **Test functions have full access to `&mut App`** — same as the current
  `run_harness` model, with the same real plugin stack via `build_app`.
- **Normal `cargo test` harness** — each test is a `#[test]` function
  that spawns the binary. No `xvfb` needed for the driver.
- **Per-domain isolation** — each domain crate owns its own tests,
  satisfying ADR-0051 (domain crates don't depend on each other).
- **Tight co-location** — test functions, drivers, and fixtures for a
  given test live in the same crate with matching directory paths.

### Negative
- **Two-step testing**: the binary must be built with `--features test`
  before the driver tests can spawn it. `cargo test` on a test crate
  implicitly builds the binary first.
- **Slower feedback**: each test spawns a new process (full plugin stack
  init) rather than sharing an `App` across tests in one process.
- **Leaked entries**: `Box::leak` means test entries are never freed.
  Acceptable because registration happens once at startup.
- **Unsafe code**: `RegisteredEntry` wraps a raw `*const TestEntry`
  with manual `unsafe impl Send + Sync`. Justified because entries are
  immutable after registration.

### Follow-up work
- Convert existing `harness = false` tests in `delta-v-physics` and other
  domain crates to the per-domain test crate structure.
- Add more per-domain test crates (`delta-v-ships-tests`, etc.) as
  existing tests are migrated.

## Notes

- ADR-0060 (`build_app` as single source of truth) is unchanged — this
  ADR governs the *test framework* that consumes `build_app`, not the
  plugin stack itself.
- The registry API (`TestEntry`, `register`, `all_tests`,
  `register_test!`) is the stable public surface of
  `delta-v-test-framework`. The `test` feature on `delta-v-test-framework`
  is an empty feature flag that exists only so the binary can
  conditionally enable registration without adding a dependency edge to
  a domain crate.
