# ADR-0044: Prohibit visual data handling within Rust code

<!--
Agents and contributors: read AGENTS.md at the repository root before
working with this document. AGENTS.md links to the ADR index and to all
rules that govern this project.
-->

- **Status**: Accepted
- **Date**: 2026-06-06
- **Deciders**: Project maintainers
- **Supersedes**: (none)
- **Superseded by**: (none)

## Context

The delta-v project is a space simulation game that requires a clear separation between core simulation logic and visual rendering. As the project grows in complexity, there is an increasing risk that visual data handling (textures, meshes, shaders, render targets) could creep into the Rust simulation code, creating tight coupling between simulation and rendering concerns.

This coupling would violate several architectural principles:

1. **ADR-0005 (Plugin Architecture)**: Domain plugins should communicate via events and components, not direct function calls. Visual data handling in simulation code creates implicit dependencies.

2. **ADR-0019 (Asset Pipeline)**: Shipped assets live under `assets/` and user content is separate. Visual data should be loaded through the asset pipeline, not generated in code.

3. **ADR-0028 (Third-party Dependency Policy)**: Visual data processing often requires heavy dependencies (image processing libraries, mesh generation libraries) that increase compile times and binary size.

4. **ADR-0034 (No Warnings)**: Generated visual data often produces compiler warnings that are difficult to suppress without blanket allows.

5. **ADR-0041 (Third-party Asset Acquisition)**: All 3D assets must come from GPL-compatible sources with proper attribution. Generated meshes cannot be credited to creators.

Technical constraints and performance considerations:

- **Compile times**: Visual data generation in Rust increases compilation time significantly
- **Binary size**: Embedded visual data increases binary size
- **Memory usage**: Generated visual data duplicates memory that could be GPU-resident
- **Testing**: Visual data generated in code is difficult to test deterministically
- **Hot-reload**: Generated visual data cannot be hot-reloaded like external assets
- **Determinism**: Visual data generation may introduce non-deterministic behavior

## Decision

We **prohibit** all visual data handling within Rust code in the delta-v project. This includes:

- **Textures**: No runtime texture generation, no image processing in Rust code
- **Meshes**: No primitive mesh generation (spheres, boxes, etc.) in Rust code
- **Shaders**: No shader compilation or shader code strings in Rust
- **Render targets**: No offscreen render target creation in Rust
- **Materials**: No material property computation in Rust code
- **Animations**: No animation curve generation in Rust code

All visual data must come from external files:

- **3D models**: glTF 2.0 format (`.glb` files) in `assets/templates/<type>/<name>/mesh.glb`
- **Textures**: Image files (PNG, JPEG) loaded via Bevy's asset server
- **Materials**: Defined in glTF files, not in Rust code

Domain plugins handle visual data through the following pattern:

1. **Spawn systems** queue glTF loading via `AssetServer::load::<Gltf>()`
2. **Marker components** (e.g., `PendingAsteroidMesh`, `PendingShipMesh`) track loading state
3. **Attachment systems** run in `InGame` state and attach loaded scenes via `SceneBundle`

This pattern is already implemented for ships (`delta-v-ships/src/spawn.rs`) and asteroids (`delta-v-world/src/asteroid_spawner.rs`).

## Consequences

### Positive consequences

- **Separation of concerns**: Simulation code remains pure and testable
- **Faster compile times**: No visual data generation code to compile
- **Smaller binaries**: No embedded visual data
- **Hot-reload support**: Visual assets can be modified without recompilation
- **Asset attribution**: All visual assets can be credited to their creators
- **Deterministic testing**: Simulation tests do not depend on visual data
- **Clear boundaries**: Plugin architecture is preserved

### Negative consequences

- **Initial setup overhead**: Need to create glTF files for all visual entities
- **External tool dependency**: Requires 3D modeling tools for asset creation
- **File management**: More files to track in the repository
- **Loading latency**: Visual data loads asynchronously, requiring marker components

### Follow-up work

- Audit existing code for any visual data generation (should be none)
- Ensure all domain plugins follow the spawn/marker/attachment pattern
- Document the asset creation pipeline for contributors
- Add CI check to reject PRs that add visual data generation in Rust

## Notes

- This decision applies only to runtime visual data. Build-time code generation (e.g., for constants) is allowed.
- The `bevy::gltf::Gltf` asset type is the only exception - we load glTF files, not generate them.
- **Debug/diagnostic visual data is exempt**: Debug axes, collision shape visualization, and other developer tooling are not subject to this rule since they do not impact player experience or gameplay.
- See also ADR-0019 (Asset pipeline), ADR-0041 (Third-party asset acquisition), ADR-0043 (Ship template split).