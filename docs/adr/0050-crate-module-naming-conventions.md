# ADR-0050: Crate and Module Naming Conventions

- **Status**: Accepted
- **Date**: 2026-06-06
- **Deciders**: Cute-Donkey

## Context

Current naming is inconsistent, making it hard for agents to know where to place
new code:

| Crate | File | Purpose |
|---|---|---|
| `delta-v-ships` | `src/spawn.rs` | Ship spawning |
| `delta-v-world` | `src/asteroid_spawner.rs` | Asteroid spawning |
| `delta-v-world` | `src/template_loader.rs` | Template loading |
| `delta-v-world` | `src/loader.rs` | World loading |
| `delta-v-world` | `src/world_def.rs` | World definition types |
| `delta-v-world` | `src/resources.rs` | World resources |
| `delta-v-world` | `src/events.rs` | SpawnEntity re-export |
| `delta-v-core` | `src/ship_templates.rs` | Ship template types |
| `delta-v-core` | `src/spawn_sets.rs` | Spawn ordering sets |
| `delta-v-core` | `src/debug_axes.rs` | Debug axis visualization |
| `delta-v-core` | `src/debug_config.rs` | Debug configuration |

An agent creating a new spawner for stations has no guidance:
- Should it be `src/spawn.rs` or `src/station_spawner.rs`?
- Should it go in `delta-v-stations` or `delta-v-world`?
- Should the system be called `spawn_station` or `spawn_station_system`?

## Decision

### Module naming rules

Every domain crate follows this standard module structure:

```
<domain>/src/
  lib.rs          - Plugin definition, re-exports
  components.rs   - Domain-specific ECS components
  systems.rs      - Domain-specific systems (non-spawn)
  spawn.rs        - Entity spawning from SpawnEntity events
  resources.rs    - Domain-specific resources
  error.rs        - Domain-specific errors
  events.rs       - Domain-specific events (if any)
```

**Rules**:
1. **Spawning logic** ALWAYS goes in `src/spawn.rs`. No exceptions.
   - Not `asteroid_spawner.rs`, not `station_spawner.rs`, not `spawn_stations.rs`.
   - The file is always `spawn.rs`.
2. **System functions** follow the pattern `spawn_<entity_type>` for spawn systems
   (e.g., `spawn_ship`, `spawn_asteroid`, `spawn_station`).
3. **Mesh attachment functions** follow the pattern `attach_<entity_type>_meshes`
   (e.g., `attach_ship_meshes`, `attach_asteroid_meshes`).
4. **Pending mesh markers** follow the pattern `Pending<EntityType>Mesh`
   (e.g., `PendingShipMesh`, `PendingAsteroidMesh`).
5. **Components** are named after the concept they represent, without a suffix
   (e.g., `FlightAssist`, not `FlightAssistComponent`).
6. **Resources** are named after the concept they represent, without a suffix
   (e.g., `WorldDefResource`, not `WorldDefRes`).
7. **Events** are named after the action they represent, in past tense or
   imperative (e.g., `SpawnEntity`, `CollisionDetected`).

### Crate naming rules

1. All crates are `delta-v-<domain>` where `<domain>` is a lowercase kebab-case
   noun describing the domain (ADR-0002).
2. Technical (non-domain) crates use descriptive names:
   - `delta-v-json` — JSON pipeline
   - `delta-v-types` — Shared types
   - `delta-v-assets` — Asset loading
   - `delta-v-spawn` — Spawning utilities
3. Domain crates use the entity type they primarily manage:
   - `delta-v-ships` — Ships
   - `delta-v-world` — World/sector management
   - `delta-v-stations` — Space stations
   - `delta-v-items` — Collectable items
   - `delta-v-weapons` — Weapons and projectiles
   - `delta-v-propulsion` — Thrusters and hyperdrive

### Where to place new functionality

| If you are adding... | Place it in... |
|---|---|
| A new shared type (used by 2+ crates) | `delta-v-types` |
| A new JSON extraction helper | `delta-v-spawn::template_extraction` |
| A new mesh attachment pattern | `delta-v-spawn::mesh_attachment` |
| A new collision shape conversion | `delta-v-spawn::collision` |
| A new entity type's spawner | `<domain>/src/spawn.rs` |
| A new domain-specific component | `<domain>/src/components.rs` |
| A new domain-specific system | `<domain>/src/systems.rs` |
| A new cross-domain event | `delta-v-core/src/events/` |
| A new config file loader | `delta-v-config/src/loader.rs` |
| A new template type | `delta-v-assets/src/template.rs` |
| Debug visualization for a domain | `delta-v-core/src/debug/` |

### File naming for tests

Per ADR-0021, test files use the `_tests.rs` suffix and live alongside the module
they test:
- `src/spawn.rs` → `src/spawn_tests.rs`
- `src/systems/my_system.rs` → `src/systems/my_system_tests.rs`

## Consequences

- **Positive**: Agents can determine the correct file and crate for new code
  without guessing.
- **Positive**: Consistent naming makes the codebase navigable.
- **Positive**: The `spawn.rs` convention means every domain crate has the same
  entry point for spawning logic.
- **Migration effort**: Low — mostly renaming files, not changing logic.

## Notes

- ADR-0002 (Repository layout)
- ADR-0021 (Test file naming)
- ADR-0023 (Code style and lints)
- ADR-0047 (Centralized spawning)
- ADR-0048 (Delta-v-core restructure)
