# ADR-0032: Snapshot and delta encoding

- **Status**: Proposed (decision deferred to M7 preparation)
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

In a server-authoritative multiplayer game, the server periodically
sends each client the relevant world state. Sending the **full**
state every tick is bandwidth-prohibitive once the world has more
than a handful of dynamic entities. The standard remedy is:

- **Snapshots**: at a low frequency (e.g. 1 Hz), the full relevant
  state for the client.
- **Deltas**: between snapshots (e.g. at simulation tick rate), only
  the difference from the most recently acknowledged snapshot.

The client maintains the last acknowledged snapshot, applies deltas
on top, and interpolates between received states for smooth
rendering.

This is well-trodden ground in shooter and simulation netcode; we
will inherit much of it from whichever library we eventually pick
([ADR-0031](0031-network-library-choice.md)), but the *content* of
snapshots and deltas (which components are replicated, at what
quantisation, with which prediction model) is project-specific.

## Decision (intent)

The intended approach, to be refined when networking is implemented:

- **Replicated components** are explicitly marked. There is no
  automatic "all components are replicated" rule. Each replicated
  component declares the set of fields it sends.
- **Quantisation**: positions and velocities may be quantised
  before transport (e.g. position to centimetre precision,
  velocity to decimetre-per-second). The choice of quantisation
  per field is part of the replicated-component declaration.
- **Snapshot/delta cadence**: target snapshot rate ~1 Hz, delta
  rate matches the fixed-update simulation tick (initial
  60 Hz per [ADR-0017](0017-fixed-timestep-and-determinism.md)),
  subject to bandwidth tuning.
- **Interest management**: a client receives data only for the
  region of the world it can perceive (typically a sphere around
  the player's ship). Outside that region, updates are coarser or
  omitted.
- **Reliability tiers**: snapshots and structural events (entity
  spawn/despawn, RPCs) are reliable; per-tick deltas are
  unreliable but timestamped and idempotent.
- **Floating origin**: positions on the wire are in a stable frame
  (e.g. sector-relative) so that the
  per-client floating origin (per
  [ADR-0007](0007-floating-origin.md)) does not corrupt cross-
  client agreement.

We do **not** commit to:

- A specific serialisation format (`bincode`, MessagePack, custom
  bit-packing) until the library is chosen.
- Client-side prediction or rollback. They may be added later if
  the gameplay needs it; until M7 we operate without them.

## Consequences

Positive:

- The replication model is explicit, not "everything by default".
- Quantisation is an early consideration, not a late optimisation.
- Floating origin is reconciled with networking on paper before
  it becomes a debugging story.

Negative:

- Marking components for replication is more boilerplate than
  automatic replication.
- Designing per-component quantisation up front takes thought; we
  accept this because retrofitting it is much worse.

Follow-up:

- The concrete encoding details live in a successor ADR written
  alongside the networking implementation.
- A small benchmark harness measures bandwidth at the chosen
  snapshot/delta rate once we have something to measure.
