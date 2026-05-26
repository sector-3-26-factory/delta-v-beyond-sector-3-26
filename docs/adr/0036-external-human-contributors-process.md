# ADR-0036: External human contributors process

- **Status**: Accepted (placeholder; activated only if external
  human contributors are admitted)
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

The project is currently maintained by a single human developer
working together with AI agents. There is no plan to accept external
human contributors in the near term. [`AGENTS.md`](../../AGENTS.md)
and [ADR-0033](0033-agents-md-and-source-file-pointers.md) are
scoped to AI agents only.

If the maintainer ever decides to accept external human
contributors, a separate onboarding process is needed. Writing it
up now -- as a deliberately small placeholder -- means the project
will not be caught flat-footed if the decision changes.

## Decision

**Today**: the project does not accept external human contributors
for **code or architecture changes**. Pull requests from external
humans that touch Rust sources, build configuration, CI, ADRs or
other architecture artefacts are not solicited and may be declined
without further explanation. There is no obligation to provide
review capacity for such PRs.

**Exception: translations.** Contributions to translation files
(`assets/i18n/<language>.json`, per
[ADR-0037](0037-internationalization.md)) are explicitly welcome
and reviewed on best-effort terms. They are accepted because:

- they are scoped to a single, well-defined file kind,
- their effect is contained: a translation file does not influence
  code paths beyond providing text,
- the schema and the typed Rust hierarchy that consume them are
  maintained by the core team, so an external translator never
  needs to touch Rust,
- accepting them broadens accessibility to non-English-speaking
  players, which is one of the project's goals.

The practical conditions for translation PRs are documented in
`CONTRIBUTING.md`. A translator who, while translating, spots a
missing or unclear key in the reference (`en.json`) is expected to
open an issue rather than to extend the Rust struct or schema.

Bug reports, feature suggestions and security reports are also
welcome via GitHub issues; opening an issue is not a code
contribution and is not affected by the closed-PR policy above.

**If and when the project also opens up to external code
contributions** (beyond translations), the following minimum
process is adopted in the same PR that makes that change:

1. **CONTRIBUTING.md** is extended to be the canonical entry point
   for human code contributors. It must:
   - state that the project is now open to external code
     contributions and under what conditions,
   - link to the ADR index ([`docs/adr/README.md`](README.md)) and
     instruct new contributors to read at least
     [ADR-0001](0001-adr-process.md),
     [ADR-0003](0003-branching-and-pr-workflow.md),
     [ADR-0004](0004-commit-message-convention.md) and
     [ADR-0034](0034-no-warnings-policy.md) before opening their
     first PR,
   - restate the CLA-equivalent statement from the current
     `CONTRIBUTING.md` (contributions are licensed under
     GPL-3.0-or-later; see
     [ADR-0027](0027-open-source-licensing.md)),
   - describe how to set up the dev container, run the
     pre-commit checks and open a pull request,
   - describe the review process and the maintainer's expected
     response time (which may be "best effort, no guarantee").
2. **AGENTS.md remains scoped to AI agents.** Human contributors
   are pointed at `CONTRIBUTING.md`, not at `AGENTS.md`. The two
   documents may share content by linking, not by duplication.
3. **Per-file headers**: this ADR does **not** require human-facing
   per-file headers. If experience shows they are useful, a
   separate ADR adds them. Until then, `CONTRIBUTING.md` is the
   single human-facing onboarding document.
4. **Code of Conduct**: when external code contributions are
   opened, a `CODE_OF_CONDUCT.md` is added. The exact text is a
   separate decision (Contributor Covenant is a reasonable
   default).
5. **Security reporting**: a `SECURITY.md` is added describing how
   to report vulnerabilities responsibly. Until external
   contributors exist, security reports go to the maintainer
   directly through whatever channel is current.

Until this happens, this ADR's effect is that AI-agent-facing
decisions ([ADR-0033](0033-agents-md-and-source-file-pointers.md))
do not silently overreach into "rules for humans", and that the
translation exception is the single, well-scoped channel through
which external humans contribute to the repository today.

## Consequences

Positive:

- The boundary between "for AI agents" and "for humans" is
  explicit. Documentation aimed at one audience does not pretend
  to address the other.
- The maintainer can change course (open up contributions, or
  not) without having to rewrite the agent-facing layer.
- If contributions are opened, the minimum required documents are
  enumerated and unambiguous.

Negative:

- The placeholder nature of this ADR means it sits in `Accepted`
  with most of its content gated on a future event. Acceptable;
  the alternative is to leave the question unaddressed and run
  into it later.

Follow-up:

- None for the code-contribution side until the project decides to
  open up. At that point, this ADR is amended (or superseded) with
  the concrete documents listed above.
- For translations: `CONTRIBUTING.md` already documents the
  workflow; the schema (`i18n.schema.json`) and the reference
  English file (`assets/i18n/en.json`) are added together with the
  first concrete i18n work, at the latest during M6 (HUD).
