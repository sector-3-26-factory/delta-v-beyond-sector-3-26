# ADR-0056: Benchmark purposes and tooling

- **Status**: Accepted
- **Date**: 2026-10-01
- **Deciders**: Cute-Donkey
- **Supersedes**: ADR-0021 (benchmark tooling clause only; the test-layer
  rules of ADR-0021 are carried forward unchanged in this ADR)

## Context

[ADR-0021](0021-testing-strategy.md) defines the four test layers and states,
in its Conventions section:

> Performance tests (benchmarks) use `criterion` and live behind a
> `--features bench` flag so that they do not slow down day-to-day
> `cargo test`.

That clause mandates one tool for every benchmark in the project. It was
written before any benchmark existed, and it does not distinguish between the
different questions a benchmark is asked to answer.

In practice "benchmark" covers three unrelated activities. What distinguishes
them is **what the number is compared against**.

| Purpose | Question | Compared against | Hardware | Run when |
| --- | --- | --- | --- | --- |
| `hardware-reference` | How fast is it on machine X? | a named reference machine | must be pinned and recorded | on release, or when the reference machine changes |
| `regression-tracking` | Is it getting slower over a period of development, and where did the time go? | its own history across a window of commits | must be pinned and recorded | periodically over a range of commits |
| `approach-comparison` | Which of these implementations is faster? | sibling variants measured in the same run | irrelevant; the result is relative | on demand, while evaluating an implementation choice |

### Why one tool cannot serve all three

`criterion` compares a named benchmark against **its own previous run on the
same disk**. That is a temporal baseline, so it structurally fits
`regression-tracking` only.

- For `approach-comparison` it is the wrong tool. The variants must be
  measured **interleaved within one process**, so that machine drift (thermal
  throttling, frequency scaling, co-tenant load on a build machine) cancels
  out. Two `criterion` `bench_function`s run back to back reintroduce exactly
  the drift that interleaving was meant to remove.
- For `hardware-reference` it is the wrong tool. `criterion`'s output is a
  delta against *this* machine's own history. Such a number cannot be quoted
  as a capability statement for a different machine.

### Regression tracking is a series over a window of commits, not a per-commit gate

Three observations each rule out automatic gating:

1. **There is no meaningful numeric threshold.** Whether 0.5 %, 2 % or 12 %
   matters depends on the absolute frame budget and on the context of the
   change. A threshold encoded in CI would either cry wolf continuously or
   mask real regressions.
2. **A regression is frequently in the data, not the program.** A world built
   from 20 000 single asteroids spread thinly can cost more than 20 fields of
   1 000 asteroids, because the second case can be culled per field (20 + 1 000
   bodies examined) while the first cannot (20 000). The same code is faster
   under one scenario and slower under another. A benchmark therefore always
   measures **code plus scenario**, and the scenario has to be named so the
   two are never confused.
3. **Regressions are frequently cumulative.** The offending work is often one
   of fifteen commits out of the last hundred and twenty, each contributing a
   little to a feature. There is no single commit to blame.

`regression-tracking` is therefore produced by running the benchmark at each
commit in a window (for example the last 120 commits, or the last three
weeks), recording one scalar per run together with commit, machine and
scenario id, and plotting the result as a line chart. The chart is read by a
human or an agent, who then investigates where the additional time came from.

Two consequences follow. First, **a benchmark result never blocks a merge.**
Second, `criterion`'s automatic verdict ("change: +2.3 %, p < 0.05, Performance
has regressed") is precisely the mechanical judgement described in point 1
above, so it must not be used as a gate.

## Decision

### 1. Every benchmark file declares its purpose

Each file that contains benchmarks carries a fixed, greppable declaration
block at the top of the file, in its module documentation:

```rust
//! BENCHMARK-PURPOSE: approach-comparison
//! BENCHMARK-BASELINE: sibling-variants-same-run
//! BENCHMARK-SCENARIO: main-belt-2p1-au-to-3p3-au
//! BENCHMARK-RUN: cargo test -p delta-v-physics --features bench -- --ignored --nocapture
```

### 2. Closed vocabulary

- `BENCHMARK-PURPOSE` is one of `hardware-reference`, `regression-tracking`,
  `approach-comparison`.
- `BENCHMARK-BASELINE` is fixed by the purpose, and a declaration whose
  baseline does not match its purpose is invalid:

  | Purpose | Required baseline |
  | --- | --- |
  | `hardware-reference` | `named-machine` |
  | `regression-tracking` | `own-history` |
  | `approach-comparison` | `sibling-variants-same-run` |

- `BENCHMARK-SCENARIO` is a stable identifier for the data set under
  measurement. A scenario change is a performance change, so it must be
  visible in the declaration and must not be renamed casually: renaming
  breaks continuity of a recorded series.

### 3. The path encodes the purpose

Benchmark files live in a per-purpose directory under the crate, so that the
purpose is visible in the path and each purpose can hold as many benchmark
files as it needs:

`crates/<crate>/src/<purpose>/<subject>_benchmarks_tests.rs`

| `BENCHMARK-PURPOSE` | Directory |
| --- | --- |
| `approach-comparison` | `approach_comparison/` |
| `regression-tracking` | `regression_tracking/` |
| `hardware-reference` | `hardware_reference/` |

