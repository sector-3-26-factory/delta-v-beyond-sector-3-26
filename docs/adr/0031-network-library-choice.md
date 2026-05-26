# ADR-0031: Network library choice

- **Status**: Proposed (decision deferred to M7 preparation)
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Several Rust networking libraries integrate with Bevy and would each
push the project's architecture in a different direction. Choosing
prematurely commits us before we know what we need; choosing too
late costs a painful retrofit. This ADR records the candidates and
the criteria so that the eventual decision is informed.

## Candidates (as of 2026)

- **`lightyear`** -- a Bevy-focused networking framework with
  client-server semantics, interpolation, prediction, and good
  documentation. Active development.
- **`bevy_replicon`** -- a Bevy-native, minimal replication layer
  on top of `bevy_renet`/`bevy_quinnet`. Less batteries-included,
  more LEGO.
- **`bevy_quinnet`** / **`bevy_renet`** -- transport-only options
  (QUIC and a small reliable-UDP, respectively). Used as building
  blocks under the above.
- **Steam networking** via `steamworks-rs` -- Steam Datagram Relay
  handles NAT traversal and provides matchmaking. Strong fit for
  Stage 2 distribution
  (per [ADR-0026](0026-release-process.md)) but ties us to a
  proprietary SDK with GPL-incompatibility (per
  [ADR-0027](0027-open-source-licensing.md)).

## Decision (intent)

We **do not commit to a library in this ADR**. We commit to:

1. Designing physics, state and input now so that the wire-side
   surface is a small, well-defined module
   (`delta-v-net`) that can be swapped without touching the rest.
2. Picking the actual library at M7 preparation time, evaluating
   the candidates above against the requirements that have
   crystallised by then:
   - NAT traversal capability (relay, hole-punching, ...).
   - Compatibility with our authoritative model
     ([ADR-0030](0030-authoritative-model.md)).
   - Licence compatibility
     ([ADR-0027](0027-open-source-licensing.md)) -- this rules
     out direct linkage with the Steamworks SDK in the same
     binary.
   - Bevy integration quality (frequency of breakage when Bevy
     bumps, examples we can learn from).
   - Maintenance health ([ADR-0028](0028-third-party-dependency-policy.md)).
3. Treating Steam as a *transport option* available behind a
   feature flag, not as the only matchmaking story. Even if Steam
   Datagram Relay is used for one distribution channel, the game
   must remain playable on non-Steam channels via a non-Steam
   transport.

Until M7, `delta-v-net` is a stub: an empty plugin and the minimal
public surface needed to keep the workspace graph buildable.

## Consequences

Positive:

- We avoid premature commitment to a library that may have moved
  on by the time we need it.
- Architectural decisions made today (fixed timestep, plugin
  isolation, no wall-clock in simulation) keep all candidates
  viable.

Negative:

- A real ADR is still owed once a choice is made. Until then,
  multiplayer is genuinely undecided beyond intent.
- Designing a stub crate is a small upfront cost.

Follow-up:

- An evaluation comparing the candidates against the criteria above
  produces the eventual `Accepted` ADR.
- Snapshot/delta encoding is tracked separately in
  [ADR-0032](0032-snapshot-and-delta-encoding.md) and may inform
  the library choice.
