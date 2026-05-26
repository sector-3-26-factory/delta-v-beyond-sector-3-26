# ADR-0015: Logging strategy

- **Status**: Accepted (initial; refine as the project grows)
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Bevy logs through the `tracing` ecosystem via `bevy_log`. Without
a project convention, log output drifts toward "every developer
logs at the level they feel like", and the resulting noise hides
the few messages that actually matter.

## Decision

We adopt a small, explicit set of rules for what gets logged at
which level. We start with these and refine in later ADRs if a
specific case justifies it.

**Levels**:

- `ERROR`: the game cannot proceed normally. A required asset is
  missing, a network connection died unrecoverably, an invariant is
  violated. Always accompanied by enough context to act on.
- `WARN`: something unexpected happened but the game continues. A
  non-critical asset failed to load and a `--features dev`
  placeholder was substituted; a frame took longer than the budget
  (see [ADR-0022](0022-performance-instrumentation.md)); a
  deprecated configuration field was used.
- `INFO`: lifecycle events of interest. App start, plugins added,
  world loaded, player joined, hyperjump completed. Roughly: things
  a user might mention in a bug report.
- `DEBUG`: developer-facing detail. State transitions, important
  input events, large allocations, asset-load timings. Off by
  default in release builds.
- `TRACE`: per-frame or per-tick spam. Physics step timings, input
  axis values, network packet contents. Off unless explicitly
  enabled.

**Format**:

- Bevy's default `LogPlugin` formatter is acceptable for now.
- Timestamps are ISO 8601 with timezone (already Bevy's default).
- Log messages are written in English, full sentences are not
  required, but they should be searchable strings.

**Configuration**:

- Default level is `INFO` for our crates and `WARN` for third-party
  crates, to keep release output readable.
- Override via the `RUST_LOG` / `BEVY_LOG` environment variables.
- `--features dev` builds default to `DEBUG` for our own crates.

**Per-domain filters**:

- Each domain uses `tracing` targets matching its crate name
  (e.g. `delta_v_physics`). This allows
  `RUST_LOG=delta_v_physics=trace,info` to enable detailed physics
  logs without drowning in unrelated output.

**Things that must not be logged**:

- Personally identifiable information.
- Full file paths from the user's home directory if avoidable
  (relative paths are preferred).
- Secrets or auth tokens of any kind (we have none yet; this rule
  is preventive).

## Consequences

Positive:

- Log output is predictable across crates.
- Bug reports become more useful: an `INFO` line is meaningful, and
  filtering by target works.

Negative:

- Some judgement remains: "is this a warn or a debug?". We accept
  occasional inconsistency rather than enumerating every possible
  case.
- The `--features dev` default-to-DEBUG might be noisy at first;
  if it becomes a problem we tighten the defaults.

Follow-up:

- A small `init_logging()` helper in `delta-v-core` (or in the
  binary) wires the above defaults so individual crates do not each
  duplicate the setup.
- Performance-warning thresholds and span names are tracked in
  [ADR-0022](0022-performance-instrumentation.md).
