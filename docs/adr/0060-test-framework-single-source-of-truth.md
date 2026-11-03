# ADR-0060: Test Framework Single Source of Truth for Plugin Stack

<!--
Agents and contributors: read AGENTS.md at the repository root before
working with this document. AGENTS.md links to the ADR index and to all
rules that govern this project.
-->

- **Status**: Accepted
- **Date**: 2026-10-14
- **Deciders**: Cute-Donkey
- **Supersedes**: None
- **Superseded by**: None

## Context

Integration tests have historically used `MinimalPlugins` or manually assembled plugin stacks that differ from the production application. This causes:

1. **Code drift**: Tests pass but production fails because the plugin stack differs
2. **Incorrect test paths**: Tests don't exercise the same system order as production
3. **Maintenance burden**: Every plugin change must be mirrored in multiple test files

CI pipelines run with xvfb (virtual framebuffer), so the production plugin stack runs unchanged — no headless-specific plugin configuration needed.

Per ADR-0051, `delta-v-core` must NOT depend on domain crates. Therefore the central app builder cannot live in `delta-v-core` — it needs a dedicated crate that registers all domain plugins.

## Decision

We establish a **Single Source of Truth** for the plugin stack and system order:

1. **New crate `delta-v-app`** exports `build_app(config: AppConfig) -> App` — the ONLY function that registers the full plugin stack
2. **Both `main.rs` and ALL integration tests** MUST use `delta_v_app::build_app`
3. **Tests run with xvfb** (virtual framebuffer) in CI, so the exact same plugin stack runs in production and tests
4. **FORBIDDEN**: Using `MinimumPlugins`, manually assembling plugin lists, or rebuilding the plugin stack in integration tests
5. **FORBIDDEN**: Any integration test that does not use `build_app` from `delta-v-app`

The `AppConfig` struct centralizes all configuration:
- `asset_root_suffix: String` — "assets" (production) or "tests/fixtures/assets" (tests)
- `world_name: Option<String>` — world to load
- `log_filter: Option<String>` — optional log filter override

## Consequences

### Positive
- **Zero drift**: Production and tests use identical plugin registration order
- **No headless mode needed**: xvfb provides virtual display, so full plugin stack runs unchanged
- **Single maintenance point**: Plugin order changes in one place (`delta-v-app`)
- **Test fixtures isolated**: Tests use `tests/fixtures/assets/` suffix; schemas always from production via `schema_root()`
- **ADR-0051 compliant**: `delta-v-app` is the only library crate besides binary `delta-v` that registers domain plugins
- **Visual regression testing**: Tests can drive the real app — stroke keys, take screenshots, compare screenshots to detect visual regressions during refactoring

### Negative
- **New crate to maintain**: `delta-v-app` adds a workspace member
- **All existing tests must migrate**: Incremental effort to switch from `MinimalPlugins` to `build_app`
- **Test fixtures required**: Each crate needs `tests/fixtures/assets/` mirroring production structure
- **CI/DevContainer need xvfb**: GitHub Actions `ubuntu-latest` has xvfb pre-installed; DevContainer needs xvfb in Dockerfile

### Follow-up Work
- Create `crates/delta-v-app` with `build_app` and `AppConfig`
- Add `resolve_asset_path`, `resolve_world_path`, `schema_root` to `delta-v-assets`
- Migrate `main.rs` to use `build_app`
- Migrate all integration tests in all crates
- Structure test fixtures under `tests/fixtures/assets/`
- Add xvfb to DevContainer Dockerfile
- Update CI workflow to use `xvfb-run -a cargo test` for physics tests

## Notes

This ADR implements step 0 of the implementation plan for the test framework. The full plan covers crate creation, migration, and fixture structure.
-------