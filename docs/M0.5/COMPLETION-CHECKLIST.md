# Milestone M0.5 Completion Checklist

Status: **COMPLETE**

This document records the completion of all M0.5 tasks as defined in
[`../roadmap.md`](../roadmap.md).

## ADR Process and Foundation ADRs

- [x] [ADR-0001](../adr/0001-adr-process.md) -- ADR process established
- [x] [ADR-0002](../adr/0002-repository-layout-and-workspace.md) -- Repository layout and workspace
- [x] [ADR-0003](../adr/0003-branching-and-pr-workflow.md) -- Branching and PR workflow
- [x] [ADR-0004](../adr/0004-commit-message-convention.md) -- Commit message convention
- [x] [ADR-0005](../adr/0005-plugin-architecture.md) -- Plugin architecture
- [x] [ADR-0006](../adr/0006-coordinate-system-and-units.md) -- Coordinate system and units
- [x] [ADR-0007](../adr/0007-floating-origin.md) -- Floating origin
- [x] [ADR-0008](../adr/0008-physical-units-in-json.md) -- Physical units in JSON
- [x] [ADR-0009](../adr/0009-newtonian-physics-with-gravity.md) -- Newtonian physics with gravity
- [x] [ADR-0010](../adr/0010-configuration-system.md) -- Configuration system
- [x] [ADR-0011](../adr/0011-keybindings-configuration.md) -- Keybindings configuration
- [x] [ADR-0012](../adr/0012-json-schema-validation.md) -- JSON schema validation
- [x] [ADR-0013](../adr/0013-no-silent-fallbacks.md) -- No silent fallbacks
- [x] [ADR-0014](../adr/0014-engine-constants-vs-gameplay-values.md) -- Engine constants vs gameplay values
- [x] [ADR-0015](../adr/0015-logging-strategy.md) -- Logging strategy
- [x] [ADR-0016](../adr/0016-error-handling-strategy.md) -- Error handling strategy
- [x] [ADR-0017](../adr/0017-fixed-timestep-and-determinism.md) -- Fixed timestep and determinism
- [x] [ADR-0018](../adr/0018-state-management.md) -- State management
- [x] [ADR-0019](../adr/0019-asset-pipeline-and-user-content.md) -- Asset pipeline and user content
- [x] [ADR-0020](../adr/0020-save-and-load-format.md) -- Save and load format
- [x] [ADR-0021](../adr/0021-testing-strategy.md) -- Testing strategy
- [x] [ADR-0022](../adr/0022-performance-instrumentation.md) -- Performance instrumentation
- [x] [ADR-0023](../adr/0023-code-style-and-lints.md) -- Code style and lints
- [x] [ADR-0024](../adr/0024-documentation-policy.md) -- Documentation policy
- [x] [ADR-0025](../adr/0025-versioning.md) -- Versioning
- [x] [ADR-0026](../adr/0026-release-process.md) -- Release process
- [x] [ADR-0027](../adr/0027-open-source-licensing.md) -- Open source licensing
- [x] [ADR-0028](../adr/0028-third-party-dependency-policy.md) -- Third-party dependency policy
- [x] [ADR-0029](../adr/0029-security-and-supply-chain.md) -- Security and supply chain
- [x] [ADR-0033](../adr/0033-agents-md-and-source-file-pointers.md) -- AGENTS.md and source file pointers
- [x] [ADR-0034](../adr/0034-no-warnings-policy.md) -- No warnings policy
- [x] [ADR-0035](../adr/0035-hot-reload-of-configs.md) -- Hot reload of configs in dev builds
- [x] [ADR-0036](../adr/0036-external-human-contributors-process.md) -- External human contributors process
- [x] [ADR-0037](../adr/0037-internationalization.md) -- Internationalization (i18n)

## Documentation

- [x] `AGENTS.md` -- Entry point for AI agents (already present)
- [x] `docs/workflow.md` -- Day-to-day workflow rules (already present)
- [x] `docs/architecture.md` -- Big-picture architecture with crate structure
- [x] `docs/roadmap.md` -- Milestone roadmap (already present)
- [x] `docs/adr/README.md` -- ADR index (already present)

## Workspace Conversion

- [x] Root `Cargo.toml` converted to workspace with members definition
- [x] `[workspace.dependencies]` section created for shared versions
- [x] `[workspace.lints]` section created for consistent linting
- [x] Binary crate `crates/delta-v/` created with plugin registration
- [x] Technical crate `crates/delta-v-core/` created
- [x] Technical crate `crates/delta-v-config/` created
- [x] Technical crate `crates/delta-v-physics/` created
- [x] Technical crate `crates/delta-v-assets/` created
- [x] Technical crate `crates/delta-v-net/` created
- [x] Domain crate `crates/delta-v-ships/` created
- [x] Domain crate `crates/delta-v-propulsion/` created
- [x] Domain crate `crates/delta-v-weapons/` created
- [x] Domain crate `crates/delta-v-stations/` created
- [x] Domain crate `crates/delta-v-items/` created
- [x] Domain crate `crates/delta-v-world/` created
- [x] All crates have proper `Cargo.toml` with workspace dependencies
- [x] All crates have `src/lib.rs` or `src/main.rs` with plugin stubs
- [x] Source file headers added with "See AGENTS.md" comment

## Tooling and CI

- [x] `cargo deny` configuration file (`deny.toml`) created
- [x] `cargo deny` integration for supply chain security
- [x] Pre-commit hook script (`.githooks/pre-commit`) created
- [x] Git configured to use `.githooks` directory
- [x] Hook enforces: `cargo fmt`, `cargo clippy`, `cargo test`, `cargo deny`

## Source File Header Convention (ADR-0033)

- [x] All source files have "See AGENTS.md" comment header
- [x] Header points to `AGENTS.md`, not to individual ADRs
- [x] Convention follows language-specific comment styles
- [x] GPL-3.0-or-later license headers included

## Next Steps (M1 -- A ship in space)

The following tasks belong to M1:
- 3D scene with a primitive "ship" mesh and a chase camera
- Default world loaded automatically at startup (JSON + glTF)
- Keybindings loaded from JSON (default + user override)
- Input wired but inert (logged only)
- Logging strategy and frame-time diagnostics in place

See [`../roadmap.md#m1----a-ship-in-space`](../roadmap.md) for details.
