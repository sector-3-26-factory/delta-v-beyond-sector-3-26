# ADR-0009: Newtonian physics with gravitational interaction

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

[`docs/physics.md`](../physics.md) already commits the project to
Newtonian flight: no top speed, momentum conservation, thrust as
force. This ADR records two refinements that are easy to overlook:

1. **Mass affects trajectories.** A heavier ship accelerates more
   slowly under the same thrust; a stray cargo container affects
   collision outcomes; a star's mass curves nearby trajectories
   through gravity. This is not optional flavour; it is the rule that
   makes the rest of the simulation feel coherent.

2. **Gravitational attraction between simulated bodies** is part of
   the model, not just a fixed-vector "down". Planets pull ships;
   stars pull planets; sufficiently massive ships could in principle
   pull other ships (though in practice their mass is many orders of
   magnitude too small for this to matter).

## Decision

The physics simulation is Newtonian with mass-coupled dynamics:

- Each rigid body has a `mass` (kg) and an inertia tensor derived from
  its mass and geometry. Translation obeys `F = m * a`; rotation obeys
  `tau = I * alpha`.
- Bodies designated as **gravity sources** (stars, planets, moons,
  large stations) generate a gravitational acceleration field
  `g = G * M / r^2` on every other body within a configured cutoff
  radius. The cutoff prevents the full O(N^2) interaction cost.
- Ordinary ships and projectiles are **gravity receivers**, not
  sources, unless explicitly marked otherwise.
- The set of receivers actually integrated against a given source is
  pruned by the cutoff. Sources beyond cutoff contribute zero.
- Projectiles inherit the firing ship's world velocity at the moment
  of firing, as already stated in `docs/physics.md`.
- "Flight assist" (inertial damping for accessibility) acts only by
  commanding existing thrusters, never by violating conservation
  laws.

Why not full N-body:

- O(N^2) cost is prohibitive in a sector with many entities.
- Cutoff radius plus designating only massive bodies as sources gives
  a good approximation at a fraction of the cost.

Why include gravity at all (versus a flat "no gravity in space"
model):

- Slingshot manoeuvres, stable orbits around stations, and the
  intuitive feeling that the world has mass distinguish a Newtonian
  space game from arcade flight.

## Consequences

Positive:

- Mass becomes a meaningful design knob for ships, cargo and
  weapons.
- Stable orbits and gravity assists fall out of the simulation for
  free.
- Designers can place massive objects as gameplay obstacles ("don't
  fly too close to the pulsar").

Negative:

- Mass values must be specified for every simulated body. Schemas
  enforce this (per [ADR-0013](0013-no-silent-fallbacks.md)).
- Gravity sources need careful tuning: an "Earth-like" planet at
  realistic mass and distance scales would be either too strong or
  too distant for fun gameplay. Designers will sometimes pick
  gameplay over realism, and that is acceptable.
- Determinism (see [ADR-0017](0017-fixed-timestep-and-determinism.md))
  requires that the order in which gravity contributions are summed
  is fixed.

Follow-up:

- The `delta-v-physics` crate implements gravity integration as a
  Bevy system running inside the fixed-timestep schedule.
- The cutoff radius is a per-source configurable property in JSON.
- A debug overlay visualises gravity sources and their influence
  ranges (deferred until needed).
