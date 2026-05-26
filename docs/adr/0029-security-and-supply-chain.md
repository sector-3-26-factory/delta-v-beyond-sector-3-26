# ADR-0029: Security and supply chain

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Rust does not protect us from supply-chain compromise: a malicious
update to a transitive dependency runs in our build and in our
binaries. High-profile incidents in other ecosystems (`event-stream`
in npm, `xz-utils` in 2024) make this a real concern, not a
theoretical one.

We are not running a high-value target; we *are* shipping software
to players whose machines we should treat with care.

## Decision

We adopt a small set of practices that are cheap, effective and
automatable. None of them is sufficient on its own; together they
reduce risk significantly.

**Reproducibility**:

- `Cargo.lock` is committed and is the source of truth for
  dependency versions. CI builds use `--locked`.
- The Rust toolchain is pinned by `rust-toolchain.toml`.
- The dev container pins its base image tag and lists explicit
  system package versions where it matters (per
  [`.devcontainer/Dockerfile`](../../.devcontainer/Dockerfile)).

**Auditing**:

- `cargo deny check` runs in CI and enforces:
  - Licence allow-list (per
    [ADR-0028](0028-third-party-dependency-policy.md)).
  - `RUSTSEC` advisory database (vulnerabilities and unsound
    crates).
  - Source restrictions (no `git` deps without an ADR; vendored
    sources documented).
  - Crate ban-list (initially empty).
- A scheduled CI run (weekly) re-runs `cargo deny` against `dev`
  so that new advisories surface promptly even without commits.

**Reducing attack surface**:

- New dependencies are added under the criteria in
  [ADR-0028](0028-third-party-dependency-policy.md).
- We prefer std and our own implementations over micro-crates that
  add complexity for trivial functionality.
- `unsafe` in our own code requires an ADR or an inline rationale
  comment and a test.

**Authentication and integrity**:

- Maintainers commit and tag with signed commits / annotated tags
  where practical.
- Release artifacts ship with a `SHA256SUMS` file (per
  [ADR-0026](0026-release-process.md)).
- Long-term: tag signing with a project key. Not blocking for the
  first releases.

**Secrets**:

- We do not commit secrets, tokens, keys or user credentials.
- The dev container does not include credentials; the user mounts
  their own.
- CI uses GitHub-managed secrets (none required at the time of
  this ADR).

**What is out of scope**:

- Sandboxing or code-signing of the released game binaries
  (operating-system concerns beyond the project).
- Reproducible builds in the strict sense (bit-identical output
  across builds). We are pinned and locked, which is enough for our
  purposes.

## Consequences

Positive:

- Known vulnerabilities are surfaced in CI, not by someone
  noticing on Twitter weeks later.
- Builds across machines and CI runs use the same dependency
  versions.
- The project is honest about its threat model and does not over-
  claim guarantees it does not provide.

Negative:

- `cargo deny` adds CI time (small) and occasional friction when
  a new advisory or licence appears (intended).
- The weekly scheduled run requires noticing and acting on its
  results; a missed week is a real risk we accept.

Follow-up:

- The initial `deny.toml` is added in the same PR as this ADR.
- A `security.md` describing how to report vulnerabilities is added
  once the project has external users.
