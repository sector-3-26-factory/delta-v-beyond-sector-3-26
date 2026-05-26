# ADR-0025: Versioning

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

A version number is a promise. Until we make a promise we can keep,
inflating the version number is misleading. We also need a rule for
when to bump and tag, so that "what version is this" has an
unambiguous answer.

## Decision

We follow **Semantic Versioning 2.0.0** with one project-specific
addition.

Pre-1.0 phase:

- The current and foreseeable phase is `0.x.y`. Per SemVer's own
  rules for the major-zero range, **anything may change between
  minor releases**.
- `0.MINOR.0` bumps occur at milestones that produce a meaningfully
  different game (e.g. M1 done, M2 done). Patch (`0.MINOR.PATCH`)
  bumps are for fixes between milestones.
- We do **not** tag every commit. Tags happen at *playable* builds
  (per the project's intent), not at every internal milestone. The
  decision of "is this a tag-worthy build" is taken by the
  maintainer at the time.

Post-1.0 phase (future):

- `1.0.0` ships when there is a first stable, playable, multiplayer-
  capable build that we are willing to call done.
- After 1.0, MAJOR bumps mark breaking changes to save format,
  configuration format or network protocol; MINOR adds compatible
  features; PATCH is bug-fix-only.
- "Compatible" specifically includes "old save files load correctly,
  with migration if necessary". Breaking this contract requires a
  MAJOR bump.

Tagging:

- Annotated tags only (`git tag -a vX.Y.Z -m "..."`), never
  lightweight tags. They are signed if the maintainer has a key.
- Tag names: `vX.Y.Z` (lowercase `v` prefix, dot-separated).
- A tag exists on `main` only; `dev` is not tagged.

Crate versions:

- All workspace crates share one version, bumped together via
  workspace inheritance.
- The version field in `Cargo.toml` is the authoritative source;
  the binary reports `env!("CARGO_PKG_VERSION")` on startup
  (per [ADR-0015](0015-logging-strategy.md)).

## Consequences

Positive:

- Players and packagers understand the version contract.
- "Which milestone is this" maps onto a version, which is searchable
  in git history.
- We do not over-commit by jumping to 1.0 prematurely.

Negative:

- The "only at playable builds" tagging rule means there will be
  long stretches without a new tag. The git history still answers
  "what changed since" in detail.
- Aligning all crates to one version is slightly less idiomatic for
  Rust workspaces but reduces coordination friction at the small
  scale of this project.

Follow-up:

- A `CHANGELOG.md` is added when there is something to release.
  Generated from Conventional Commits (per
  [ADR-0004](0004-commit-message-convention.md)) at tag time.
- The release process itself is described in
  [ADR-0026](0026-release-process.md).
