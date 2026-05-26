# ADR-0023: Code style and lints

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

A consistent style is a courtesy to the next reader (often the
author, in three months). Rust's `rustfmt` and `clippy` cover most
of it for free, but their default configurations leave room for
project preference.

We also want the lint configuration to reinforce the rules from
other ADRs: no silent fallbacks ([ADR-0013](0013-no-silent-fallbacks.md)),
no warnings in committed builds ([ADR-0034](0034-no-warnings-policy.md)),
discouraged `unwrap()` ([ADR-0016](0016-error-handling-strategy.md)).

## Decision

**Formatting**: `cargo fmt --all` is mandatory and configured by a
workspace-root `rustfmt.toml`. The configuration is intentionally
close to defaults so that contributors do not have to think:

```toml
# rustfmt.toml
edition = "2021"
max_width = 100
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
reorder_imports = true
use_field_init_shorthand = true
```

Anything else is `rustfmt` default.

**Linting**: `cargo clippy --workspace --all-targets -- -D warnings`
must pass. In addition, each crate's `lib.rs` (or `main.rs`)
enables a curated set of lints:

```rust
#![warn(
    missing_docs,                       // see ADR-0024
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::cargo,
)]
#![deny(
    clippy::unwrap_used,                // see ADR-0016
    clippy::expect_used,                // see ADR-0016
    clippy::panic,                      // panic only via assertions / unreachable
    clippy::indexing_slicing,           // prefer get / iter
    clippy::todo,                       // no TODO left in committed code
    clippy::unimplemented,              // same
    clippy::dbg_macro,
)]
#![allow(
    clippy::module_name_repetitions,    // common in plugin-per-crate layouts
    clippy::must_use_candidate,         // noisy in ECS-heavy code
    clippy::missing_errors_doc,         // rustdoc on errors is encouraged but not blocking
)]
```

Test code: the `#![deny]`s above are allowed to be relaxed in test
modules via `#[allow(clippy::unwrap_used)]` etc., but only inside
`#[cfg(test)]` blocks.

**Naming**:

- Types: `UpperCamelCase`.
- Traits: `UpperCamelCase`; verb-y when behaviour, noun-y when role.
- Functions and methods: `snake_case`.
- Constants and statics: `SCREAMING_SNAKE_CASE`.
- Crate names: `kebab-case` in `Cargo.toml`, `snake_case` in
  imports.
- Files and modules: `snake_case.rs`.
- Test functions: `fn test_<thing>_<expected>()` or
  `fn <thing>_<expected>()` (consistent within a file).

**File and module organisation**:

- Each crate has a `lib.rs` (or `main.rs`) that re-exports the
  public API and registers no logic itself.
- Module files prefer a flat `mod foo;` plus `foo.rs` rather than
  `foo/mod.rs`.

## Consequences

Positive:

- Code looks the same regardless of who wrote it (human or AI).
- Lints catch a large class of bugs and bad habits without manual
  review.
- The lint set encodes other ADRs, so a contributor who has not
  read them still gets a hint at compile time.

Negative:

- `clippy::pedantic` is opinionated; we will sometimes need
  per-call `#[allow(...)]` with a comment explaining why.
- A new clippy version may introduce a new lint that fires on
  existing code. We treat this as a bump that includes the
  necessary fixes or allows.

Follow-up:

- A `rustfmt.toml` and per-crate lint preamble are added in the
  same PR as this ADR's acceptance.
- The CI `clippy` invocation includes `--workspace --all-targets`
  to cover tests, benches and examples too.
