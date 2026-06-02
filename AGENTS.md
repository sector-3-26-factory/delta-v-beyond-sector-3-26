# AGENTS.md

This file is the canonical entry point for **AI agents** working in this
repository. It collects the rules they must respect when modifying code,
configuration or documentation.

> **If you are an AI agent reading this**: read this file completely
> before producing any change. Then read
> [`docs/adr/ARCHITECTURAL_RULES.md`](docs/adr/ARCHITECTURAL_RULES.md) —
> the condensed, machine-readable ruleset derived from all ADRs.
> Every rule there is binding. You do not decide which rules are relevant.
> **ALL rules are always relevant.**

Scope: this file is written for AI agents. The project accepts
external human contributions only in narrowly defined areas (notably
translations); the rules and exceptions are described in
`CONTRIBUTING.md` and governed by ADR-0036 (see
[`docs/adr/ARCHITECTURAL_RULES.md`](docs/adr/ARCHITECTURAL_RULES.md)).

---

## 1. Architectural rules are binding

All architectural decisions for this project are condensed into strict,
machine-readable rules in
[`docs/adr/ARCHITECTURAL_RULES.md`](docs/adr/ARCHITECTURAL_RULES.md).
That file is the **mandatory read** before any change. It is derived from
the full ADR set in [`docs/adr/`](docs/adr/); the full ADRs remain the
authoritative source for rationale and history.

Rules:

1. **READ [`docs/adr/ARCHITECTURAL_RULES.md`](docs/adr/ARCHITECTURAL_RULES.md)
   in full** before producing any change. Every rule there applies to every
   change. You do not filter, you do not prioritize, you do not skip.
2. **Honor every rule derived from an `Accepted` ADR.** If a change requires
   violating an Accepted ADR, the ADR must be superseded by a new ADR
   *first*, in a separate commit / pull request, before the change is made.
3. **Rules derived from `Proposed` ADRs** describe future intent; they are
   informational but not yet enforceable. New code should still align with
   their direction whenever practical.
4. **Never delete or rewrite an Accepted ADR.** Supersede it instead by
   adding a new ADR that references the old one and updating the old one's
   status to `Superseded by ADR-NNNN`. Update `ARCHITECTURAL_RULES.md`
   in the same commit.
5. **When you make a non-trivial architectural choice not covered by an
   existing ADR, create a new ADR** in status `Proposed` and add the
   corresponding rule to `ARCHITECTURAL_RULES.md`. Do not bury hidden
   decisions in code.

The ADR template lives in [`docs/adr/0000-template.md`](docs/adr/0000-template.md).
The ADR process is described in the full ADR-0001 at
[`docs/adr/0001-adr-process.md`](docs/adr/0001-adr-process.md).

## 2. Every source file points here

Every source file we author (Rust, JSON schema, shell scripts, build
scripts, ...) that supports comments contains a one-line header
instructing agents to read this file. The header does **not** list
ADRs; it only points at `AGENTS.md`, which points at `ARCHITECTURAL_RULES.md`.
The exact rule and per-language examples are in ADR-0033 (see
[`docs/adr/ARCHITECTURAL_RULES.md`](docs/adr/ARCHITECTURAL_RULES.md)).

The chain is intentional:

```
source file header  ->  AGENTS.md  ->  docs/adr/ARCHITECTURAL_RULES.md
```

This indirection keeps source files stable when ADRs are added or
renumbered.

## 3. Workflow rules (summary)

Full rules are in [`docs/workflow.md`](docs/workflow.md). The minimum an
agent must know:

- **Branching**: Gitflow-light. `main` = stable releases. `dev` = active
  integration. Feature work happens on `feature/<slug>` branches off
  `dev` and is merged back into `dev` via pull request. See
  [ADR-0003](docs/adr/ARCHITECTURAL_RULES.md).
- **Commits**: Conventional Commits format. See
  [ADR-0004](docs/adr/ARCHITECTURAL_RULES.md).
- **No warnings**: A build that emits any warning must not be committed.
  See [ADR-0034](docs/adr/ARCHITECTURAL_RULES.md).
- **No silent fallbacks**: Missing or invalid configuration is a hard
  error. See [ADR-0013](docs/adr/ARCHITECTURAL_RULES.md).

## 4. Coding rules (summary)

- **Language**: All code, comments, commit messages and documentation are
  written in **English**.
