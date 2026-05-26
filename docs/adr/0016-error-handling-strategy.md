# ADR-0016: Error handling strategy

- **Status**: Accepted (initial; refine as cases appear)
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Rust's `Result` / `Option` / `panic!` give us three different ways to
express failure, plus the ecosystem split between `anyhow` (one
opaque error type for everything) and `thiserror` (one typed error
per crate). Without a project rule we get inconsistency:
library-style `?` chains in binaries, untyped errors in libraries,
silent `unwrap()` everywhere.

## Decision

We split errors into three categories:

1. **Programmer mistakes** -- the kind of failure that means our
   *code* is wrong, not that the world is being uncooperative. Index
   out of bounds in a buffer we just sized, a `match` arm that was
   supposed to be unreachable, a precondition we wrote and then
   violated ourselves. These use `panic!`, `unreachable!()`,
   `assert!()`, `debug_assert!()`. They must never be reached at
   runtime; if they are, that is a bug to fix, not a condition to
   recover from.

2. **Recoverable failures inside library crates** (`delta-v-core`,
   `delta-v-config`, `delta-v-physics`, ...) -- file not found,
   JSON validation failure, network disconnect. These use **typed
   errors** declared with `thiserror`:

   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum ConfigError {
       #[error("config file not found: {path}")]
       NotFound { path: PathBuf },
       #[error("schema validation failed: {pointer}: {reason}")]
       Schema { pointer: String, reason: String },
       // ...
   }
   ```

   Callers can pattern-match and react if needed.

3. **Recoverable failures at the binary level** (`delta-v` /
   `main.rs`) -- by the time errors reach the application top, the
   typed error structure has usually already been used by an
   intermediate layer. The binary can collapse to `anyhow::Result`
   for ergonomics when it makes sense, but only at the outermost
   boundary.

Rules:

- **No `unwrap()` or `expect()` outside of tests and `main`-level
  binary code unless followed by a `// SAFETY: ...` or
  `// INVARIANT: ...` comment explaining why the call cannot fail.**
- **No `?` that throws away context.** Use `.map_err(...)` or
  `.context(...)` (anyhow) / a typed variant (thiserror) to
  preserve the *what was I doing* information.
- **Errors at startup are fatal and verbose.** If a defaults file
  fails to load, the program exits with a non-zero status and a
  multi-line, human-readable message naming the file and the
  failing field.
- **Errors during gameplay are logged and recovered from where
  possible.** A failed asset load that has a `--features dev`
  placeholder fallback is logged at `WARN` (see
  [ADR-0015](0015-logging-strategy.md)). A failed asset load in a
  release build follows
  [ADR-0013](0013-no-silent-fallbacks.md).
- **Bevy systems return `()` by default**, so `?` does not work in
  them directly. We use the established pattern of a thin wrapper
  that logs the error and continues, or upgrade to Bevy's
  `Result`-returning system signatures where appropriate.

## Consequences

Positive:

- Library APIs have descriptive, matchable errors; CLI/binary code
  stays ergonomic.
- A consistent rule lets reviewers reject "drive-by `unwrap()`s".
- Crashes carry context; silent recoveries do not exist.

Negative:

- `thiserror` enums in every library crate add boilerplate.
  Acceptable for the clarity gained.
- Discipline required: it is always tempting to `.unwrap()` "just
  for now". The lint and review process catches it.

Follow-up:

- A small style example lives in `docs/architecture.md` showing the
  canonical `thiserror` pattern for the project.
- A clippy configuration (see
  [ADR-0023](0023-code-style-and-lints.md)) denies `unwrap_used`
  and `expect_used` in non-test code.
