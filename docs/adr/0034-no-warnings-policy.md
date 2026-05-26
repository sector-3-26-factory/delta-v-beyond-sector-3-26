# ADR-0034: No warnings policy

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

A project that tolerates compiler and linter warnings accumulates
them. Within a few months the output is so noisy that genuinely
new warnings hide in the scroll-back. The classic solution is the
**broken-window rule**: warnings are always zero, every warning is
either fixed or explicitly acknowledged.

This ADR formalises that rule for our project.

## Decision

**No commit may introduce warnings.**

Concretely:

1. The build `cargo build --workspace --all-targets` must emit
   **zero** warnings.
2. The lint `cargo clippy --workspace --all-targets -- -D warnings`
   must pass without changes. (`-D warnings` already turns clippy
   warnings into errors; this ADR makes it the project policy.)
3. CI configures `RUSTFLAGS="-D warnings"` for `cargo check` as
   well, so that even non-clippy compiler warnings break the
   build.
4. **Acknowledging** an unavoidable warning is done with the
   narrowest possible `#[allow(...)]`, **with an inline comment
   explaining why**. Examples:

   ```rust
   // `Mass` is read only via serde deserialisation; the field is
   // not unused, but rustc does not see the deserialiser.
   #[allow(dead_code)]
   mass: Mass,
   ```

5. Blanket allows at the crate level (`#![allow(...)]` in `lib.rs`)
   are only permitted for lints that we have explicitly listed in
   [ADR-0023](0023-code-style-and-lints.md). New crate-level
   allows require an ADR amendment.
6. The `cargo deny check` job
   (per [ADR-0028](0028-third-party-dependency-policy.md),
   [ADR-0029](0029-security-and-supply-chain.md)) must also pass.
   `cargo-deny` warnings count as warnings.

**Pre-commit discipline**:

- Before committing, run `cargo fmt --all && cargo clippy
  --workspace --all-targets -- -D warnings && cargo test --workspace`.
- An optional Git hook script is provided in `scripts/git-hooks/`
  to automate this; installing it is documented in
  `docs/workflow.md`.

**Exemptions**:

- Test code may relax `#![deny]`s with localised `#[allow(...)]`
  (per [ADR-0023](0023-code-style-and-lints.md)).
- Generated code and vendored code are out of scope.
- Compiler-version transition warnings (new lints introduced by a
  newer rustc) are fixed in a dedicated PR within a reasonable
  time, not left to accumulate.

## Consequences

Positive:

- Warnings remain meaningful: when one appears, it is real news.
- New contributors get immediate, mechanical feedback.
- The "broken windows" cycle never starts.

Negative:

- Occasional friction: a `clippy::pedantic` lint that fires on a
  legitimate pattern must be silenced with a localised allow and
  comment. This is intended behaviour, not a flaw.
- A toolchain bump may surface many new warnings at once; the
  policy turns them into a forcing function to address them rather
  than ignore them.

Follow-up:

- The CI workflow already sets `-D warnings`; that is preserved.
- The pre-commit hook script is added as part of the foundation
  PR, with installation instructions in `docs/workflow.md`.
