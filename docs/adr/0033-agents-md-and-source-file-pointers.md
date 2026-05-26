# ADR-0033: AGENTS.md and source file pointers

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

AI agents working on this repository need a single, reliable entry
point that tells them which rules apply. Without one, agents
reinvent conventions, miss ADRs, or simply do not know they exist
and head in a direction that contradicts decisions the project has
already made.

A second concern is that ADRs are numerous and will keep growing.
We do not want every source file to maintain a list of "the ADRs
that apply to me" -- such lists go stale the day after an ADR is
added or renumbered.

This ADR is scoped to AI agents. Onboarding for human contributors
is a separate concern and is currently out of scope (the project is
not accepting external human contributors); see
[ADR-0036](0036-external-human-contributors-process.md) for the
placeholder process that would apply if and when that changes.

## Decision

**Two layers of indirection**, both aimed at AI agents:

1. The repository root contains [`AGENTS.md`](../../AGENTS.md). It
   is the canonical entry point. It instructs the agent to consult
   the ADR index before doing non-trivial work, and summarises the
   most important rules.

2. **Every source file we author** carries a short header comment
   pointing at `AGENTS.md`. The header does **not** name individual
   ADRs and does **not** repeat rules. It only contains the
   pointer.

   ```
   source file header  ->  AGENTS.md  ->  docs/adr/
   ```

   This indirection keeps source files stable when ADRs are added,
   renamed, or renumbered.

**Required header content**: the single sentence instructing the
agent to read `AGENTS.md` before modifying the file. Nothing else.
Licence information, copyright notices and similar concerns live in
`LICENSE` at the repository root, not in per-file headers.

Examples (language-appropriate comment syntax):

Rust / C-style:

```rust
// AGENTS: before modifying this file, read AGENTS.md at the
// repository root.
```

Shell / Dockerfile / YAML / TOML:

```sh
# AGENTS: before modifying this file, read AGENTS.md at the
# repository root.
```

JSON Schema (JSON has no comments; we use the schema's own
`description` at the top level):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "...",
  "title": "...",
  "description": "AGENTS: before modifying this schema, read AGENTS.md at the repository root.\n\n<actual description follows>"
}
```

Plain JSON content files (without a `description` slot) are exempt;
they are data, and an agent that touches them is expected to have
read the corresponding schema, which carries the pointer.

**Scope**: all source files we author and that support comments.
Exempt:

- The `LICENSE` file itself.
- Third-party assets and vendored code.
- Auto-generated artefacts (`Cargo.lock`, generated bindings,
  etc.).
- Plain JSON content files that do not have a `description` field.
- Documentation files under `docs/` and the top-level `README.md`,
  `CONTRIBUTING.md`, `AGENTS.md` themselves -- they are the
  documentation an agent is being pointed at; pointing at oneself
  is noise.

**Enforcement**: a CI check (deferred, optional) can grep for the
`AGENTS:` marker in every tracked source file that is in scope.
Until then, the rule is enforced on review.

## Consequences

Positive:

- An AI agent opening any source file immediately learns where the
  rules live.
- Source files do not need to be touched when an ADR is added or
  renamed.
- The pointer is the same single sentence in every language; no
  per-file template to maintain.
- Per-file headers stay small and do not duplicate licence or ADR
  text that lives elsewhere.

Negative:

- Every new in-scope file needs a one-line header. Trivially small
  boilerplate.
- An agent that ignores the header still does so; the convention
  helps the careful agent, not the careless one.

Follow-up:

- A short script that inserts the appropriate header into a freshly
  created source file is a future convenience; not blocking.
- The optional CI check for the `AGENTS:` marker can be added
  when the project grows enough to warrant it.
- If and when external human contributors are accepted, the
  separate process described in
  [ADR-0036](0036-external-human-contributors-process.md) takes
  effect. That ADR may or may not require a parallel pointer for
  human contributors; this ADR is unaffected.
