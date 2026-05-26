# ADR-0024: Documentation policy

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

The project will, for the foreseeable future, be developed by one
human plus AI agents. Both audiences read code, but documentation
serves different purposes for each:

- For the human author: a reminder of *why* a particular design was
  chosen, six months after writing it.
- For AI agents: a contract that lets them generate code that fits
  the existing project without inventing new patterns from scratch.

The AI audience usually reads the code itself faster than it reads
prose, so we do not need exhaustive narrative documentation. We *do*
need every non-obvious decision to be findable.

## Decision

**Layered documentation**:

1. **Architectural intent** lives in ADRs, indexed from
   [`docs/adr/README.md`](README.md). Pointed at from
   [`AGENTS.md`](../../AGENTS.md).
2. **Cross-cutting overviews** live in `docs/`:
   - `architecture.md`: module map, plugin composition, data flow.
   - `workflow.md`: branching, PRs, CI, commit conventions.
   - `roadmap.md`: milestones.
   - `physics.md`: physics conventions and formulas.
3. **API documentation** lives in rustdoc, in the source itself.
   Required when intent is not obvious from the signature alone.
4. **In-file context comments** explain *why* a piece of code looks
   the way it does when the reason is non-local (e.g. a workaround
   for a Bevy quirk, a constant tuned by experiment).

**Mandatory rustdoc**:

- Every `pub` item (module, type, function, trait, macro) of every
  library crate has at least a one-line doc comment.
- Public functions whose contract is non-trivial carry a
  documented example. Examples are run by `cargo test` as doctests
  (per [ADR-0021](0021-testing-strategy.md)).
- Crate root (`lib.rs`) of each library crate has a short summary
  paragraph and, where helpful, a brief example of registering its
  plugin.
- The lint `#![warn(missing_docs)]` is enabled in every library
  crate (per [ADR-0023](0023-code-style-and-lints.md)) so missing
  docs surface immediately.

**Optional rustdoc**:

- Private items are documented when the intent is not obvious from
  the code. Quick-and-dirty internals get a one-line comment, not
  a full rustdoc block.

**JSON schema documentation**:

- Every field in every JSON Schema carries a `description` so the
  schema doubles as documentation for content authors (per
  [ADR-0012](0012-json-schema-validation.md)).

**What we do not require**:

- Tutorials, narrative chapters or "getting started" guides at this
  stage. They become relevant once external contributors are
  expected, which is not the case for the immediate future.
- A separate user manual; the schemas and the in-game UI (later)
  serve that role.

## Consequences

Positive:

- Future-author and AI agents both have a way to recover the *why*
  behind any decision.
- The build will refuse to ship a public API item without a doc
  comment.
- Schemas, ADRs and code references one another, forming a small
  but coherent corpus.

Negative:

- Writing docs is friction. We mitigate by accepting one-line docs
  on items that are genuinely self-explanatory.
- The `#![warn(missing_docs)]` rule combined with "no warnings"
  ([ADR-0034](0034-no-warnings-policy.md)) means a forgotten doc
  blocks a commit. This is intentional.

Follow-up:

- A short style example in `docs/architecture.md` shows the
  preferred shape of rustdoc for a plugin, a component and a
  system.
