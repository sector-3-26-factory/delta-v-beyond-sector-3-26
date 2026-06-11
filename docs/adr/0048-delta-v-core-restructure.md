# ADR-0048: Delta-V Core Crate Restructuring

## Status

Accepted

## Context

`delta-v-core` has 16 modules flat in `src/` with no subdirectory structure. The
crate mixes fundamentally different concerns:

```
delta-v-core/src/
  lib.rs              - Plugin definition, state transitions
  state.rs            - AppState enum
  state_tests.rs      - Tests
  events.rs           - SpawnEntity event
  spawn_sets.rs       - WorldSpawnSet system sets
  input.rs            - Input pipeline (ActiveActions, LogicalAction)
  input_tests.rs      - Tests
  keybindings_resource.rs - KeybindingsResource
  camera.rs           - Camera follow, chase camera, Vec3Json, CameraDefinition
  camera_tests.rs     - Tests
  debug_axes.rs       - Debug axis visualization (DebugAxes, DebugAxesEligible)
  debug_axes_tests.rs - Tests
  debug_config.rs     - DebugConfig
  debug_config_tests.rs - Tests
  diagnostics.rs      - DiagnosticsConfig, DiagnosticsPlugin
  diagnostics_tests.rs - Tests
  boundary.rs         - Sector boundary types
  boundary_systems.rs - Boundary checking system
  boundary_tests.rs   - Tests
  flight_assist.rs    - FlightAssist component/config
  floating_origin.rs  - FloatingOrigin types
  floating_origin_tests.rs - Tests
  ship_templates.rs   - Ship template types (BoundingBox, ShipCollisionShape, etc.)
```

Problems:
1. **Too many files in one directory** — 23 files in `src/` with no structure.
2. **Mixed concerns** — debug utilities, camera, input, physics types, and template
   types all in one crate.
3. **Unclear boundaries** — an agent looking for "where do I add a new shared
   component?" has no guidance.
4. **Debug files dominate** — debug_axes, debug_config, and their tests are 4 of
   23 files, yet debug is a cross-cutting concern, not a core responsibility.

## Decision

Restructure `delta-v-core` into subdirectories by concern. The crate remains the
single root crate that all others depend on, but its internal structure becomes
hierarchical.

### New directory structure

```
delta-v-core/src/
  lib.rs                    - Plugin, re-exports, top-level documentation
  state/
    mod.rs                  - AppState, state transition systems
    tests.rs                - State transition tests
  events/
    mod.rs                  - SpawnEntity event
  input/
    mod.rs                  - ActiveActions, LogicalAction, input systems
    keybindings_resource.rs - KeybindingsResource (moved from root)
    sets.rs                 - InputSet system set ordering
    tests.rs                - Input tests
  camera/
    mod.rs                  - CameraFollow, ChaseCameraOffset, chase_camera_system
    types.rs                - CameraDefinition, ShipCamerasTemplate
    tests.rs                - Camera tests
  debug/
    mod.rs                  - Re-exports of debug types
    axes.rs                 - DebugAxes, DebugAxesEligible, DebugAxisRootMarker
    axes_system.rs          - mark_debug_axes, spawn_debug_axes, update systems
    axes_tests.rs           - Debug axes tests
    config.rs               - DebugConfig
    config_tests.rs         - Debug config tests
  diagnostics/
    mod.rs                  - DiagnosticsConfig, DiagnosticsPlugin
    tests.rs                - Diagnostics tests
  boundary/
    mod.rs                  - BoundaryBehavior, SectorBoundary, SectorBoundaryResource
    systems.rs              - check_sector_boundary_system
    tests.rs                - Boundary tests
  flight_assist/
    mod.rs                  - FlightAssist, FlightAssistConfig, FlightAssistState
  floating_origin/
    mod.rs                  - FloatingOrigin, FloatingOriginConfig, FloatingOriginEligible
    tests.rs                - Floating origin tests
  spawn/
    mod.rs                  - WorldSpawnSet system sets
    sets.rs                 - (same content, moved from spawn_sets.rs)
```

### Rules for delta-v-core modules

1. **`state/`** — The `AppState` state machine and its transition systems. Nothing
   else. This is the root of the state hierarchy.
2. **`events/`** — Cross-domain events (like `SpawnEntity`). Events defined here
   are the ONLY events that cross domain boundaries.
3. **`input/`** — The logical input pipeline. `KeybindingsResource` lives here
   because it's needed by the input translation system.
4. **`camera/`** — Camera follow and chase systems. `Vec3Json` and camera template
   types move to `delta-v-types` (ADR-0046).
5. **`debug/`** — All debug visualization. This is a self-contained subdirectory
   because debug is a cross-cutting concern that touches multiple systems.
6. **`diagnostics/`** — Performance diagnostics (frame time monitoring).
7. **`boundary/`** — Sector boundary checking.
8. **`flight_assist/`** — Flight assist component and config.
9. **`floating_origin/`** — Floating origin types and config.
10. **`spawn/`** — System sets for spawn ordering (`WorldSpawnSet`).

### What leaves delta-v-core

Per ADR-0046, the following types move to `delta-v-types`:
- `BoundingBox` → `delta-v-types/src/spatial.rs`
- `PhysicalQuantity` → `delta-v-types/src/physics.rs`
- `Vec3Json` → `delta-v-types/src/spatial.rs`

Per ADR-0047, ship template types stay in `delta-v-core` temporarily but should
eventually move to `delta-v-ships` (they are ship-specific, not core).

### Naming conventions for new files in delta-v-core

1. Each subdirectory has a `mod.rs` that contains the primary types/systems.
2. Tests live in `tests.rs` within the same subdirectory.
3. Large modules MAY be split into additional files within the subdirectory
   (e.g., `axes.rs` and `axes_system.rs` in `debug/`).
4. The crate root `lib.rs` MUST re-export all public types.

## Consequences

- **Positive**: Clear directory structure makes it obvious where to add new code.
- **Positive**: Debug code is isolated in its own subdirectory.
- **Positive**: Each subdirectory is small enough to understand at a glance.
- **Migration effort**: Moderate — mostly moving files and updating `mod.rs` declarations.
  No logic changes needed.

## Related

- ADR-0002 (Repository layout)
- ADR-0005 (Plugin architecture)
- ADR-0018 (State management)
- ADR-0046 (Shared types crate)
