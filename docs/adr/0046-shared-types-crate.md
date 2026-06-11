# ADR-0046: Shared Types Crate

## Status

Accepted

## Context

Agents repeatedly create duplicate types for bounding boxes, collision shapes, and
physical quantities across different crates. The current situation:

- `delta-v-core/src/ship_templates.rs` defines `BoundingBox`, `ShipCollisionShape`,
  `PhysicalQuantity`, `Vec3Json`, `ShipPropulsionTemplate`, `ThrustCommand`,
  `TorqueCommand`, and `ShipPropulsionConfig`.
- `delta-v-physics/src/collision.rs` defines `CollisionShape`, `CollisionShapeType`,
  `CollisionLayers`, `CollisionDetected`, `StaticBody`, `DynamicBody`.
- `delta-v-world/src/world_def.rs` defines `Vec3Json` and `QuatJson` (duplicated from core).
- `delta-v-ships/src/spawn.rs` contains `create_collision_shape()` which converts
  `ShipCollisionShape` (JSON deserialization struct) into `CollisionShape` (physics component).
- `delta-v-world/src/asteroid_spawner.rs` contains `get_collision_shape_from_template()`,
  `get_mass_from_template()`, and `get_debug_axis_length_from_template()` which
  duplicate the logic of extracting data from template JSON.

The core problems:
1. **Duplicated types**: `Vec3Json` is defined in both `delta-v-core` and `delta-v-world`.
   `ShipCollisionShape` is ship-specific in name only — asteroids, planets, and stations
   use the same JSON structure.
2. **Two representations, no naming convention**: Every concept that crosses the JSON/runtime
   boundary has two types (e.g., `Vec3Json` → `Vec3`, `CollisionShapeJson` → `CollisionShape`),
   but there is no consistent naming convention to distinguish them.
3. **No central home**: Shared types are scattered across domain crates.

## Decision

Create a new crate `delta-v-types` that owns all shared domain types. This is a
**technical** crate (not a domain crate) that provides the common vocabulary for
all other crates.

### Crate: `delta-v-types`

**Purpose**: Shared domain types used across multiple crates. No systems, no plugins,
no Bevy `Component` derive — only plain types and serde deserialization structs.

**Dependencies**: `serde`, `serde_json`, `bevy` (math types only: `Vec3`, `Quat`).

**Module structure**:
```
delta-v-types/src/
  lib.rs              - crate root, re-exports
  spatial.rs          - BoundingBox, Vec3Json, QuatJson
  physics.rs          - PhysicalQuantity
  collision.rs        - CollisionShapeJson
  ids.rs              - EntityId, TemplatePath (newtype wrappers for type safety)
```

### JSON value naming convention

Types that exist solely to deserialize a JSON value into a simpler Rust type
MUST use the `Json` suffix. This makes the JSON/runtime distinction explicit
at the type level.

| JSON type (serde) | Runtime type | Crate | JSON example |
|---|---|---|---|
| `Vec3Json` | `Vec3` (bevy) | `delta-v-types` | `{"x": 1.0, "y": 2.0, "z": 3.0}` |
| `QuatJson` | `Quat` (bevy) | `delta-v-types` | `{"x": 0, "y": 0, "z": 0, "w": 1}` |
| `CollisionShapeJson` | `CollisionShape` (Component) | `delta-v-types` / `delta-v-physics` | `{"type": "sphere", "radius": {"value": 1, "unit": "m"}}` |

**Rules**:
1. JSON deserialization value types MUST use the `Json` suffix (e.g., `Vec3Json`, `CollisionShapeJson`).
2. The corresponding runtime type MUST NOT use the `Json` suffix (e.g., `Vec3`, `CollisionShape`).
3. The `Json` type lives in `delta-v-types`; the runtime type lives in the domain crate that owns it.
4. Conversion from `Json` type to runtime type lives in `delta-v-spawn`.
5. This convention applies ONLY to types whose sole purpose is JSON deserialization of a simple value. Domain types that happen to also be deserialized (e.g., `PhysicalQuantity`, `BoundingBox`, `CameraDefinition`) do NOT use the `Json` suffix.

### Two collision shape types, two crates

| Type | Crate | Purpose | Dependencies |
|---|---|---|---|
| `CollisionShapeJson` | `delta-v-types/src/collision.rs` | Deserialize collision shape from JSON. Has `String` shape_type, `Option<PhysicalQuantity>` radius, `Option<Vec3Json>` half_extents. | `serde`, `bevy` (Vec3) |
| `CollisionShape` | `delta-v-physics/src/collision.rs` | Bevy `Component` for physics simulation. Has `CollisionShapeType` enum, `Vec3` offset, `f32` radius. | `bevy` (Component) |

