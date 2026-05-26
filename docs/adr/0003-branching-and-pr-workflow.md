# ADR-0003: Branching and PR workflow

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

A small team (one human + AI agents) still benefits from a predictable
branching model: it makes CI rules unambiguous, prevents accidental
direct pushes to release branches, and gives a clear path for
multi-step feature work.

Full Gitflow is overkill at this scale (no `release/*`, `hotfix/*`,
`support/*` branches). A trivial trunk-based model has no integration
branch to stabilise work-in-progress between merges.

## Decision

We use **"Gitflow-light"**:

- `main`: long-lived, stable, only release-ready code. Tagged for
  releases per [ADR-0025](0025-versioning.md). Strictly protected.
- `dev`: long-lived integration branch. Default branch on GitHub.
  Receives all feature merges. Protected (no force-push, no deletion,
  CI must pass), but direct commits by the maintainer are allowed for
  small fixes.
- `feature/<slug>`: short-lived feature branches off `dev`. Merged
  back into `dev` via pull request. Unprotected; force-push and
  rebase are fine here.

PR rules:

- PRs target `dev` unless they are an explicit `dev -> main` release
  PR.
- PRs must pass CI (`fmt`, `clippy`, `check`, `test`, `cargo-deny`).
- PRs into `main` require linear history and a passing CI; approval
  count is set to 0 while the project has a single maintainer.
- Squash-merge or rebase-and-merge into `dev` are both allowed; merges
  into `main` are linear (per the ruleset).
- Branch names are lowercase, kebab-case, prefixed by intent:
  `feature/`, `fix/`, `chore/`, `docs/`, `refactor/`.

## Consequences

Positive:

- `main` is always shippable.
- `dev` collects incremental work; a broken `dev` is recoverable
  without touching `main`.
- Feature branches keep large changes reviewable in isolation.

Negative:

- Two long-lived branches mean occasional `dev -> main` merge PRs that
  carry many commits. Manageable because `main` is linear.

Follow-up:

- The branch protection / ruleset configuration on GitHub mirrors this
  ADR. See [`docs/workflow.md`](../workflow.md) for the concrete
  GitHub settings.
