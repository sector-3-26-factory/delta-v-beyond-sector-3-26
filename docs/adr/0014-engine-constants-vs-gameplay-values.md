# ADR-0014: Engine constants vs. gameplay values

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Some numeric values in the game are *gameplay knobs* (a ship's max
thrust, a weapon's damage per shot, a planet's mass). Others are
*implementation constants* of the engine itself (a floating-point
comparison tolerance, the Newtonian gravitational constant G,
mathematical constants like pi).

Mixing these creates two opposite problems:

1. If everything is hard-coded in Rust, gameplay tuning requires
   recompilation and is invisible to content authors.
2. If everything is in JSON, the engine cannot rely on any value at
   all without first checking that some file existed, parsed and
   validated -- including for things that have nothing to do with
   gameplay (epsilon for vector comparisons, conversion factors).

## Decision

We draw an explicit line:

**Engine constants stay in Rust.** They are:

- Mathematical constants (`PI`, `TAU`, `E`).
- Physical constants of nature (`G`, speed of light `c`, etc.) that
  do not change for gameplay reasons.
- Unit conversion factors (`1 AU = 1.495978707e11 m`, etc.).
- Numerical tolerances and algorithm parameters (epsilon for
  comparing vectors, maximum iteration counts in solvers, sentinel
  values inside a system that has no meaning outside it).
- Buffer sizes, hash-table capacity hints and other low-level
  implementation knobs.

These are declared as `pub const` in the crate they belong to and
documented with rustdoc.

**Gameplay values come from JSON, governed by schemas.** They are:

- Per-ship-type: mass, max thrust, hitpoints, hardpoints, cargo
  capacity, sensor range, ...
- Per-weapon: damage, fire rate, projectile mass, projectile
  velocity, ...
- Per-world / sector: dimensions, contained entities, ambient
  parameters.
- Per-settings: keybindings, language, audio volumes, graphics
  options.
- Per-player-progression: starting credits, mission unlock
  conditions, ...

There is no Rust-side default for these. They are required by their
schema; a missing value is an error
([ADR-0013](0013-no-silent-fallbacks.md)).

Edge cases:

- A Rust enum representing a *category* of gameplay choice (e.g.
  `enum DamageType { Kinetic, Energy, ... }`) is fine in code. The
  enum is the closed set of categories; the per-instance values
  (how much kinetic damage this gun does) still come from JSON.
- A Rust `struct` representing a content type often has fields with
  no `Default` impl, precisely because constructing one without
  loading it from JSON would be meaningless. See
  "ECS-Component-Defaults" discussion in the project notes;
  Components for gameplay entities are spawned with explicit values
  from loaded content, not via `Component::default()`.

Compiler-warning interaction:

- Rust will sometimes warn about fields that *appear* unused because
  they are only written from `serde` deserialisation. These warnings
  are suppressed per-field with an inline comment that points to
  this ADR; we never blanket-allow `dead_code` at crate level.

## Consequences

Positive:

- Engine code has access to compile-time constants it needs without
  conditionalising every access on "did the config load".
- Gameplay tuning happens in JSON, observable to designers and AI
  tools, and protected by schemas.
- The boundary is clear and easy to apply on review: "is this a
  gameplay knob or an implementation detail?"

Negative:

- Edge cases will need judgement (is the gravity cutoff radius a
  tuning value or an engine constant?). The default is "if a
  designer might ever care, it goes to JSON".
- Per-field `#[allow]` annotations are slightly verbose but
  intentional; they make the deserialisation contract explicit.

Follow-up:

- A small style guide in `docs/architecture.md` lists common cases
  and which side of the line they fall on, updated as we hit edge
  cases.
