# ADR-0047: Centralized Spawning Infrastructure

- **Status**: Accepted
- **Date**: 2026-06-06
- **Deciders**: Cute-Donkey

## Context

Entity spawning is scattered across multiple crates with no shared infrastructure:

- `delta-v-ships/src/spawn.rs` — spawns ships from `SpawnEntity` events. Contains
  `spawn_ship_from_template()`, `spawn_player_ship()`, `spawn_static_ship()`,
  `create_collision_shape()`, `attach_ship_meshes()`, `setup_scene_lighting()`.
- `delta-v-world/src/asteroid_spawner.rs` — spawns asteroids from `SpawnEntity` events.
  Contains `spawn_asteroid_system()`, `get_mass_from_template()`,
  `get_collision_shape_from_template()`, `get_debug_axis_length_from_template()`,
  `attach_asteroid_meshes()`.
- `delta-v-world/src/lib.rs` — contains `build_spawn_event()`, `load_player_controlled_ship()`,
  `load_asteroid()`, `load_ship()` which are template loading and merging functions.

The problems:
1. **Duplicated JSON extraction**: `get_collision_shape_from_template()` in
   `asteroid_spawner.rs` duplicates the collision shape creation logic in
   `create_collision_shape()` in `spawn.rs`. Both parse `radius`, `half_extents`,
   `offset` from JSON manually.
2. **Duplicated mesh attachment**: `attach_ship_meshes()` and `attach_asteroid_meshes()`
   are nearly identical — both query for a pending marker, check if glTF loaded,
   and attach the scene.
3. **Inconsistent naming**: `spawn.rs` (in `delta-v-ships`) vs.
   `asteroid_spawner.rs` (in `delta-v-world`). An agent creating a new spawner
   has no guidance on where to put it or what to name it.
4. **No reusable spawner utilities**: Every new spawner must reimplement the
   template-to-entity pipeline from scratch.

## Decision

### 1. Create `delta-v-spawn` crate

A new **technical** crate that owns the shared spawning infrastructure. This crate
provides the reusable building blocks that domain-specific spawners compose.

**Purpose**: Centralized spawning utilities, JSON-to-entity conversion
helpers, and the mesh attachment pipeline.

**Dependencies**: `delta-v-types`, `delta-v-json`, `bevy`.

**Module structure**:
```
delta-v-spawn/src/
  lib.rs                - crate root, plugin, re-exports
  template_extraction.rs - JSON template field extraction helpers
  mesh_attachment.rs    - Generic glTF mesh attachment system
  collision.rs          - CollisionShapeJson to physics CollisionShape conversion
  lighting.rs           - Scene lighting setup
```

### 2. Naming conventions for spawners

All domain crates that spawn entities follow this pattern:

| Crate | Spawner File | System Function | Naming Pattern |
|---|---|---|---|
| `delta-v-ships` | `src/spawn.rs` | `spawn_ship` | `spawn_<entity_type>` |
| `delta-v-world` | `src/spawn.rs` | `spawn_asteroid` | `spawn_<entity_type>` |
| `delta-v-stations` | `src/spawn.rs` | `spawn_station` | `spawn_<entity_type>` |
| `delta-v-items` | `src/spawn.rs` | `spawn_item` | `spawn_<entity_type>` |

**Rules**:
1. Each domain crate that spawns entities MUST have a `src/spawn.rs` module.
2. The spawn system function MUST be named `spawn_<entity_type>` (singular).
3. The mesh attachment function MUST be named `attach_<entity_type>_meshes`.
4. Domain spawners MUST use the shared utilities from `delta-v-spawn` for:
   - Template field extraction (`template_extraction`)
   - Collision shape conversion (`collision`)
   - Mesh attachment (`mesh_attachment`)
   - Scene lighting (`lighting`)
5. Domain spawners MUST NOT manually parse JSON template fields — they MUST
   use the deserialization structs from `delta-v-types` and the extraction
   helpers from `delta-v-spawn`.

### 3. Shared utilities in `delta-v-spawn`

#### `template_extraction` module
```rust
/// Extracts a PhysicalQuantity's value from a validated template JSON value.
pub fn extract_mass(template: &Value) -> f32 { ... }

/// Extracts a Vec3 from a JSON object with x, y, z fields.
pub fn extract_vec3(json: &serde_json::Map<String, Value>) -> Vec3 { ... }

/// Extracts a BoundingBox from a validated template JSON value.
pub fn extract_bounding_box(template: &Value) -> BoundingBox { ... }

/// Computes debug axis length from a BoundingBox (120% of longest side).
pub fn compute_debug_axis_length(bbox: &BoundingBox) -> f32 { ... }
```

