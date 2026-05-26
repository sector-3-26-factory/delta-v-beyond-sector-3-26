# ADR-0035: Hot reload of configs in dev builds

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Tuning configuration (keybindings, physics constants exposed in
JSON, audio mix, ...) is significantly faster when changes take
effect without restarting the game. Bevy supports asset hot-reload
through `bevy_asset`, and our configuration loader will treat config
files as assets.

In release builds we explicitly do not want hot-reload: it adds
moving parts, can mask bugs ("the game ran fine *because* I had
just changed the file") and the player is not editing config files
live.

## Decision

**Hot-reload is enabled in builds with `--features dev`, disabled
otherwise.**

Rules:

- The `delta-v` binary exposes a `dev` cargo feature. It enables:
  - Bevy's asset hot-reload (`AssetPlugin { watch_for_changes: ... }`
    or the modern equivalent).
  - Hot-reload of JSON config files under `assets/config/` and the
    corresponding user-config files.
  - Bevy's `dynamic_linking` feature (already used today for fast
    incremental builds, see `Cargo.toml`).
  - A few extra debug overlays (deferred until they exist).
- Without `--features dev`, configuration is loaded once at startup
  and on explicit reload commands (e.g. a future "Apply" button in
  the settings UI).
- Hot-reload applies only to **safe-to-swap** configuration: data
  that can be replaced without restructuring the ECS world.
  Examples: keybindings, language, audio mix, gameplay-tuning
  numbers.
  Hot-reload does **not** apply to: world definitions (which
  require respawning entities), schema files themselves (they are
  loaded into the validator once), engine-startup configuration
  (window size, renderer backend).
- A reload that fails validation (per
  [ADR-0012](0012-json-schema-validation.md)) does **not** apply.
  The previous valid configuration remains in effect and an
  `ERROR`-level log entry is emitted (per
  [ADR-0015](0015-logging-strategy.md)) naming the offending
  file and field. The game does not fall back silently (per
  [ADR-0013](0013-no-silent-fallbacks.md)).

## Consequences

Positive:

- Iteration speed during development is dramatically better.
- Designers and AI tooling can experiment with content changes
  live.
- Release builds are not affected by file-watcher overhead or
  watcher-related bugs.

Negative:

- Two code paths (with and without hot-reload) for the
  configuration loader. We minimise duplication with feature-
  gated initialisation.
- Bevy asset hot-reload sometimes interacts badly with editors
  that write files in two steps (temp file + rename); we rely on
  Bevy's debouncing and accept the occasional spurious reload.

Follow-up:

- The `dev` feature is declared in the workspace and re-exported
  by `delta-v` and `delta-v-config`. Per
  [ADR-0002](0002-repository-layout-and-workspace.md), feature
  flags propagate through the workspace.
- The list of "safe-to-swap" config kinds is maintained alongside
  the corresponding loader code, not duplicated here.
