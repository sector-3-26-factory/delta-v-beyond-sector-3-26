# ADR-0021: Testing strategy

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

A game engine project mixes pure algorithmic code (good for unit
tests), ECS interactions (test-able by constructing a minimal Bevy
`App`), asset loading (integration tests against real files), and
rendered output (which we will not pixel-compare in CI). We need a
strategy that picks the right tool per layer.

## Decision

Four kinds of tests, all run by `cargo test --workspace` (with extra
flags where noted):

1. **Unit tests**: pure functions, conversions, math, schema-aware
   deserialisation. Fast, deterministic, no I/O. The default.

   Unit tests live in their **own file**, never inline with
   production code. For a source file `foo.rs`, the unit tests live
   in `foo_tests.rs` in the same module and are wired in with:

   ```rust
   // foo.rs
   #[cfg(test)]
   #[path = "foo_tests.rs"]
   mod tests;
   ```

   The test file itself starts with `#![cfg(test)]` and only
   contains test code. Rationale: production files stay focused on
   the production code; reviewers can see at a glance whether a
   file has tests by looking for the sibling `_tests.rs`; large
   test modules do not bloat production files.

2. **Integration tests** (`tests/` directory in each crate): build
   a small Bevy `App`, register the crate's plugin(s) and exercise
   the system behaviour end-to-end without a window. Reads real
   JSON files from a test fixtures directory under the crate's
   `tests/fixtures/`.

3. **Headless smoke tests**: launch the full game binary with
   `--headless` (a flag we will add when relevant) or with a
   minimal feature set, advance N fixed-update ticks, then exit
   cleanly with status 0. Catches "the App does not even start"
   regressions. Runs in CI; the absence of a GPU on the runner is
   handled by Bevy's `MinimalPlugins` or a `--no-render` mode.

4. **Doctests**: every public function whose contract is non-
   trivial carries a usage example in its rustdoc, executed by
   `cargo test`. Documentation that compiles is documentation that
   stays correct.

Conventions:

- Tests live close to the code they test. Tests for `delta-v-physics`
  live in that crate, not in a top-level `tests/` mega-directory.
- Unit tests are in sibling `_tests.rs` files (see above), never
  mixed with production code in the same file.
- Tests do not reach into the file system outside their own
  fixtures directory.
- Tests do not depend on the order they run in or on shared mutable
  global state.
- A failing test in CI blocks the merge per
  [ADR-0003](0003-branching-and-pr-workflow.md). The CI ruleset
  enforces this through required checks.
- Performance tests (benchmarks) use `criterion` and live behind a
  `--features bench` flag so that they do not slow down day-to-day
  `cargo test`.

What we explicitly do **not** test:

- Pixel-perfect rendering. Rendering correctness is judged by eye
  during development.
- Cross-platform bit-exact determinism (per
  [ADR-0017](0017-fixed-timestep-and-determinism.md)).

## Consequences

Positive:

- Each layer has a clear test idiom; contributors do not have to
  invent the pattern each time.
- CI failures point at a specific layer.
- Doctests double as documentation.

Negative:

- Headless smoke tests for a windowing game require some plumbing
  (`MinimalPlugins`, optional render skip). One-time cost.
- Benchmarks behind a feature flag are slightly less discoverable;
  documented in `docs/architecture.md`.

Follow-up:

- A `delta-v-test-support` (internal-only) crate may emerge if
  shared test fixtures grow; not yet.
- A coverage report is nice-to-have, not blocking.
