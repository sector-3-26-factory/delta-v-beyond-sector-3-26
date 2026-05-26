# ADR-0002: Repository layout and workspace

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

The project is a non-trivial game with several technical concerns
(physics, configuration, networking, assets) and several content
domains (ships, propulsion, weapons, stations, items, world). Putting
everything in a single crate would:

- slow down incremental compilation (Rust recompiles a whole crate on
  any change to it),
- allow accidental dependency cycles between domains,
- make it hard to enforce that, for example, the weapons domain may
  depend on physics but not vice versa.

A Cargo workspace with multiple crates avoids all three problems at the
cost of more `Cargo.toml` boilerplate.

## Decision

We use a Cargo workspace with a binary crate and a set of library
crates, split along technical and domain boundaries.

Layout:

```
crates/
  delta-v/               # binary; only App composition + plugin registration
  delta-v-core/          # ECS fundamentals, shared components, plugin traits
  delta-v-config/        # JSON loading, schema validation, user-override merge
  delta-v-physics/       # Newtonian physics, gravity, floating origin (on avian3d)
  delta-v-assets/        # asset loader extensions, glTF helpers
  delta-v-net/           # networking (placeholder until ADR-0030/0031 are decided)
  delta-v-ships/         # ship types + ship-specific systems
  delta-v-propulsion/    # thrusters, hyperdrive
  delta-v-weapons/       # weapons
  delta-v-stations/      # space stations and station components
  delta-v-items/         # collectable items
  delta-v-world/         # sectors, worlds, hyperspace gates
```

Conventions:

- Crate names use **hyphens** in `Cargo.toml` (`delta-v-ships`); the
  derived module identifier uses **underscores** (`delta_v_ships`).
  This is the Rust ecosystem default.
- Every domain crate exports exactly one Bevy `Plugin` named after the
  crate (`ShipsPlugin`, `WeaponsPlugin`, ...). The binary registers all
  plugins explicitly in one place. See
  [ADR-0005](0005-plugin-architecture.md).
- Dependency rule of thumb: domain crates may depend on technical
  crates and on each other only in one direction, never circularly.
  When two domain crates want to know about each other, the shared
  surface goes into `delta-v-core`.
- `delta-v-net` and any other crate that is not yet implemented exists
  as a minimal stub (empty plugin, no public items) so that the
  workspace graph is complete from day one.

## Consequences

Positive:

- Better parallel build performance; a change in `delta-v-weapons` does
  not recompile `delta-v-physics`.
- Cycles between crates are prevented by the compiler, not by review
  discipline.
- Each domain has a natural place for unit tests and for documentation.
- Splitting or merging crates later is much cheaper than starting flat
  and reorganising.

Negative:

- Twelve `Cargo.toml` files to maintain; bumping a dependency means
  touching several files (mitigated by `[workspace.dependencies]`).
- More moving parts for newcomers; the workspace structure must be
  documented in `docs/architecture.md`.
- Some boilerplate per crate (plugin stub, `lib.rs` header, etc.).

Follow-up:

- All shared dependency versions are pinned in the workspace root
  `Cargo.toml` under `[workspace.dependencies]` and inherited by
  members via `workspace = true`.
- The workspace root keeps `rust-toolchain.toml`, `deny.toml`, CI
  config, and other repository-wide files.
