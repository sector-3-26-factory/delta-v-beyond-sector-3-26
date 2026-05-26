# ADR-0001: ADR process

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

This project will accumulate a non-trivial number of architectural
decisions: workspace layout, physics units, networking model, asset
pipeline, configuration system, and many more. Without a written
record, these decisions are forgotten, re-litigated, or silently
violated, and reviewers (human or AI) cannot tell whether a piece of
code reflects an intentional choice or an accident.

We need a lightweight, durable format that:

- captures one decision per document,
- explains *why*, not only *what*,
- can evolve over time without rewriting history,
- is easy to enforce in code review and automated checks.

## Decision

We adopt **Architecture Decision Records (ADRs)** in the style described
by Michael Nygard's original write-up.

Rules:

1. Each ADR lives in `docs/adr/NNNN-kebab-case-title.md`. `NNNN` is a
   monotonically increasing four-digit number; numbers are never
   reused.
2. The template lives in [`docs/adr/0000-template.md`](0000-template.md)
   and must be used for every new ADR.
3. Every ADR has the sections: Status, Date, Deciders, Context,
   Decision, Consequences. Other sections (Notes, Alternatives) are
   optional.
4. Allowed statuses are: `Proposed`, `Accepted`, `Deprecated`,
   `Superseded by ADR-NNNN`.
5. **Accepted ADRs are immutable in content.** Typos and link fixes are
   the only edits allowed. Any change of decision is done by writing a
   new ADR that supersedes the old one. The old ADR's status is then
   changed to `Superseded by ADR-NNNN`, and that is the only
   semantically meaningful edit it ever receives after acceptance.
6. The index in [`docs/adr/README.md`](README.md) lists every ADR with
   its status. New ADRs add a row in the same pull request.
7. ADRs are referenced from [`AGENTS.md`](../../AGENTS.md) and from
   source code through it; never from source files directly. See
   [ADR-0033](0033-agents-md-and-source-file-pointers.md).

## Consequences

Positive:

- Every non-trivial decision has a single, citable home.
- New contributors (human or AI) have a documented project memory.
- Reviewers can point at an ADR instead of re-explaining rationale.
- Decisions that turned out wrong remain visible, with their reasoning,
  so the same mistake is not repeated.

Negative:

- Overhead: every architectural decision requires writing a document.
- Risk of bureaucracy if used for trivial decisions. We avoid this by
  not requiring ADRs for purely local refactors or single-file
  implementation details.

Follow-up:

- An ADR linter or CI check could verify that every file in `docs/adr/`
  has the required sections and a valid status. Optional, deferred.
