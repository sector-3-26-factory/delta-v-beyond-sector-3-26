# ADR-0020: Save and load format

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Saving and loading game state needs a format that:

- is consistent with the rest of the project's data formats
  (JSON for data, glTF for 3D),
- is human-readable enough that bugs can be diagnosed by looking at
  a save file,
- survives version changes without silently corrupting data,
- can be written and re-read deterministically.

## Decision

Saved game state and player-created content use the same family of
formats as the rest of the project:

- **Structured state** (sector state, ship state, player inventory,
  mission progress): **JSON**, validated against schemas in
  `assets/json/schema/` per
  [ADR-0012](0012-json-schema-validation.md).
- **3D content authored by the player** (custom ship, station
  layout): **glTF 2.0**.
- **Composite saves** that need both (a snapshot containing custom
  models): a directory under the user data root containing a
  top-level `manifest.json` and referenced glTF files, optionally
  zipped as `.dvsave` (decision deferred; uncompressed directory is
  the v0 format).

Rules:

- Every save file references the game version it was written by.
  Loading a save written by a different version is allowed, with
  validation, and may invoke a migration step. The migration
  framework itself is deferred until we ship the first stable
  release.
- Floating-point values are written with enough precision to round-
  trip exactly (Rust's default `f32` formatting via `serde_json`
  preserves this).
- Identifiers (entity IDs, ship type IDs, ...) inside saves are
  stable strings, not raw integer indices, to survive engine
  changes.
- Saves are written atomically: write to a temporary file alongside
  the target, fsync, rename. A crash mid-write must leave the prior
  save intact.

User data lives under the user data root from
[ADR-0019](0019-asset-pipeline-and-user-content.md). Saves are
**never** written into `assets/`.

## Consequences

Positive:

- One mental model for all data.
- Saves are inspectable, diff-able, and source-controllable by the
  player.
- Migration story is well-defined when we get to it.

Negative:

- JSON is larger than a binary format. Acceptable: the size cost is
  small relative to glTF and audio.
- Schema migrations are real work that we are deferring. We are
  paying for it later, not avoiding it.
- The "directory containing files" layout for composite saves is
  less convenient to share than a single file. We mitigate by
  optionally zipping at export.

Follow-up:

- A small `delta-v-saves` module (or a section of `delta-v-config`)
  centralises the read/write code.
- Schemas for save formats are added incrementally as we have
  things worth saving (deferred until M5 or so).
