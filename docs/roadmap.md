# Roadmap

This document refines the milestone roadmap originally sketched in
[`design.md`](design.md). It is a living document; dates are
intentionally absent.

If you have not read [`AGENTS.md`](../AGENTS.md), read it first.

## Milestones

Each milestone ends on a runnable, demoable build.

### M0 -- Skeleton
*Status: complete.*

- Project scaffolding, license, CI, dev container.
- `cargo run` opens an empty Bevy window.
- GPU-accelerated rendering verified inside the dev container.

### M0.5 -- Architecture foundation
*Status: complete.*

- AGENTS.md created.
- ADR process established
  ([ADR-0001](adr/0001-adr-process.md)).
- All foundation ADRs (0001-0035) written and adopted; ADR-0036
  (external contributors) and ADR-0037 (i18n) also adopted.
- `docs/workflow.md`, `docs/architecture.md`, `docs/roadmap.md`
  populated.
- Workspace conversion (binary + 11 library crates per
  [ADR-0002](adr/0002-repository-layout-and-workspace.md)).
- `cargo deny` configuration (`deny.toml`) and CI integration:
  audited on every push and PR; weekly scheduled re-audit via
  `.github/workflows/security.yml`
  ([ADR-0028](adr/0028-third-party-dependency-policy.md),
  [ADR-0029](adr/0029-security-and-supply-chain.md)).
- `--locked` enforced on all CI cargo invocations
  ([ADR-0029](adr/0029-security-and-supply-chain.md)).
- Source file header convention rolled out
  ([ADR-0033](adr/0033-agents-md-and-source-file-pointers.md)).
- Pre-commit hook script
  ([ADR-0034](adr/0034-no-warnings-policy.md)).

### M1 -- A ship in space
- 3D scene with a primitive "ship" mesh and a chase camera.
- Default world loaded automatically at startup, defined in JSON +
  glTF
  ([ADR-0019](adr/0019-asset-pipeline-and-user-content.md),
  [ADR-0020](adr/0020-save-and-load-format.md)).
- Keybindings loaded from JSON, default + user override
  ([ADR-0011](adr/0011-keybindings-configuration.md)).
- Input wired but inert (logged only).
- Logging strategy and frame-time diagnostics in place
  ([ADR-0015](adr/0015-logging-strategy.md),
  [ADR-0022](adr/0022-performance-instrumentation.md)).

### M2 -- Newtonian flight
- Apply forces and torques from input to a rigid body
  ([ADR-0009](adr/0009-newtonian-physics-with-gravity.md)).
- 6 degrees of freedom, no speed cap.
- Optional flight-assist toggle for inertial damping.
- Fixed timestep in place
  ([ADR-0017](adr/0017-fixed-timestep-and-determinism.md)).

### M3 -- Collisions and the world
- Static obstacles (asteroids), collision response via avian3d.
- Sector boundaries.
- Floating origin engaged
  ([ADR-0007](adr/0007-floating-origin.md)).

### M4 -- Weapons
- Status: **complete**.
- Projectile weapons that inherit the firing ship's velocity.
- Damage model on rigid bodies.

### M5 -- Enemies
- Minimal scripted / state-machine AI opponents.
- Win/lose conditions per skirmish.

### M6 -- HUD and feel
- Status: **in progress**.
- Cockpit overlay system with PNG overlays and station switching (Step 5 implemented).
- Velocity vector indicator, thrust gauges, target reticle.
- Sound effects, camera shake, basic VFX.

### M7 -- Multiplayer prototype
- Concrete networking decisions
  ([ADR-0030](adr/0030-authoritative-model.md),
  [ADR-0031](adr/0031-network-library-choice.md),
  [ADR-0032](adr/0032-snapshot-and-delta-encoding.md)) reach
  Accepted status.
- 2-player deathmatch in a single sector.

### M8 -- Content and polish
- Multiple sectors, ship variants, hyperdrive / hyperspace gates.
- Audio pass, settings menu, in-game rebinding UI.
- First playable build; first release tag candidate
  ([ADR-0025](adr/0025-versioning.md),
  [ADR-0026](adr/0026-release-process.md)).

## Out of scope (for now)

- Persistent universe, economy, trading.
- Procedural galaxy generation.
- Mobile / console ports.
- Steam integration (deferred to a post-M8 stage; intent recorded
  in [ADR-0026](adr/0026-release-process.md) and
  [ADR-0031](adr/0031-network-library-choice.md)).

## Open questions

These are tracked here so they are not forgotten:

- Exact flight-assist model: pure damping vs. target-velocity?
- Art direction: stylised low-poly vs. semi-realistic?
- Single-binary distribution vs. separate launcher.
