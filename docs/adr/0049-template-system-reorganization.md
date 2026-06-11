# ADR-0049: Template System Reorganization

## Status

Accepted

## Context

The template system is scattered across multiple crates with unclear responsibilities:

- `delta-v-json` — Provides `load_validated()`, `load_validated_with_units()`,
  `fill_defaults()`, `validate()`. This is the JSON pipeline: read → validate →
  fill defaults → deserialize.
- `delta-v-assets` — Empty crate. Only logs "AssetsPlugin initialized". Was
  intended for "asset loader extensions and glTF helpers" (per its doc comment).
- `delta-v-world/src/template_loader.rs` — Loads entity templates using
  `delta-v-json`, resolves template paths, validates `entity_type` field.
- `delta-v-world/src/lib.rs` — Contains `build_spawn_event()`,
  `load_player_controlled_ship()`, `load_asteroid()`, `load_ship()` which
  merge templates and emit `SpawnEntity` events.
- `delta-v-world/src/loader.rs` — Loads world definitions using `delta-v-json`.
- `delta-v-config/src/loader.rs` — Loads config files using `delta-v-json`.

The problems:
1. **`delta-v-assets` is empty** — it should own the template loading pipeline
   and asset path resolution, but it does nothing.
2. **Template loading is in `delta-v-world`** — but templates are not just for
   worlds. The template loading logic should be centralized.
3. **Unclear separation between `delta-v-json` and `delta-v-assets`** —
   `delta-v-json` handles JSON validation. `delta-v-assets` should handle
   asset path resolution and template loading. But the boundary is blurry.
4. **Template merging is in `delta-v-world/src/lib.rs`** — the
   `load_player_controlled_ship()` function merges `player_controlled_ship.json`
   with `ship.json`. This is template processing logic, not world loading logic.

## Decision

### Clarify crate responsibilities

```
delta-v-json     → JSON pipeline only (read, validate, fill defaults, deserialize)
delta-v-assets   → Asset path resolution, template loading, template merging
delta-v-world    → World definition loading (uses delta-v-assets for templates)
delta-v-config   → Config loading (uses delta-v-json directly for simple files)
```

### `delta-v-json` — JSON pipeline (unchanged)

**Responsibilities**:
- Read JSON from file
- Validate against JSON Schema (Draft 2020-12+)
- Fill schema defaults
- Re-validate after default-fill
- Deserialize into Rust structs
- Unit validation (physical quantities)

**Does NOT**:
- Resolve asset paths
- Load templates
- Merge templates
- Know about the `assets/` directory structure

### `delta-v-assets` — Asset and template loading

**Responsibilities**:
- Resolve asset paths (templates, configs, meshes) relative to the `assets/` root
- Load entity templates (single file or merged for `player_controlled_ship`)
- Provide the `resolve_template_path()` function (moved from `delta-v-world`)
- Provide the `load_template()` function (moved from `delta-v-world`)
- Provide the `load_player_controlled_ship()` merge function (moved from `delta-v-world`)
- Provide the `load_asteroid()` and `load_ship()` functions (moved from `delta-v-world`)
- Own the `AssetRoot` resource that knows the path to `assets/`

**Module structure**:
```
delta-v-assets/src/
  lib.rs              - Plugin, re-exports
  paths.rs            - Asset path resolution (template paths, mesh paths)
  template.rs         - Template loading and merging
  error.rs            - Asset-specific errors
```

**Dependency**: `delta-v-assets` depends on `delta-v-json` for the JSON pipeline.

### `delta-v-world` — World definition loading (simplified)

After the migration, `delta-v-world` only:
- Loads the world definition file (`*.world.json`) using `delta-v-assets::template`
- Emits `SpawnEntity` events for each entity in the world
- Contains asteroid spawning logic (in `src/spawn.rs`)

The `template_loader.rs` module is deleted — its functions move to `delta-v-assets`.

### `delta-v-config` — Config loading (unchanged)

`delta-v-config` continues to use `delta-v-json` directly for loading config files.
Config files are simple key-value mappings, not entity templates, so they don't need
`delta-v-assets`.

### Template loading flow

```
World Definition (*.world.json)
  │
  ├─ delta-v-world::loader
  │    Uses: delta-v-assets::template::load_world()
  │    Which uses: delta-v-json::load_validated()
  │
  └─ For each entity in world:
       │
       ├─ delta-v-assets::template::load_template()
       │    Resolves path: "ships/debug-ship-cube" → "templates/ships/debug-ship-cube/ship.json"
       │    Uses: delta-v-json::load_validated_with_units()
       │
       └─ delta-v-assets::template::load_player_controlled_ship()
            Loads player_controlled_ship.json
            Loads ship.json
            Merges them
            Returns merged Value
```

### Naming conventions

1. Template loading functions are named `load_<template_type>()` (e.g., `load_ship()`,
   `load_asteroid()`, `load_player_controlled_ship()`).
2. Path resolution functions are named `resolve_<asset_type>_path()` (e.g.,
   `resolve_template_path()`, `resolve_mesh_path()`).
3. All template loading goes through `delta-v-assets`.
4. All JSON validation goes through `delta-v-json`.

## Consequences

- **Positive**: `delta-v-assets` is no longer empty — it owns a clear responsibility.
- **Positive**: Template loading is centralized — agents know to use `delta-v-assets`.
- **Positive**: Clear separation: `delta-v-json` = JSON pipeline, `delta-v-assets` =
  asset/template loading, `delta-v-world` = world loading.
- **Positive**: Template merging logic is in `delta-v-assets`, not buried in
  `delta-v-world/src/lib.rs`.
- **Migration effort**: Moderate — moving functions between crates, updating imports.

## Related

- ADR-0019 (Asset pipeline)
- ADR-0038 (Entity template system)
- ADR-0040 (delta-v-json for JSON validation)
- ADR-0042 (Custom asset loader / two roots)
- ADR-0043 (Ship template split)
- ADR-0047 (Centralized spawning)
