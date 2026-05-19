# Design notes

This document captures the high-level design intent and the milestone-based
roadmap of *Delta-V beyond Sector 3.26*. It is a living document; treat it
as a sketch, not a contract.

## Vision

A modern, open-source successor to *Parsec* / *OpenParsec*: fast, skill-based
3D space combat where mastering Newtonian flight is the core of the
gameplay loop. Single-player playable from day one, multiplayer as a
first-class long-term goal.

## Pillars

1. **Newtonian flight is the gameplay.** No magic top speed. Momentum and
   thrust budgeting are tactical resources. See [`physics.md`](physics.md).
2. **ECS-native architecture.** Bevy's ECS shapes every system; avoid
   monolithic "manager" objects.
3. **Deterministic where it matters.** Physics and gameplay simulation
   should be reproducible enough for replays and lockstep-style netcode
   later on.
4. **Approachable code base.** Small modules, clear boundaries, good docs,
   English everywhere.

## Tech stack

| Concern             | Choice                              |
| ------------------- | ----------------------------------- |
| Language            | Rust (edition 2021)                 |
| Engine              | [Bevy](https://bevyengine.org) 0.14 |
| Physics             | [avian3d](https://docs.rs/avian3d)  |
| Asset format        | glTF 2.0                            |
| Build / CI          | cargo, GitHub Actions               |
| Dev environment     | Dev Container (Docker, Linux host)  |
| Networking (future) | TBD (lightyear or bevy_replicon)    |

## Milestone roadmap

Each milestone must end on a runnable, demoable build.

### M0 -- Skeleton  *(current)*
- Project scaffolding, license, CI, dev container.
- `cargo run` opens an empty Bevy window.

### M1 -- A ship in space
- 3D scene with a primitive "ship" mesh and a chase camera.
- Keyboard input wired but inert.

### M2 -- Newtonian flight
- Apply forces and torques from input to a rigid body.
- 6 degrees of freedom, no speed cap.
- Optional "flight assist" toggle for inertial damping.

### M3 -- Collisions and the world
- Static obstacles (asteroids), collision response via avian3d.
- Bounding world volume / sector boundaries.

### M4 -- Weapons
- Projectile weapons that inherit the firing ship's velocity.
- Damage model on rigid bodies.

### M5 -- Enemies
- Minimal scripted / state-machine AI opponents.
- Win/lose conditions per skirmish.

### M6 -- HUD and feel
- Velocity vector indicator, thrust gauges, target reticle.
- Sound effects, camera shake, basic VFX.

### M7 -- Multiplayer prototype
- Client/server skeleton, 2-player deathmatch in a single sector.

### M8 -- Content and polish
- Multiple sectors, ship variants, audio pass, settings menu.

## Out of scope (for now)

- Persistent universe, economy, trading.
- Procedural galaxy generation.
- Mobile / console ports.

## Open questions

- Exact "flight assist" model: pure damping vs. target-velocity mode?
- Authoritative-server vs. lockstep for multiplayer?
- Art direction: stylised low-poly vs. semi-realistic?
