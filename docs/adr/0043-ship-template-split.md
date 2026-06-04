# ADR-0043: Split ship template into base ship and player-controlled ship

- **Status**: Accepted
- **Date**: 2026-06-04
- **Deciders**: OWL (AI agent), human reviewer

## Context

The original `local_player_ship.schema.json` contained 90% of data that every ship would have (mass, inertia_scale, mesh, propulsion) and only 10% that was player-specific (cameras). All ship templates used `entity_type: "local_player_ship"` even though debug ships don't need cameras.

This caused two problems:
1. **No separation of concerns**: Player-specific data (cameras) was mixed with common ship data
2. **No easy ship switching**: To fly a different ship, you had to create a whole new template file with all properties duplicated

Per ADR-0038 (entity template system), each entity type should have its own schema. Per ADR-0012 (JSON schema validation), schemas should avoid `allOf`/`oneOf` composition patterns.

## Decision

We split the ship template into two levels:

1. **Base ship schema** (`ship.schema.json`): Contains common properties (mass, inertia_scale, mesh, propulsion). Used for all ship types.

2. **Player-controlled ship schema** (`player_controlled_ship.schema.json`): Contains only player-specific properties (cameras) plus a `ship_template` reference to a base ship template. At load time, the referenced ship template is loaded and merged.

### Schema Structure

```
ship.schema.json (base ship)
├── entity_type: "ship" (const)
├── mass, inertia_scale, mesh, propulsion
└── $defs: physical_quantity_mass, physical_quantity_force, etc.

player_controlled_ship.schema.json (player ship)
├── entity_type: "player_controlled_ship" (const)
├── ship_template: path to base ship template
├── cameras: { cockpit, chase }
└── $defs: vec3
```

### Template Structure

```
templates/ships/space-fighter-comrade1280/template.json (entity_type: "ship")
├── mass, inertia_scale, mesh, propulsion
└── NO cameras

templates/ships/player_ship/template.json (entity_type: "player_controlled_ship")
├── ship_template: "templates/ships/space-fighter-comrade1280/template.json"
└── cameras: { cockpit: { x, y, z } }
```

### Switching Ships

To fly a different ship, change ONE line in `player_ship.json`:
```json
{
  "entity_type": "player_controlled_ship",
  "ship_template": "templates/ships/cruiser.json",
  "cameras": { "cockpit": { "x": 0, "y": 0.5, "z": -0.2 } }
}
```

### World Definition

The `entity_type` field was removed from `world.json`. The entity type is now derived from the template's `entity_type` field at load time.

### Rust Structs

- `ShipTemplate`: Base struct with mass, inertia_scale, propulsion
- `PlayerShipTemplate`: Extends base with cameras (for deserializing merged template)

## Consequences

### Positive
- **Single source of truth**: Ship properties defined once in `ship.schema.json`
- **Easy ship switching**: Change one line in `player_ship.json` to fly a different ship
- **Clear separation**: Player-specific data (cameras) separated from common ship data
- **ADR-0012 compliant**: No `allOf`/`oneOf` composition in schemas
- **Extensible**: Easy to add NPC ships with AI config in the future

### Negative
- **Template merging complexity**: The template loader now needs to load and merge two templates
- **Runtime overhead**: Slightly more work at load time to merge templates
- **Two-level template pattern**: More complex than a single template file

### Follow-up Work
- NPC ship schema (`npc_ship.schema.json`) with AI behavior config
- Consider adding template inheritance for other entity types (stations, asteroids)

## Notes

- The `delta-v-json` crate only resolves local `$ref` pointers (`#/$defs/<name>`), so we couldn't use `allOf` with external `$ref`s
- The `player_controlled_ship` template uses a flat schema (no `allOf`) to comply with ADR-0012 Rule 8
- Template merging happens at load time, not at spawn time, so the spawn system receives a fully merged template