- **Formatting**: Run `cargo fmt --all` before committing.
- **Linting**: `cargo clippy --workspace --all-targets -- -D warnings`
  must pass.
- **Tests**: `cargo test --workspace` must pass.
- **No `unsafe`** without an ADR or an inline comment that justifies it
  and is approved on review. See
  [ADR-0028](docs/adr/ARCHITECTURAL_RULES.md) for the
  related dependency rule.
- **JSON defaults in schema only** (ADR-0012, ADR-0013, ADR-0040 **strictly enforced**):
  - ❌ No `#[serde(default = "...")]` on JSON-backed struct fields
  - ❌ No custom `fn default_X()` functions
  - ❌ No `Option<T>` for fields that always exist after schema validation
  - ✅ All defaults in `*.schema.json` only
  - ✅ Use the `delta-v-json` crate for all JSON loading (see [ADR-0040](docs/adr/ARCHITECTURAL_RULES.md))
  - See [ADR-0039](docs/adr/ARCHITECTURAL_RULES.md) for
    code review checklist and rationale. Violations are code review failures.

## 5. MANDATORY SEQUENCE: Read ARCHITECTURAL_RULES.md before doing anything else

**NON-NEGOTIABLE: This sequence must be followed in order before you take any action.**

Before reading task descriptions, examining code, running commands, or using any tool, you MUST:

1. **IMMEDIATELY read [`docs/adr/ARCHITECTURAL_RULES.md`](docs/adr/ARCHITECTURAL_RULES.md)**
   in full. Every rule there applies to every change.
2. **ONLY AFTER you have read `ARCHITECTURAL_RULES.md` in full** may you:
   - Read task descriptions
   - Examine project files
   - Run any terminal command
   - Use any tool
   - Respond to the user

**VIOLATION:** If you attempt to do any work before reading `ARCHITECTURAL_RULES.md` in full,
you have violated this binding constraint. There is no exception, no context where this is
acceptable.

**VERIFICATION REQUIRED:** After reading `ARCHITECTURAL_RULES.md`, you MUST explicitly
state in your response which ADRs you have read, listing every ADR number referenced
in the file. For example:

```
ARCHITECTURAL_RULES.md read (ADR-0001 through ADR-0042).
```

If additional ADRs exist (e.g. ADR-0043, ADR-0044, …), you **must** list the full
actual range/coverage – do not hard-code a number. The user must be able to verify
that you read every rule.

Then STOP and WAIT for the user's confirmation before proceeding to any other task.

---

## 6. What to read after ADRs are complete

After you have verified that all ADRs are read, proceed with:

1. [`docs/workflow.md`](docs/workflow.md) -- branching, PRs, commits, CI.
2. [`docs/architecture.md`](docs/architecture.md) -- the big-picture
   architecture, which references the ADRs.
3. [`docs/roadmap.md`](docs/roadmap.md) -- the milestone roadmap.

When in doubt, ask in a pull request comment before writing code.

## 7. MANDATORY: Agents NEVER commit changes

**ABSOLUTE RULE: Under no circumstances may an agent execute `git commit`, `git push`, `git merge`, or any other git command that modifies the repository history or branches.**

Git operations are reserved exclusively for human developers. This includes:

- ❌ `git commit` (any variant)
- ❌ `git push`
- ❌ `git merge`
- ❌ `git rebase`
- ❌ `git cherry-pick`
- ❌ `git tag`
- ❌ `git branch -d` (or any branch deletion)
- ❌ `git reset` (any form of history rewriting)
- ❌ Any other git command that modifies history or branches

**The only git operations an agent may perform are read-only:**

- ✅ `git log` (viewing history)
- ✅ `git diff` (viewing changes)
- ✅ `git status` (checking state)
- ✅ `git show` (viewing commits)
- ✅ `git branch -l` (listing branches)

**After making code changes:**

1. Use the edit tools (`edit_existing_file`, `create_new_file`, etc.) to modify files.
2. The user will see the changes in their editor.
3. The user will review the changes.
4. The user will commit and push when satisfied.

**Why this rule exists:**

- Only humans decide which changes go into the repository and when.
- Only humans write commit messages and are accountable for what they claim in the message.
- Only humans decide on branching strategy and integration timing.
- Accidental agent commits can corrupt the repository state and blame history.

**VIOLATION:** If an agent attempts to run any git write command, it has violated this binding constraint. There is no exception, no context where this is acceptable.
