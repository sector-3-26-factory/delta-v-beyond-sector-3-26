# ADR-0010: Configuration system

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

The game has several kinds of configuration:

1. **Content defaults** that ship with the game: ship types, planet
   templates, default keybindings, default audio settings, default
   language. These live under `assets/config/` (and equivalent
   subdirectories) and are part of the repository.
2. **User overrides**: the player's personal preferences, calibration
   data, custom keybindings. These live in the user's home directory
   and never touch the repository.
3. **Per-session state**: in-memory only; not persisted as
   configuration.

We need one consistent loader, validator and merge story for all of
the above.

## Decision

We adopt a layered configuration model:

- **Default layer**: JSON files under `assets/config/`. Always
  present, validated against JSON Schemas (see
  [ADR-0012](0012-json-schema-validation.md)). Missing or invalid
  defaults are a hard error (see
  [ADR-0013](0013-no-silent-fallbacks.md)).
- **User layer**: JSON files under
  `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/`
  (or `~/.config/delta-v-beyond-sector-3-26/` if `XDG_CONFIG_HOME`
  is unset) on Linux, with platform-appropriate equivalents on other
  OSes. Optional; absent files mean "no overrides".
- **Effective configuration** is computed by deep-merging the user
  layer on top of the default layer, key by key. The result is
  re-validated against the schema.

Merge rules:

- Objects: merged recursively; user keys override defaults; defaults
  not mentioned in user file remain in effect.
- Arrays: **replaced wholesale** by the user value (no element-wise
  merge). This avoids ambiguity in "should the user array append,
  replace or merge".
- Scalars and `null`: user value replaces default.

Library choices:

- JSON parsing: `serde_json` (no JSON5; comments are not used; see
  [ADR-0008](0008-physical-units-in-json.md) for how content stays
  self-documenting).
- Schema validation: a Rust JSON-Schema validator (`jsonschema`
  crate) running both defaults and effective configs through the
  schemas.

Loading model:

- At startup, the default world is loaded automatically; the user
  can later trigger loading another world. Both go through the same
  loader.
- For settings (keybindings, language, audio device): the user layer
  is loaded once at startup and reloaded on demand. With
  `--features dev`, hot-reload is active (see
  [ADR-0035](0035-hot-reload-of-configs.md)).
- The first time the user changes settings through an in-game UI
  (later milestone), the application writes the **complete merged
  state** to the user layer, not a diff. From that point on, the
  user file is the authoritative override source for that
  particular settings file.

Write rules for user files:

- The application creates the user-config directory on demand with
  restrictive permissions (`0700`) on Unix-like systems.
- Existing user files are never overwritten silently; the writer
  always rewrites in full and either atomically replaces or backs up
  the previous version (decision deferred to the implementing PR).

## Consequences

Positive:

- Defaults are tracked in the repository and reviewable.
- User configuration is decoupled from the install location; updates
  do not destroy user preferences.
- Per-key override gives a frictionless authoring loop (one
  keybinding can be changed without copying the full file).
- One loader/validator for the whole project.

Negative:

- The merge logic must be precise and tested; subtle merge bugs are
  notoriously hard to diagnose.
- "Defaults moved or renamed" between versions is a real migration
  problem; until we have a migration system, schema changes can
  silently invalidate user files. A future ADR may add explicit
  config versioning.

Follow-up:

- The keybindings configuration uses this system; see
  [ADR-0011](0011-keybindings-configuration.md).
- User content (worlds, ships) follows a related but separate scheme;
  see [ADR-0019](0019-asset-pipeline-and-user-content.md).
