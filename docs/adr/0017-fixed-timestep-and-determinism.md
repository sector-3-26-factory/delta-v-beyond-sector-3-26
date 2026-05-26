# ADR-0017: Fixed timestep and determinism

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

A space game with Newtonian physics, projectile inheritance and
gravity needs a stable simulation cadence. Running physics directly
in the render loop ties simulation rate to frame rate, with the
classic consequences: physics behaves differently at 30 fps than at
240 fps; players with faster PCs effectively "experience" the game
differently.

A separate concern is **determinism**: given the same starting state
and the same inputs, does the simulation produce the same outputs?
Deterministic simulation makes replays cheap, bug reports
reproducible, and (eventually) certain multiplayer architectures
possible.

## Decision

**Fixed timestep for simulation**:

- Physics, gameplay state, AI and any other simulation system run
  inside Bevy's `FixedUpdate` schedule at a fixed rate.
- The initial rate is **60 Hz** (a fixed step of `1 / 60` seconds).
  This is high enough for fast projectiles and low enough to be
  affordable on modest hardware.
- Rendering runs at whatever the display can deliver, decoupled
  from simulation. Interpolation between simulation steps is added
  later if needed.
- If a frame falls behind, Bevy's `FixedUpdate` catches up by
  running multiple sub-ticks; the maximum number of catch-up ticks
  per frame is bounded to prevent a "spiral of death". The bound is
  configurable; the initial value is `4`.

**Determinism, as far as practical**:

- Simulation systems do not read wall-clock time
  (`std::time::Instant::now()`, `SystemTime::now()`); they use the
  fixed-step delta provided by Bevy.
- Iteration over unordered collections (`HashMap`, `HashSet`) in
  hot or simulation-relevant paths is replaced with ordered
  iteration (`BTreeMap`, `IndexMap`, or sorted iteration over a
  `Vec`).
- Gravity contributions are summed in a fixed order (e.g. by stable
  entity id), per [ADR-0009](0009-newtonian-physics-with-gravity.md).
- Random number generation, when used by simulation, draws from an
  explicit, seeded RNG that lives in an ECS resource; never from
  thread-local randomness.
- We do **not** commit to cross-platform bit-exact determinism.
  Aiming for it would constrain us to a small subset of f32
  operations and cost performance for a benefit we do not yet need
  (we are not building lockstep networking). Same-platform,
  same-binary determinism is the target.

**Implications for networking**:

- Multiplayer (per [ADR-0030](0030-authoritative-model.md)) will use
  a server-authoritative or client-authoritative model where minor
  divergence between clients is acceptable and corrected by
  snapshots, not lockstep.
- Replays record inputs against a known seed and a known simulation
  rate; on the same binary they reproduce the original session.

## Consequences

Positive:

- Simulation behaviour is independent of frame rate.
- Bug reproduction is cheap: same inputs, same outcome.
- Performance characteristics are predictable; we know how often
  each system runs per second.

Negative:

- Render-side interpolation will eventually be needed to avoid
  visible stepping at high frame rates; deferred until visible.
- Catch-up logic must be bounded to prevent runaway pauses on
  systems that briefly fall behind. The bound itself is a tunable.
- The "no wall-clock in simulation" rule must be enforced by review;
  there is no easy lint for it.

Follow-up:

- A small documented helper, e.g. a `SimRng` resource, is added in
  the physics or core crate when first needed.
- Performance budget per fixed tick is tracked in
  [ADR-0022](0022-performance-instrumentation.md).
