# ADR-0038: Entity template system

- **Status**: Accepted
- **Date**: 2026-05-28
- **Deciders**: Cute-Donkey

## Context

The game world contains many entity types: suns, planets, moons, asteroids,
stations, ships, and more. Each type has different properties (mass, radius,
textures, collision geometry) and spawning requirements.

A naive approach would hard-code every entity type in Rust or create a
monolithic world-definition format. Both approaches create maintenance
burdens:

1. Hard-coding couples content creation to code releases.
2. A monolithic format forces every entity type to know about every other
   type's fields, violating separation of concerns.
3. Complex entity dependencies (e.g., stations orbit planets, which orbit
   suns) are hard to express and validate.

We need a way to:

- Describe entity types once, independently.
- Reference entity types by name in world definitions.
- Validate each entity type against its own schema.
- Spawn entities in the correct order (respecting dependencies like
  "planets after suns").
- Extend the system without modifying the world loader.

See ADR-0005 (plugin architecture), ADR-0012 (JSON schema validation),
ADR-0013 (no silent fallbacks), ADR-0019 (asset pipeline), and
ADR-0020 (save and load format).

## Decision

We implement an **entity template system** with the following design:

### 1. Entity Templates

An entity template is a JSON file that describes a **single entity type**.
Each template:

- Lives under `assets/templates/<entity_category>/`.
- Contains a required `entity_type` field (string discriminator).
- Has its own JSON Schema: `assets/json/schema/<entity_type>.schema.json`.
- Includes all static properties for that entity type (mass, radius, color,
  collision geometry, etc.).

Example:

```json
// assets/templates/suns/red_giant.json
{
  "entity_type": "sun",
  "name": "Betelgeuse",
  "mass": { "value": 16.5, "unit": "M_sun" },
  "radius": { "value": 700, "unit": "R_sun" },
  "color": { "r": 0.9, "g": 0.3, "b": 0.2 }
}
```

### 2. World Definition Referencing Templates

A world definition lists entities to spawn by reference, not by inlining
their full definition. Each entry specifies:

- `template`: path to the template file.
- `entity_type`: the entity type (redundant with the template, but
  explicit for validation).
- `position`: spawn location in world coordinates (metres).
- Optional: `rotation`, `scale`, or other per-instance overrides.

Example:

```json
// assets/worlds/default.world.json
{
  "format_version": 1,
  "name": "Sector 3.26",
  "entities": [
    {
      "template": "templates/suns/red_giant.json",
      "entity_type": "sun",
      "position": { "x": 0, "y": 0, "z": 0 }
    },
    {
      "template": "templates/planets/earth.json",
      "entity_type": "planet",
      "position": { "x": 100, "y": 0, "z": 0 }
    },
    {
      "template": "templates/ships/local_player_ship.json",
      "entity_type": "local_player_ship",
      "position": { "x": 120, "y": 10, "z": 0 }
    }
  ]
}
```

### 3. Loading and Spawning

The process follows these steps (ADR-0013 three-step load process):

1. **Load world JSON**: Parse `world.json`, validate against
   `world.schema.json`.
2. **Validate and fill defaults**: Apply schema defaults for the world
   definition.
3. **For each entity entry**:
   a. Load the template file (e.g., `red_giant.json`).
   b. Validate the template against its schema
      (`<entity_type>.schema.json`).
   c. Apply schema defaults to the template.
   d. Emit a `SpawnEntity` event with the loaded template + instance data
      (position, etc.).
4. **Spawn by type**: Domain plugins (SunPlugin, PlanetPlugin,
   ShipsPlugin) listen for `SpawnEntity` events, filter by `entity_type`,
   and spawn the entity.

### 4. Spawn Ordering via System Sets

Entity dependencies are managed via Bevy System Sets, not by the loader.
Example:

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorldSpawnSet {
    SpawnSuns,
    SpawnPlanets,
    SpawnMoons,
    SpawnStations,
    SpawnShips,
}

// In each plugin's build():
app.add_systems(
    OnEnter(AppState::SpawningEntities),
    sun_spawn_system.in_set(WorldSpawnSet::SpawnSuns),
);
app.add_systems(
    OnEnter(AppState::SpawningEntities),
    planet_spawn_system
        .in_set(WorldSpawnSet::SpawnPlanets)
        .after(WorldSpawnSet::SpawnSuns),
);
```

This ensures planets are spawned after suns, moons after planets, etc.,
without the loader needing to know about dependencies.

### 5. Schema Organization

Each entity type has its own schema:

```
assets/json/schema/
  sun.schema.json
  planet.schema.json
  moon.schema.json
  station.schema.json
  local_player_ship.schema.json
  asteroid.schema.json
  ...
```

The world schema (`world.schema.json`) defines only the envelope
(format_version, name, entities array shape), not the individual entity
contents.

## Consequences

Positive:

- **Content and code decoupled**: New entity types can be added by adding
  templates and schemas without touching Rust code.
- **Clear separation of concerns**: Each entity type owns its schema and
  spawning logic.
- **Type safety via schema**: Every entity is validated against its schema
  before spawning (ADR-0012, ADR-0013).
- **Extensible**: Adding a new entity type is a matter of:
  1. Creating a new schema.
  2. Adding a new plugin with a spawn system.
  3. Creating template files.
- **Dependency management**: System sets make spawn ordering explicit and
  maintainable.
- **Reusable templates**: The same `red_giant.json` can be used in multiple
  worlds.

Negative:

- **More files**: Each entity type requires a template file + schema file
  + plugin code. This is intentional; the trade-off is accepted.
- **Indirection**: World definitions reference templates by path, which
  adds one level of indirection when reading. This is offset by clarity:
  you know immediately that the world uses "earth.json", not an inline
  definition.
- **Validation complexity**: Loaders must handle template loading and
  validation. This is a one-time cost (implemented in `delta-v-json` and
  `delta-v-world`).

Follow-up work:

- Implement `SpawnEntity` event in `delta-v-world`.
- Define `WorldSpawnSet` system set.
- Update `WorldPlugin` to emit `SpawnEntity` events instead of spawning
  directly.
- Update `ShipsPlugin` to listen for `SpawnEntity` events with
  `entity_type: "local_player_ship"`.
- Create template files for M1 entities (player ship, asteroids if any).
- Create schemas for each entity type.
- Extend `world.schema.json` to define the entities array shape.

## Notes

- This design is inspired by game engines like Godot and Bevy's own
  scene system, which separate data (scenes/templates) from code (plugins).
- The system is incremental: M1 will only use `local_player_ship` and
  perhaps `asteroid` templates. M2+ will extend it with suns, planets, etc.
- Schemas must NOT use `oneOf`/`anyOf`/`if-then-else` combinators
  (ADR-0012, rule 8); each `entity_type` gets its own schema file instead.
