# ADR-0030: Authoritative model

- **Status**: Proposed (concept stage; needs concrete design before
  M7)
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Multiplayer architecture is a load-bearing decision that touches
physics determinism, networking library choice, cheating resistance,
NAT traversal and the relationship with third-party services (Steam,
custom relay servers). It is too early to commit to an
implementation, but we need a documented intent so that physics,
state management and input handling are designed in a way that does
not preclude the eventual model.

## Decision (intent)

The intended multiplayer model is **peer-to-peer with optional
"player as server"** semantics:

- Any player can mark their instance as the **session host**. Other
  players connect to that host.
- The host instance is the authoritative simulator. Clients send
  inputs; the host runs the simulation and sends back state.
- This is functionally close to a client-server model where the
  server happens to be the host's process. The architecture is
  "client-server", the deployment is "P2P".
- **NAT traversal**: players behind firewalls and NAT must be able
  to find each other. Options under consideration:
  - Steam networking (relay + matchmaking provided by Valve), via
    the Steamworks SDK, with the licence considerations of
    [ADR-0026](0026-release-process.md) and
    [ADR-0027](0027-open-source-licensing.md).
  - Public STUN/TURN servers operated by the project or by third
    parties.
  - Manual configuration (port forwarding) as a last-resort
    fallback.
- **No dedicated game servers** owned by the project at this stage.
  Hosting is by players, for players.
- **Trust model**: we accept that a malicious host can cheat. The
  game is cooperative-or-competitive among friends; we do not aim
  to defeat a determined cheater on day one. Anti-cheat is an
  explicit non-goal for the early multiplayer.

What we do **not** commit to in this ADR:

- The exact wire protocol, transport (UDP vs. WebRTC vs. Steam
  Datagram Relay), or serialisation format. Those are
  [ADR-0031](0031-network-library-choice.md) and
  [ADR-0032](0032-snapshot-and-delta-encoding.md).
- Lockstep simulation. Per
  [ADR-0017](0017-fixed-timestep-and-determinism.md), we are not
  aiming for cross-platform bit-exact determinism, which would be
  a prerequisite for lockstep.

## Consequences

Positive (of the intended model):

- No infrastructure costs for the project.
- Familiar mental model for players ("I host a session, you
  join").
- The architecture maps cleanly to single-player (server and client
  collapse into one process).

Negative (we accept):

- The hosting player needs a half-decent connection.
- Cheating is possible by the host. Mitigated by social trust,
  which is the same way most P2P games handle it.
- NAT traversal is a real engineering problem; without Steam or a
  relay it can be unreliable across asymmetric networks.

Follow-up:

- Concrete library choice in
  [ADR-0031](0031-network-library-choice.md).
- Encoding details in [ADR-0032](0032-snapshot-and-delta-encoding.md).
- A future ADR may revisit dedicated server hosting if the project
  grows beyond P2P scale.