### Rules for delta-v-types

1. `delta-v-types` MUST NOT depend on any other `delta-v-*` crate.
2. `delta-v-types` MUST NOT contain systems, plugins, or Bevy resources — only plain types and serde structs.
3. `delta-v-types` MAY depend on `serde` and `bevy` (math types only).
4. `delta-v-types` MUST NOT derive `bevy::ecs::component::Component` — that's for domain crates.
5. All domain crates that need shared types MUST import them from `delta-v-types`.
6. There is ONE `CollisionShapeJson` for ALL entity types — NOT per-entity-type variants.

### Migration of existing types

| Current Location | Type | New Location |
|---|---|---|
| `delta-v-core/src/ship_templates.rs` | `BoundingBox` | `delta-v-types/src/spatial.rs` |
| `delta-v-core/src/ship_templates.rs` | `PhysicalQuantity` | `delta-v-types/src/physics.rs` |
| `delta-v-core/src/camera.rs` | `Vec3Json` | `delta-v-types/src/spatial.rs` |
| `delta-v-world/src/world_def.rs` | `Vec3Json` | `delta-v-types/src/spatial.rs` (deduplicate) |
| `delta-v-world/src/world_def.rs` | `QuatJson` | `delta-v-types/src/spatial.rs` |
| `delta-v-core/src/ship_templates.rs` | `ShipCollisionShape` | `delta-v-types/src/collision.rs` (renamed to `CollisionShapeJson`) |
| `delta-v-physics/src/collision.rs` | `CollisionLayers` | `delta-v-types/src/collision.rs` |
| `delta-v-physics/src/collision.rs` | `StaticBody` | `delta-v-types/src/collision.rs` |
| `delta-v-physics/src/collision.rs` | `DynamicBody` | `delta-v-types/src/collision.rs` |

### Types that stay where they are

| Type | Current Location | Reason |
|---|---|---|
| `CollisionShape` | `delta-v-physics/src/collision.rs` | Bevy `Component` with `CollisionShapeType` enum — physics runtime |
| `CollisionShapeType` | `delta-v-physics/src/collision.rs` | Physics runtime enum — stays with `CollisionShape` |
| `CollisionDetected` | `delta-v-physics/src/collision.rs` | Physics-specific event |
| `RigidBody` | `delta-v-physics/src/rigid_body.rs` | Physics-specific component |
| `ShipPropulsionTemplate` | `delta-v-core/src/ship_templates.rs` | Ship-specific template type |
| `ThrustCommand` | `delta-v-core/src/ship_templates.rs` | Per-tick command buffer (resource) |
| `TorqueCommand` | `delta-v-core/src/ship_templates.rs` | Per-tick command buffer (resource) |
| `ShipPropulsionConfig` | `delta-v-core/src/ship_templates.rs` | Ship-specific runtime config resource |
| `DebugAxesEligible` | `delta-v-core/src/debug_axes.rs` | Debug-specific marker |

### Conversion: JSON → runtime

Each `Json` type has a corresponding conversion function in `delta-v-spawn`:

```rust
// delta-v-spawn/src/collision.rs
/// Converts a CollisionShapeJson into a physics CollisionShape.
/// This is the SINGLE function that handles sphere/box conversion for ALL entity types.
pub fn shape_from_json(json: &CollisionShapeJson, scale: f32) -> CollisionShape { ... }
```

## Consequences

- **Positive**: Consistent `Json` suffix naming convention makes it immediately obvious which types are JSON deserialization wrappers vs runtime types.
- **Positive**: `Vec3Json` and `QuatJson` deduplicated into `delta-v-types`.
- **Positive**: One `CollisionShapeJson` for all entity types — no per-type variants.
- **Positive**: Clear separation: `delta-v-types` = serde JSON types, domain crates = runtime components, `delta-v-spawn` = conversion.
- **Negative**: One more crate in the workspace. Mitigated by it being small and stable.
- **Migration effort**: Moderate. Rename `ShipCollisionShape` → `CollisionShapeJson`, deduplicate `Vec3Json`/`QuatJson`, update all imports.

## Related

- ADR-0002 (Repository layout and workspace)
- ADR-0038 (Entity template system)
- ADR-0040 (delta-v-json for JSON validation)
- ADR-0047 (Centralized spawning infrastructure)
- ADR-0050 (Crate and module naming conventions)
