# Workflow

This document describes the day-to-day workflow for working on
*Delta-V beyond Sector 3.26*. It is a reference, not a tutorial; for
the *why* behind any rule below, follow the ADR link.

If you have not read [`AGENTS.md`](../AGENTS.md) yet, read it first.

## Branching

We use a Gitflow-light model (see
[ADR-0003](adr/0003-branching-and-pr-workflow.md)):

- `main` -- stable, release-tagged.
- `dev` -- default branch, integration of all work.
- `feature/<slug>` -- short-lived branches off `dev`.
  Allowed prefixes: `feature/`, `fix/`, `chore/`, `docs/`,
  `refactor/`.

Branch naming: lowercase, kebab-case, intent-prefixed.

```bash
git checkout dev
git pull --ff-only
git checkout -b feature/my-thing
# ... work, commit ...
git push -u origin feature/my-thing
# open a PR to dev on GitHub
```

## Commit messages

Conventional Commits (see
[ADR-0004](adr/0004-commit-message-convention.md)):

```
<type>(<optional scope>): <imperative summary>

<optional body explaining *why*>

<optional footer(s)>
```

Allowed types: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`,
`build`, `ci`, `chore`, `style`, `revert`.

Scope, when used, is the crate name without the `delta-v-` prefix
(`ships`, `physics`, ...) or a top-level concern (`adr`, `ci`,
`docs`, `devcontainer`).

Breaking changes: `feat(api)!: ...` or a `BREAKING CHANGE:` footer.

## Pre-commit checklist

Before every commit:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If any of these complains, fix it; do not commit warnings (see
[ADR-0034](adr/0034-no-warnings-policy.md)).

An optional Git hook in `scripts/git-hooks/` automates this; install
it once per clone with:

```bash
./scripts/install-git-hooks.sh
```

(The script is added in a follow-up commit; until then, run the
commands above manually.)

## Pull requests

PRs target `dev` by default. PRs from `dev` into `main` are release
PRs and require linear history.

Required CI checks (enforced by branch rulesets on GitHub):

- `Formatting (rustfmt)`
- `Clippy + cargo check`

Both `main` and `dev` are protected against:

- force-pushes,
- deletions,
- merging while CI is red.

`main` additionally requires:

- a pull request,
- linear history.

Feature branches are not protected; force-push and rebase freely.

## Adding a dependency

See [ADR-0028](adr/0028-third-party-dependency-policy.md). In short:

1. Confirm the licence is GPL-3.0-or-later-compatible.
2. Confirm the crate is actively maintained.
3. Pin the version in `[workspace.dependencies]` at the workspace
   root; inherit in member crates via `workspace = true`.
4. Run `cargo deny check`; it must pass.
5. Mention the dependency and its justification in the PR
   description.

## Adding an ADR

See [ADR-0001](adr/0001-adr-process.md). In short:

1. Copy [`adr/0000-template.md`](adr/0000-template.md) to
   `adr/NNNN-kebab-case-title.md` with the next free number.
2. Fill in Context, Decision, Consequences.
3. Start in status `Proposed`.
4. Open a PR; on merge, set status to `Accepted` if consensus is
   reached.
5. Add a row in `adr/README.md` in the same PR.

## Releasing

See [ADR-0025](adr/0025-versioning.md) and
[ADR-0026](adr/0026-release-process.md). Until the first release
exists there is nothing to do here.

## GitHub repository configuration (reference)

What we have configured on the GitHub side, for the record:

- **Default branch**: `dev`.
- **Rulesets**:
  - `main protection`: targets `main`. Restrict deletions, require
    linear history, require a pull request (0 required approvals),
    require status checks (`Formatting (rustfmt)`,
    `Clippy + cargo check`) with branches up to date, block force
    pushes.
  - `dev protection`: targets `dev`. Restrict deletions, require
    status checks (same two), block force pushes. No PR requirement
    -- the maintainer may commit directly for small changes.

If GitHub UI changes break the link to the rules page, navigate to
`Settings -> Rules` on the repository.
