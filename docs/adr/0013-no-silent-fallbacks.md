# ADR-0013: No silent fallbacks

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Game engines and game configurations are full of opportunities to
"helpfully" recover from missing or wrong data: a missing keybinding
becomes `Space`, a missing ship mass becomes `1.0`, a missing texture
becomes pink. These helpful behaviours mask bugs and let broken
configurations ship.

Our project has decided that configuration defaults belong in JSON
Schemas, not in Rust constants
([ADR-0014](0014-engine-constants-vs-gameplay-values.md)), and that
JSON Schemas are mandatory and strictly enforced
([ADR-0012](0012-json-schema-validation.md)). For this to mean
anything, missing or invalid data must actually stop the game.

## Decision

The game **never silently substitutes a fallback value** for missing
or invalid configuration or content. Specifically:

- A missing **required** field in a JSON file is an error.
- A field that fails schema validation is an error.
- A defaults file (anything under `assets/config/`) that is missing
  or invalid prevents application startup.
- A user override file
  (`$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/...`) that fails
  validation prevents application startup with a clear error
  pointing at the offending file and field. The user is not
  silently downgraded to defaults; instead they are told what to
  fix.
- Asset loaders do not substitute placeholder meshes or textures in
  release builds. In `--features dev` builds, a clearly recognisable
  placeholder (e.g. magenta material) may be substituted **and** a
  `warn!` is logged. The dev placeholder must look obviously wrong.
- Missing user override files are not errors; they simply mean "no
  overrides".

Programmer mistakes (missing match arm, unreachable branch reached)
are handled with `panic!` or `unreachable!`, not with silent
defaults. See [ADR-0016](0016-error-handling-strategy.md).

This rule applies to **content and configuration**, not to optional
features like crash reporting, telemetry or update checks, which may
fail gracefully.

### Schema defaults are part of the contract, not a fallback

A field that is declared in the schema with a `"default"` value
**may** be omitted in JSON. The loader fills it in with the schema
default. This is **not** a silent fallback; it is the documented
behaviour of the schema, equivalent to writing the default value
explicitly into the file.

The loader applies schema defaults to validated input as a separate,
explicit step:

1. Parse the JSON.
2. Validate against the schema. Reject if validation fails (per the
   rules above).
3. **Walk the schema and the parsed value in parallel, recursively,
   inserting any `"default"` from the schema for every field that
   the JSON omitted.** This applies at every nesting level: a
   defaulted field may itself be an object whose own fields have
   defaults, and so on. The recursion is bounded by the schema's
   own structure.
4. Re-validate the result (cheap; catches bugs in the default-
   filling logic itself).
5. Deserialise the completed value into Rust structs.

`serde` is not aware of JSON Schema and does not perform step 3 on
its own. The default-filling pass is therefore part of our loader,
not of `serde_json`. It is the same pass for every schema; there is
no per-type machinery to keep in sync.

What counts as "omitted" is exactly what JSON itself defines:
- A property that is not present in an object is omitted.
- A property whose value is JSON `null` is **present**, not omitted,
  and is only accepted if the schema permits `null` for that field.
- An empty array or empty object is **present**, not omitted.

What is **not** acceptable:

- A Rust struct that silently substitutes `Default::default()` for a
  missing field. The default must be reachable from the schema, or
  the field must be marked required in the schema and the load must
  fail.
- A loader that adds a default which is not in the schema. The
  schema is the single source of truth
  (per [ADR-0014](0014-engine-constants-vs-gameplay-values.md)).
- Cargo features or environment variables that quietly change which
  default applies. If the chosen default depends on a property the
  document itself carries (e.g. a `"type"` discriminator), the
  decision is modelled by **using a different schema file per
  variant** rather than by branch-dependent JSON-Schema
  combinators (`oneOf`, `if-then-else`, ...). The latter are
  forbidden by [ADR-0012](0012-json-schema-validation.md), rule 8,
  precisely because they would make the default-filling pass
  above ambiguous.
- Branch-dependent JSON-Schema combinators with differing
  `default` values. The default-filling pass walks the schema
  structurally; it does not evaluate `oneOf`/`anyOf`/
  `if-then-else` to decide which branch's defaults to use. The
  schema discipline (per
  [ADR-0012](0012-json-schema-validation.md), rule 8) is what
  makes this loader simple enough to trust.

User override merging (per [ADR-0010](0010-configuration-system.md))
happens **before** validation and default-filling. The merged value
is what is validated; the user override layer is therefore subject
to the same schema contract as the defaults layer.

## Consequences

Positive:

- Bugs in content are visible the first time the game starts, not
  three hours into a session.
- Schemas, code and behaviour stay aligned: a field defined in the
  schema is actually loaded; a field expected by code is actually in
  the schema.
- AI-generated content is held to the same standard.

Negative:

- A single typo in a defaults file blocks startup. This is the
  intended behaviour, but it raises the bar for content authors;
  schema-aware editing tools (which validate as you type) mitigate
  this.
- Error messages must be high quality. A loader that fails with
  "validation error" without saying which file and which field is
  worse than a silent fallback; we commit to good messages.

Follow-up:

- The loader emits errors in the form
  `<file>:<json-pointer>: <reason>` and includes the schema's
  `description` of the offending field when available.
- The CI job from [ADR-0012](0012-json-schema-validation.md) catches
  defaults-file bugs before they reach `main`.
