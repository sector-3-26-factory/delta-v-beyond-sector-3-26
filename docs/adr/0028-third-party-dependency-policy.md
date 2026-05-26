# ADR-0028: Third-party dependency policy

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Every third-party crate is a permanent commitment: it adds attack
surface, build time, maintenance load, and a licence we have to
live with. A project can drown in dependencies without ever making
a conscious choice about any of them.

We want explicit, automatable criteria for what gets pulled in.

## Decision

**Acceptance criteria**: a crate may be added as a dependency only
when **all** of the following hold:

1. **Licence is compatible with GPL-3.0-or-later** (per
   [ADR-0027](0027-open-source-licensing.md)). Practically this
   means MIT, Apache-2.0, MPL-2.0, BSD-2-Clause/3-Clause,
   Unlicense, CC0, LGPL-3.0-or-later, GPL-3.0-or-later. The
   crate-graph licence audit is enforced by `cargo deny`.
2. **Actively maintained**, defined as at least one of:
   - a release in the last 12 months, **or**
   - a clear, current commit log on the upstream repository, **or**
   - the crate is considered finished and depends only on crates
     that meet the same criteria.
   "Finished and unmaintained" is acceptable as long as security
   updates remain plausible (a single maintainer who responds to
   reports counts).
3. **No `unsafe`** in our own crates without an inline justification
   and a passing test that exercises the unsafe path. Third-party
   crates may use `unsafe` internally; we evaluate when it is
   pervasive.
4. **Carries its weight**: the crate should provide value that we
   would otherwise have to write, and should not be trivially
   replaceable by a few lines of standard library.

**Workflow for adding a dependency**:

- The PR description explains *why* this crate is needed and
  briefly addresses the criteria above.
- `cargo deny check` passes in CI (license, advisories, bans,
  sources).
- The dependency is pinned in
  `[workspace.dependencies]` at the workspace root and inherited
  by member crates via `workspace = true`. Per-crate ad-hoc
  versioning is not allowed.

**`cargo deny` configuration** lives at the workspace root in
`deny.toml`. It enforces:

- An allow-list of licences (above).
- A deny-list of crates we have specifically decided not to use
  (initially empty).
- A check against `RUSTSEC` advisories (see
  [ADR-0029](0029-security-and-supply-chain.md)).
- A restriction on sources (no `git = "..."` dependencies in
  `main`/`dev` without an ADR; vendored sources documented).

**Updating dependencies**:

- `cargo update` is a normal maintenance commit, not a feature.
- Major-version bumps require checking the changelog and may need
  code changes; they merit their own PR.
- The `Cargo.lock` is committed (per
  [ADR-0029](0029-security-and-supply-chain.md)).

## Consequences

Positive:

- Dependency growth is intentional, not incidental.
- The licence audit is automated; surprises do not reach `main`.
- A reviewer has a checklist to point at when an unfamiliar crate
  appears in a PR.

Negative:

- More friction when adding a useful crate; we accept this.
- Reviewing the licence audit output adds CI time and reviewer
  attention. Small cost relative to the benefit.

Follow-up:

- The initial `deny.toml` is added in the same PR as this ADR.
- A short paragraph in `docs/workflow.md` describes the practical
  steps to add a dependency.
