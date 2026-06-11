# ADR-0051: Crate Architecture Overview

## Status

Accepted

## Context

There is no single document that describes all crates, their purposes, and their
dependencies. Agents must read multiple ADRs and inspect `Cargo.toml` files to
understand the architecture. This leads to:
- Agents placing code in the wrong crate.
- Agents creating duplicate functionality.
- Agents unsure which crate to depend on.

## Decision

This ADR defines the canonical crate architecture. All other ADRs reference this
as the authoritative source for "which crate owns what."

### Crate dependency graph

```
                    ┌─────────────────────┐
                    │     delta-v-json     │  (technical: JSON pipeline)
                    └──────────┬──────────┘
                               │
                    ┌──────────┴──────────┐
                    │    delta-v-types     │  (technical: shared types)
                    └──────────┬──────────┘
                               │
         ┌─────────────────────┼─────────────────────┐
         │                     │                     │
┌────────┴────────┐  ┌────────┴────────┐  ┌────────┴────────┐
│  delta-v-assets  │  │  delta-v-config  │  │   delta-v-spawn  │
│ (technical:      │  │ (domain: config) │  │ (technical:      │
│  asset loading)  │  │                  │  │  spawn utils)    │
└────────┬────────┘  └──────────────────┘  └────────┬────────┘
         │                                          │
         └──────────────────┬───────────────────────┘
                            │
                    ┌───────┴───────┐
                    │  delta-v-core  │  (foundation: state, events, input)
                    └───────┬───────┘
                            │
    ┌───────────┬───────────┼───────────┬───────────┐
    │           │           │           │           │
┌───┴───┐ ┌────┴────┐ ┌───┴───┐ ┌────┴────┐ ┌───┴───┐
│ ships │ │  world  │ │stations│ │  items  │ │weapons│
└───────┘ └─────────┘ └───────┘ └─────────┘ └───────┘
                            │
                    ┌───────┴───────┐
                    │  delta-v-net   │  (domain: networking, stub)
                    └───────────────┘
                            │
                    ┌───────┴───────┐
                    │    delta-v     │  (binary: app assembly)
                    └───────────────┘
```

### Crate descriptions

#### Technical crates (no domain logic, no plugins with systems)

| Crate | Purpose | Depends on | Used by |
|---|---|---|---|
| `delta-v-json` | JSON read/validate/fill-defaults/deserialize pipeline | `serde`, `serde_json`, `jsonschema` | All crates that load JSON |
| `delta-v-types` | Shared domain types (`BoundingBox`, `CollisionShapeJson`, `PhysicalQuantity`, etc.) | `serde`, `bevy` (math types only) | All domain crates |
| `delta-v-spawn` | Spawning utilities (template extraction, collision conversion, mesh attachment) | `delta-v-json`, `delta-v-types`, `bevy` | All domain crates with spawners |
| `delta-v-assets` | Asset path resolution, template loading, template merging | `delta-v-json`, `delta-v-types` | `delta-v-world`, `delta-v-config` |

#### Foundation crate (shared infrastructure, not a domain)

| Crate | Purpose | Depends on | Used by |
|---|---|---|---|
| `delta-v-core` | `AppState`, cross-domain events (`SpawnEntity`), input pipeline, camera, debug, diagnostics, boundary, flight assist, floating origin | `delta-v-types`, `bevy` | All domain crates |

#### Domain crates (each owns one entity type or gameplay system)

| Crate | Purpose | Depends on | Spawns |
|---|---|---|---|
| `delta-v-ships` | Ship entities, input→forces pipeline, propulsion | `delta-v-core`, `delta-v-spawn`, `delta-v-physics` | `player_controlled_ship`, `ship` |
| `delta-v-world` | World/sector loading, asteroid spawning | `delta-v-core`, `delta-v-assets`, `delta-v-spawn` | `asteroid` |
| `delta-v-stations` | Space stations, docking | `delta-v-core`, `delta-v-spawn`, `delta-v-physics` | `station` |
| `delta-v-items` | Collectable items, inventory | `delta-v-core`, `delta-v-spawn` | `item` |
| `delta-v-weapons` | Weapons, projectiles, damage | `delta-v-core`, `delta-v-spawn`, `delta-v-physics` | `projectile` |
| `delta-v-propulsion` | Thrusters, hyperdrive | `delta-v-core`, `delta-v-physics` | (systems only, no spawning) |
| `delta-v-physics` | Newtonian physics, gravity, collision detection | `delta-v-core`, `delta-v-types` | (systems only, no spawning) |
| `delta-v-net` | Networking (stub) | `delta-v-core` | (systems only, no spawning) |

#### Binary crate

| Crate | Purpose | Depends on |
|---|---|---|
| `delta-v` | Application assembly — registers all plugins, contains `main()` | All crates |

### Dependency rules

1. **Technical crates** MUST NOT depend on domain crates.
2. **Domain crates** MAY depend on technical crates and `delta-v-core`.
3. **Domain crates** MUST NOT depend on other domain crates (use events/components
   for cross-domain communication per ADR-0005).
4. **`delta-v-core`** MUST NOT depend on domain crates.
5. **`delta-v`** (binary) depends on all crates and is the only crate that
   registers plugins.

### What goes where — decision tree

```
Is it a plain data type used by 2+ crates?
  └─ YES → delta-v-types

Is it a JSON loading/validation function?
  └─ YES → delta-v-json

Is it an asset path or template loading function?
  └─ YES → delta-v-assets

Is it a spawning utility (template extraction, collision conversion, mesh attachment)?
  └─ YES → delta-v-spawn

Is it a cross-domain event or the AppState machine?
  └─ YES → delta-v-core

Is it a debug/diagnostic utility?
  └─ YES → delta-v-core/src/debug/ or delta-v-core/src/diagnostics/

Is it specific to one entity type (ships, asteroids, stations, etc.)?
  └─ YES → the corresponding domain crate (delta-v-ships, delta-v-world, etc.)

Is it a config file loader?
  └─ YES → delta-v-config

Is it a physics simulation system?
  └─ YES → delta-v-physics
```

## Consequences

- **Positive**: Agents have a clear decision tree for placing new code.
- **Positive**: The crate dependency graph is documented and enforced.
- **Positive**: Cross-domain dependencies are explicitly forbidden.
- **Positive**: Each crate's purpose is a single sentence.

## Related

- ADR-0002 (Repository layout)
- ADR-0005 (Plugin architecture)
- ADR-0046 (Shared types crate)
- ADR-0047 (Centralized spawning)
- ADR-0048 (Delta-v-core restructure)
- ADR-0049 (Template system reorganization)
- ADR-0050 (Naming conventions)
