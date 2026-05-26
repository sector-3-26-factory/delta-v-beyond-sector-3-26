# ADR-0007: Floating origin for large worlds

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

`f32` (Bevy's and avian3d's default for `Transform` and physics) has
about 7 significant decimal digits. Once a position exceeds ~10 km
from the origin, vertices visibly jitter, the camera shakes, and
physics loses precision. This is a well-known issue in open-world
games.

Our content can describe interstellar distances:

| Reference                | Distance from origin       | f32 precision at that range |
| ------------------------ | -------------------------- | --------------------------- |
| Earth -> Moon            | ~3.8 x 10^8 m              | ~16 m                       |
| Earth -> Sun (1 AU)      | ~1.5 x 10^11 m             | ~16 km                      |
| Alpha Centauri           | ~4.0 x 10^16 m             | ~4 x 10^9 m                 |
| Galactic centre          | ~2.5 x 10^20 m             | unusable                    |

We do, however, intend to keep moment-to-moment gameplay inside one
**sector** at a time. Travel between sectors is mediated by a
hyperdrive or by hyperspace gates, both of which are natural seams at
which we can re-centre or load.

## Decision

We use the **floating origin** technique:

- All physics and rendering happen in `f32` world coordinates relative
  to a movable origin.
- Whenever the player ship's distance from the current origin exceeds
  a configured threshold (initial value: a few kilometres), the
  origin is **recentred** on the ship and every other entity's
  `Transform` is translated by the inverse offset in a single system
  pass.
- The recenter operation is bookkeeping only; from the player's point
  of view nothing visibly happens.
- Between sectors, the origin is reset as part of the sector load
  (hyperjump destination or gate exit).
- Astronomical entities at very large nominal distances (background
  stars, distant galaxies) are rendered via a separate "skybox / far
  field" mechanism that does *not* live in physics space. They are
  data, not simulated objects.

The recenter threshold, the maximum allowed nominal sector size and
the policy for moving non-physical entities (particles, audio
emitters) along with the origin are tracked as configuration with
sensible JSON-schema defaults.

We do **not** adopt:

- `f64` everywhere -- intrusive, no first-class Bevy support, and
  unnecessary if floating origin is in place.
- Hierarchical chunk/cell coordinates -- more complex than we need
  before there is a real gameplay reason.

## Consequences

Positive:

- `f32` remains the engine-wide numeric type; no fork of Bevy or
  avian3d needed.
- Within a sector, precision is constant regardless of how far the
  player has travelled.
- The recentre point doubles as a natural place to garbage-collect
  far-away streamed assets later.

Negative:

- A small but real subsystem must exist that translates every
  `Transform`, particle, audio emitter, etc. when recentering. Bugs
  here manifest as objects "drifting" relative to the world.
- Networking and replays must be aware of the origin so that
  positions are exchanged in a stable frame, not the local one.
- Strictly cross-platform-deterministic physics is harder when
  origins differ between clients; this interacts with
  [ADR-0017](0017-fixed-timestep-and-determinism.md) and
  [ADR-0030](0030-authoritative-model.md).

Follow-up:

- A dedicated ADR may follow once a concrete implementation is in
  place, capturing the exact recenter algorithm and the rule for
  what does and does not get translated.
- Interstellar travel (between sectors) is treated as a discrete
  event in the game logic, not a continuous simulation.