#### `collision` module
```rust
/// Converts a CollisionShapeJson (from delta-v-types, deserialized from JSON)
/// into a physics CollisionShape (Bevy Component for the physics simulation).
/// This is the SINGLE function that handles sphere/box conversion for ALL entity types.
/// See ADR-0046 for the Json suffix naming convention.
pub fn shape_from_json(
    json: &CollisionShapeJson,
    scale: f32,
) -> CollisionShape { ... }
```

#### `mesh_attachment` module
```rust
/// Generic system that attaches loaded glTF scenes to entities with a pending marker.
/// The marker component type is generic — each domain defines its own.
pub fn attach_meshes<T: Component + PendingMesh>(
    mut commands: Commands,
    gltf_assets: Res<Assets<Gltf>>,
    query: Query<(Entity, &T)>,
) { ... }

/// Trait for pending mesh marker components.
pub trait PendingMesh {
    fn gltf_handle(&self) -> &Handle<Gltf>;
}
```

#### `lighting` module
```rust
/// Sets up scene lighting (directional + ambient). Called once per world load.
pub fn setup_scene_lighting(commands: &mut Commands) { ... }
```

### 4. Collision shape architecture (two types, two crates)

The collision shape exists at two layers (see ADR-0046 for the `Json` suffix naming convention):

| Type | Crate | Purpose |
|---|---|---|
| `CollisionShapeJson` | `delta-v-types` | Serde deserialization from JSON. Has `String` shape_type, `Option<PhysicalQuantity>` radius, `Option<Vec3Json>` half_extents. |
| `CollisionShape` | `delta-v-physics` | Bevy `Component` for physics simulation. Has `CollisionShapeType` enum, `Vec3` offset, `f32` radius. |

The conversion function `shape_from_json()` in `delta-v-spawn` bridges them:
```
JSON → CollisionShapeJson (serde) → shape_from_json() → CollisionShape (Component)
```

This means:
- **One** `CollisionShapeJson` for ALL entity types (ships, asteroids, planets, stations)
- **One** `CollisionShape` component for the physics simulation
- **One** conversion function in `delta-v-spawn`
- **Zero** per-entity-type collision shape variants

### 5. Migration plan for existing code

1. `delta-v-ships/src/spawn.rs`:
   - Replace `create_collision_shape()` with `delta-v-spawn::collision::shape_from_json()`.
   - Replace `attach_ship_meshes()` with `delta-v-spawn::mesh_attachment::attach_meshes::<PendingShipMesh>()`.
   - Keep `PendingShipMesh` marker and `spawn_ship_from_template` (domain-specific logic).

2. `delta-v-world/src/asteroid_spawner.rs`:
   - Delete `get_mass_from_template()`, `get_collision_shape_from_template()`,
     `get_debug_axis_length_from_template()`.
   - Use `delta-v-spawn::template_extraction` instead.
   - Use `delta-v-spawn::collision::shape_from_json()` instead.
   - Replace `attach_asteroid_meshes()` with `delta-v-spawn::mesh_attachment::attach_meshes::<PendingAsteroidMesh>()`.
   - Rename file to `src/spawn.rs` (convention).

3. `delta-v-world/src/lib.rs`:
   - Move `load_player_controlled_ship()`, `load_asteroid()`, `load_ship()` to
     `delta-v-assets` (see ADR-0049).

## Consequences

- **Positive**: New spawners (stations, items, weapons) follow a clear pattern.
- **Positive**: JSON extraction logic is centralized — no more duplication.
- **Positive**: Mesh attachment is generic — no more copy-pasted glTF loading.
- **Positive**: One `CollisionShapeJson` for all entity types — no per-type variants.
- **Positive**: Naming follows the `Json` suffix convention from ADR-0046 (consistent with `Vec3Json`, `QuatJson`).
- **Positive**: Clear separation: `delta-v-types` = serde JSON types, `delta-v-physics` = runtime components, `delta-v-spawn` = conversion.
- **Positive**: Naming convention (`spawn.rs` + `spawn_<entity_type>`) is enforced
  by ADR, discoverable by agents.
- **Negative**: Moderate migration effort for existing spawners.
- **Negative**: One more crate. Mitigated by it being a thin utility layer.

## Notes

- ADR-0005 (Plugin architecture)
- ADR-0038 (Entity template system)
- ADR-0046 (Shared types crate)
- ADR-0049 (Template system reorganization)