The directory name is the snake_case form of `BENCHMARK-PURPOSE`
(ADR-0023 requires snake_case names), and it MUST match the purpose declared
by every file inside it. The file name then names the *subject* being
measured — `collision_benchmarks_tests.rs`, `gravity_benchmarks_tests.rs` —
and deliberately does not repeat the purpose, because the directory already
carries it.

Each purpose directory has a `mod.rs` that wires its files, and the crate root
declares that directory module behind the `bench` feature (see the next
section). No `benches/` target is used: a benchmark stays an ordinary gated
module, which is what lets it reach crate-private systems — the reason
`approach-comparison` cannot be a separate bench binary.

This is a deliberate exception to ADR-0021's flat sibling `_tests.rs` layout:
benchmark files are grouped one level down so a purpose can accumulate one
file per subject. The `_tests.rs` suffix, the `#![cfg(test)]` header and the
"never mixed with production code in the same file" rule all still apply to
every file in the directory.

### 4. All benchmarks are opt-in

Benchmarks are compiled only under a `bench` cargo feature and are therefore
excluded from day-to-day `cargo test --workspace`. This holds for every
purpose and is inherited unchanged from ADR-0021.

### 5. Tool policy per purpose

- **`approach-comparison`** — hand-rolled harness, living in a `#[cfg(test)]`
  module inside the crate (ADR-0021 `_tests.rs` layout), because it may need
  crate-private items. Variants are measured interleaved in one process and in
  one run. `criterion` MUST NOT be used.
- **`regression-tracking`** — the harness MUST emit **one stable scalar per
  run** for a pinned scenario, so a series can be plotted without
  post-processing, and MUST record commit, machine and scenario id. `criterion`
  MAY be used for result retention; its pass/fail verdict MUST NOT gate
  anything.
- **`hardware-reference`** — the harness MUST record CPU model, core count,
  build profile and commit alongside the number, because the number is
  meaningless without them.

### 6. No benchmark result is a merge gate

No numeric regression threshold is binding in this project. Interpretation of
benchmark output is by a human or an agent. CI does not fail a build on a
benchmark result.

### 7. Carried forward from ADR-0021 unchanged

- Four test layers — unit tests, integration tests in `tests/` with fixtures,
  headless smoke tests, doctests — all run by `cargo test --workspace`.
- Unit tests live in sibling `_tests.rs` files, start with `#![cfg(test)]`,
  and are never mixed with production code in the same file.
- Tests do not depend on execution order or on shared mutable global state.
- Every public function with a non-trivial contract carries a rustdoc example
  that runs as a doctest.

## Consequences

Positive:

- The `--features bench` gate now covers all three purposes, so no benchmark
  can accidentally slow down day-to-day `cargo test`.
- `criterion` is no longer mandated where it does not fit, so the existing
  approach-comparison benchmark in
  `crates/delta-v-physics/src/approach_comparison/performance_benchmarks_tests.rs` no longer
  violates an Accepted ADR, and nothing has to move to a `benches/` target
  (which would have forced two crate-private systems to become `pub`).
- Naming the scenario makes "the data changed, not the code" visible in the
  declaration rather than buried in the harness.
- The header is greppable, so the set of benchmarks and their purposes is one
  command away: `grep -rn "BENCHMARK-PURPOSE:" crates/`.

Negative:

- Tooling is now per purpose instead of one tool for everything, which is more
  to remember. The mandatory header is the mitigation.
- `regression-tracking` requires running the benchmark across a range of
  commits, which is slow and needs a dedicated, quiet machine. That
  infrastructure does not exist yet.
- Scenario ids are only useful if they are stable; a careless rename silently
  breaks a recorded series.
- Because the purpose is encoded in the directory, changing a benchmark's
  purpose means moving it. That is deliberate friction: the three purposes
  need different baselines and different tooling, so a silent change of
  purpose would invalidate the comparison.

Follow-up:

- `scripts/check-forbidden-patterns.sh` enforces the declaration block, the
  closed vocabulary, the purpose-to-baseline mapping, the purpose-prefixed
  file name, the `bench` feature gate and the `criterion` ban. It runs from
  `.githooks/pre-commit` and CI like the ADR-0013 checks, so the rules above
  fail a build rather than relying on review.
- `hardware-reference` and `regression-tracking` benchmarks do not exist yet.
  The vocabulary is defined now so they can slot in without another ADR.
- `docs/architecture.md` gains a "Performance benchmarks" section in the same
  change, which also discharges the outstanding consequence of ADR-0021.

## Notes

- The only benchmark in the repository today is
  `crates/delta-v-physics/src/approach_comparison/performance_benchmarks_tests.rs`, declared as
  `approach-comparison`. Its four benchmarks (scaling, solar system, gravity,
  collision) all answer "how does cost grow, and is this within budget",
  which is a capacity question against sibling configurations measured in one
  run.
- [ADR-0022](0022-performance-instrumentation.md) governs runtime
  instrumentation (frame-time warnings, spans). That is a different mechanism
  operating inside a running game and is unaffected by this ADR.
