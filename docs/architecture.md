# Architecture overview

This document is the big-picture map of the project. It is
intentionally short and references the relevant ADRs for the
details.

If you have not read [`AGENTS.md`](../AGENTS.md), read it first.

## Mental model

*Delta-V beyond Sector 3.26* is a Bevy game. It is structured as a
**Cargo workspace** of one binary crate and several library crates
(see [ADR-0002](adr/0002-repository-layout-and-workspace.md)).
Each library crate owns one domain and exports exactly one Bevy
plugin (see [ADR-0005](adr/0005-plugin-architecture.md)). The
binary composes the plugins.

```
+---------------------------------------------------------------+
|                       crates/delta-v                          |
|  (binary; composes plugins, owns the App, parses CLI)         |
+----------+---------------+----------------+-------------------+
           |               |                |
           v               v                v
+----------+-----+ +-------+-----+ +--------+--------+
| delta-v-core   | | delta-v-... | | delta-v-net     |
| ECS basics,    | | (config,    | | (placeholder    |
| shared types,  | |  physics,   | |  until M7)      |
| logging init   | |  assets)    | |                 |
+----------------+ +-------------+ +-----------------+
           ^               ^                ^
           |               |                |
+----------+---------------+----------------+-------------------+
|                  domain crates                                |
|  delta-v-ships, -propulsion, -weapons,                        |
|  -stations, -items, -world                                    |
+---------------------------------------------------------------+
```

Domain crates depend on technical crates and may depend on each
other only in one direction; cycles are prevented by Cargo and by
review discipline.

## Crate structure (M0.5 and beyond)

The workspace contains 13 crates:

**Binary:**
- `crates/delta-v/` -- Application composition, plugin registration, CLI parsing.

**Technical (must not depend on domain crates):**
- `crates/delta-v-core/` -- ECS fundamentals, shared components, plugin traits.
- `crates/delta-v-json/` -- Shared JSON utilities: read, validate, fill-defaults pipeline. No Bevy dependency. Used by all JSON-loading crates.
- `crates/delta-v-config/` -- Configuration management: default layer, user-override merge, XDG path resolution, hot-reload (dev builds).
- `crates/delta-v-physics/` -- Newtonian physics, gravity, floating origin (avian3d).
- `crates/delta-v-assets/` -- Asset loaders, glTF helpers.
- `crates/delta-v-net/` -- Networking (stub until M7; ADR-0030/31/32).

**Domain (may depend on technical crates; dependency graph is acyclic):**
- `crates/delta-v-ships/` -- Ship types and ship-specific systems.
- `crates/delta-v-propulsion/` -- Thrusters, hyperdrive.
- `crates/delta-v-weapons/` -- Projectiles, damage.
- `crates/delta-v-stations/` -- Space stations, docking.
- `crates/delta-v-items/` -- Collectables, inventory.
- `crates/delta-v-world/` -- Sectors, boundaries, hyperspace gates.

See [ADR-0002](adr/0002-repository-layout-and-workspace.md) for the rationale.

## Data flow at a glance

1. **Startup**:
   `Boot -> LoadingDefaults -> LoadingWorld -> InGame`
   (see [ADR-0018](adr/0018-state-management.md)).

2. **Each fixed tick** (60 Hz, see
   [ADR-0017](adr/0017-fixed-timestep-and-determinism.md)):
   - Input events are translated to logical actions
     (see [ADR-0011](adr/0011-keybindings-configuration.md)).
   - Gameplay systems update intents on entities.
   - Physics systems (in `delta-v-physics`, on top of `avian3d`)
     integrate forces, torques, gravity
     (see [ADR-0009](adr/0009-newtonian-physics-with-gravity.md)).
   - Floating origin recentres if needed
     (see [ADR-0007](adr/0007-floating-origin.md)).

3. **Each render frame**:
   - Render runs at display rate, decoupled from simulation.
   - Interpolation between simulation states is added later if
     visible stepping becomes a problem.

## Configuration and content

All configuration and content is JSON (validated by JSON Schemas)
or glTF 2.0:

- **Shipped defaults** under `assets/` -- managed by the game
  package, never edited by the player.
- **User overrides** under
  `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/` (settings) and
  `$XDG_DATA_HOME/delta-v-beyond-sector-3-26/` (content).
- See [ADR-0010](adr/0010-configuration-system.md),
  [ADR-0019](adr/0019-asset-pipeline-and-user-content.md),
  [ADR-0012](adr/0012-json-schema-validation.md).
- No silent fallbacks
  ([ADR-0013](adr/0013-no-silent-fallbacks.md)).
- Hot-reload only with `--features dev`
  ([ADR-0035](adr/0035-hot-reload-of-configs.md)).

## Coordinate system and units

Bevy / glTF 2.0 convention, unchanged: right-handed, `+Y` up,
`-Z` forward. Engine base units: m, kg, s, rad. Content JSON uses
explicit unit objects (`{"value": N, "unit": "AU"}`), converted at
load time. See
[ADR-0006](adr/0006-coordinate-system-and-units.md) and
[ADR-0008](adr/0008-physical-units-in-json.md).

## Engine constants vs. gameplay values

Mathematical and physical constants, conversion factors, and
implementation tolerances live in Rust. Gameplay knobs (ship mass,
weapon damage, ...) live in JSON, with no Rust-side defaults. See
[ADR-0014](adr/0014-engine-constants-vs-gameplay-values.md).

## Error handling and logging

- Programmer mistakes panic.
- Library crates return typed errors (`thiserror`).
- The binary collapses to `anyhow::Result` at the top.
- No `unwrap()` / `expect()` outside tests without justification.
- Levels: `error`, `warn`, `info` for releases; `debug`, `trace`
  for development.
- See [ADR-0016](adr/0016-error-handling-strategy.md) and
  [ADR-0015](adr/0015-logging-strategy.md).

## Code style

- `rustfmt`, `clippy::pedantic` enabled, with a small allow-list.
- `unwrap()` / `expect()` / `panic!` / `todo!` / `unimplemented!`
  denied in non-test code.
- Public items documented (`#![warn(missing_docs)]`).
- See [ADR-0023](adr/0023-code-style-and-lints.md) and
  [ADR-0024](adr/0024-documentation-policy.md).

## Multiplayer (planned)

Peer-to-peer with a player-as-host model; NAT traversal candidates
include Steam Datagram Relay (with licence caveats) and STUN/TURN.
Networking library deferred until M7 preparation. See
[ADR-0030](adr/0030-authoritative-model.md),
[ADR-0031](adr/0031-network-library-choice.md),
[ADR-0032](adr/0032-snapshot-and-delta-encoding.md).

## Where the rules live

| Topic                        | Document                                   |
| ---------------------------- | ------------------------------------------ |
| Entry point for any agent    | [`AGENTS.md`](../AGENTS.md)                |
| All architectural decisions  | [`docs/adr/`](adr/)                        |
| Day-to-day workflow          | [`docs/workflow.md`](workflow.md)          |
| Roadmap and milestones       | [`docs/roadmap.md`](roadmap.md)            |
| Physics specifics            | [`docs/physics.md`](physics.md)            |
| High-level design intent     | [`docs/design.md`](design.md)              |
