# ADR-0022: Performance instrumentation

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Performance regressions creep into a codebase that does not measure
itself. Without instrumentation, we discover that a frame takes
40 ms only when the game noticeably hitches. We need a low-friction
way to attribute time to systems and to flag obvious overruns.

We are explicitly not picking a numeric budget yet. The point of
this ADR is to make sure we *can* measure when the time comes.

## Decision

Two complementary mechanisms:

1. **`tracing` spans on every non-trivial system**:
   - Each Bevy system that is expected to do real work wraps its
     body in a `tracing::info_span!("system_name")` or uses the
     `#[instrument]` attribute.
   - Span targets match the crate (`delta_v_physics`,
     `delta_v_ships`, ...) so that
     `RUST_LOG=delta_v_physics=trace` produces per-system timing
     when desired.
   - We do not pre-emptively instrument trivial getters; this is
     for systems whose cost is plausibly visible at a frame
     budget.

2. **Frame-time threshold warning**:
   - A diagnostic plugin observes `bevy_diagnostic::FrameTimeDiagnosticsPlugin`
     output and emits a `WARN` (per
     [ADR-0015](0015-logging-strategy.md)) when frame time exceeds
     a configured threshold for more than N consecutive frames.
   - Initial threshold: 33 ms (i.e. drops below 30 fps), N = 60
     (roughly one second of sustained slowness). Both are JSON-
     configurable per
     [ADR-0010](0010-configuration-system.md), with sensible
     defaults shipped in `assets/config/diagnostics.json`.

Profiling-friendly builds:

- A `--features profile` cargo feature pulls in `tracing-tracy`
  (or `puffin`, decision deferred) and exports spans to an
  external profiler. Off by default in `--release`.
- A `--features dev` build includes the diagnostic plugin
  unconditionally; release builds include it but with the warning
  threshold raised by default.

What we do **not** do:

- We do not collect telemetry from end users.
- We do not write profiling data to disk by default in shipping
  builds.

## Consequences

Positive:

- "What is taking so long" has a one-command answer.
- Sudden regressions emit a log warning instead of a silent slow
  game.
- Profiler integration is a feature flag away when needed.

Negative:

- Span overhead is non-zero. Bevy's tracing macros are very cheap
  but not free. We accept the cost; if it becomes visible in hot
  systems we drop the span there.
- "When to add a span" is a judgement call. We err on the side of
  adding one to any system that loops over more than a constant
  number of entities.

Follow-up:

- The diagnostic plugin and the helper to log a warning when frame
  time spikes are added in `delta-v-core` (or in the binary,
  depending on layering).
- A concrete frame-time budget (per system, per category) is
  a future ADR or an addendum to this one.
