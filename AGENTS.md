# AGENTS.md

This file is the canonical entry point for **AI agents** working in this
repository. It collects the rules they must respect when modifying code,
configuration or documentation.

> **If you are an AI agent reading this**: read this file completely
> before producing any change. Then read every Architecture Decision
> Record under `docs/adr/` before making decisions that touch the
> corresponding area. Never silently ignore an ADR.

Scope: this file is written for AI agents. The project accepts
external human contributions only in narrowly defined areas (notably
translations); the rules and exceptions are described in
`CONTRIBUTING.md` and governed by
[ADR-0036](docs/adr/0036-external-human-contributors-process.md).

---

## 1. Architecture Decision Records (ADRs) are binding

All architectural decisions for this project are recorded as ADRs in
[`docs/adr/`](docs/adr/). They are not suggestions; they are binding
constraints on every change.

Rules:

1. **Read the index** at [`docs/adr/README.md`](docs/adr/README.md) before
   starting work that might touch architecture, conventions, tooling,
   networking, physics, configuration, assets, or any cross-cutting
   concern.
2. **Honor every ADR in status `Accepted`.** If a change requires
   violating an Accepted ADR, the ADR must be superseded by a new ADR
   *first*, in a separate commit / pull request, before the change is
   made.
3. **ADRs in status `Proposed`** describe future intent; they are
   informational but not yet enforceable. New code should still align
   with their direction whenever practical.
4. **Never delete or rewrite an Accepted ADR.** Supersede it instead by
   adding a new ADR that references the old one and updating the old
   one's status to `Superseded by ADR-NNNN`.
5. **When you make a non-trivial architectural choice that is not covered
   by an existing ADR, create a new ADR** in status `Proposed` and
   reference it from your code/PR. Do not bury hidden decisions in code.

The ADR template lives in [`docs/adr/0000-template.md`](docs/adr/0000-template.md).
The ADR process itself is described in
[ADR-0001](docs/adr/0001-adr-process.md).

## 2. Every source file points here

Every source file we author (Rust, JSON schema, shell scripts, build
scripts, ...) that supports comments contains a one-line header
instructing agents to read this file. The header does **not** list
ADRs; it only points at `AGENTS.md`, which points at the ADR index.
The exact rule and per-language examples are in
[ADR-0033](docs/adr/0033-agents-md-and-source-file-pointers.md).

The chain is intentional:

```
source file header  ->  AGENTS.md  ->  docs/adr/
```

This indirection keeps source files stable when ADRs are added or
renumbered.

## 3. Workflow rules (summary)

Full rules are in [`docs/workflow.md`](docs/workflow.md). The minimum an
agent must know:

- **Branching**: Gitflow-light. `main` = stable releases. `dev` = active
  integration. Feature work happens on `feature/<slug>` branches off
  `dev` and is merged back into `dev` via pull request. See
  [ADR-0003](docs/adr/0003-branching-and-pr-workflow.md).
- **Commits**: Conventional Commits format. See
  [ADR-0004](docs/adr/0004-commit-message-convention.md).
- **No warnings**: A build that emits any warning must not be committed.
  See [ADR-0034](docs/adr/0034-no-warnings-policy.md).
- **No silent fallbacks**: Missing or invalid configuration is a hard
  error. See [ADR-0013](docs/adr/0013-no-silent-fallbacks.md).

## 4. Coding rules (summary)

- **Language**: All code, comments, commit messages and documentation are
  written in **English**.
- **Formatting**: Run `cargo fmt --all` before committing.
- **Linting**: `cargo clippy --workspace --all-targets -- -D warnings`
  must pass.
- **Tests**: `cargo test --workspace` must pass.
- **No `unsafe`** without an ADR or an inline comment that justifies it
  and is approved on review. See
  [ADR-0028](docs/adr/0028-third-party-dependency-policy.md) for the
  related dependency rule.

## 5. What to read next

In order:

1. [`docs/adr/README.md`](docs/adr/README.md) -- index of all ADRs.
2. [`docs/workflow.md`](docs/workflow.md) -- branching, PRs, commits, CI.
3. [`docs/architecture.md`](docs/architecture.md) -- the big-picture
   architecture, which references the ADRs.
4. [`docs/roadmap.md`](docs/roadmap.md) -- the milestone roadmap.
5. The ADR(s) closest to the area you are about to touch.

When in doubt, ask in a pull request comment before writing code.
