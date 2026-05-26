# ADR-0012: JSON schema validation

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Every piece of JSON content the game loads (configuration, ship types,
worlds, items, ...) is a potential source of subtle bugs: typos in
field names, wrong units, missing required fields, extra fields with
silent meaning. Catching these at load time -- not at the point where
a missing value crashes a system three plugins deep -- is essential.

The project also has an explicit goal that **players (with AI help)
generate content**. Without a machine-checkable schema, AI tools have
no reliable contract, and "the JSON looks right" gets us further than
"the JSON is right".

## Decision

Every JSON file the game reads is governed by a **JSON Schema**
(Draft 2020-12 or later) stored under:

```
assets/json/schema/
```

Rules:

1. There is **one schema per kind of file**: `keybindings.schema.json`,
   `ship-type.schema.json`, `sector.schema.json`, ...
2. Every schema sets `"additionalProperties": false` at every object
   level unless there is an explicit reason not to (documented inline
   in the schema). Unknown fields are bugs, not extensibility points.
3. Every field that has a meaningful default is given that default in
   the schema via `"default"`. The schema is the **single source of
   truth for defaults** (per
   [ADR-0014](0014-engine-constants-vs-gameplay-values.md)). The
   loader inserts schema defaults for omitted fields as a separate,
   recursive pass after validation and before deserialisation; the
   exact algorithm and rules are specified in
   [ADR-0013](0013-no-silent-fallbacks.md). `serde_json` is not
   schema-aware and does not perform this pass on its own.
4. Schemas use JSON Schema's `description` field generously: each
   field, each enum, each named type carries human-readable
   documentation. This documentation is read by humans, by AI agents
   generating content, and (eventually) by docs generators.
5. Validation runs at load time with `jsonschema` (Rust crate) or
   equivalent. Validation failures abort loading with a precise
   error message (file path, JSON pointer, expected vs. actual).
6. Schemas are versioned implicitly with the game version; cross-
   version migration is a separate concern not covered here.
7. The validator runs the **defaults file** through the schema at
   startup too. A defaults file that violates its own schema is a
   release-blocking bug.
8. **Schemas do not use combinator keywords whose chosen branch
   would change which defaults apply.** Concretely:
   `oneOf`, `anyOf`, `if`/`then`/`else` and conditional
   `dependentSchemas` are **forbidden** when their branches
   declare different `default` values, different required fields,
   or different property sets. Reason: our default-filling pass
   (see [ADR-0013](0013-no-silent-fallbacks.md)) is intentionally
   a simple structural walk; it does not evaluate JSON-Schema
   combinators to decide which branch applies. Allowing such
   combinators would force the loader to re-implement a sizeable
   part of the JSON-Schema engine, with all the corner cases that
   come with it.

   The supported way to model "a thing that can be one of several
   variants" is **a separate schema file per variant**, with an
   explicit `"type"` (or similarly named) discriminator field:

   - `ship.schema.json`: requires `"type": "ship"`, plus the
     ship-specific properties and their defaults.
   - `sun.schema.json`: requires `"type": "sun"`, plus the
     sun-specific properties and their defaults.
   - `planet.schema.json`: requires `"type": "planet"`, etc.

   The loader uses the discriminator to pick the matching schema
   and then runs the standard validate-then-fill-defaults pass
   against it. Each variant schema is self-contained; the union
   exists in code (as a Rust enum or a registry of loaders), not
   in the schema language.

   `enum` for closed sets of *scalar* values (`"easy" | "normal"
   | "hard"`) remains fine, because no branch-dependent defaults
   are involved.

## Consequences

Positive:

- Bad content fails fast and loud.
- AI-generated content has a reliable target.
- Schemas double as documentation.
- The "no silent fallbacks" rule
  ([ADR-0013](0013-no-silent-fallbacks.md)) is mechanically
  enforceable.

Negative:

- Adding a field means updating the schema, the loader struct and
  possibly the migration story. This is exactly the discipline we
  want; the cost is the price of the safety.
- Strict `additionalProperties: false` will occasionally bite during
  prototyping; the fix is to add the field to the schema, not to
  loosen the schema.
- Banning branch-dependent combinators (`oneOf`, `if-then-else`,
  ...) means a few patterns that JSON Schema can express are out
  of reach. We accept this: the per-variant-schema alternative is
  more explicit, plays well with our loader, and aligns with how
  the Rust side already models such cases (one enum variant per
  variant schema).

Follow-up:

- A linter pass on schemas themselves ensures they declare a
  `$schema` and a `$id`, set `additionalProperties: false`, carry
  `description` on every field, and contain no forbidden
  combinators (rule 8). Deferred.
- The `units.schema.json` mentioned in
  [ADR-0008](0008-physical-units-in-json.md) is reused via `$ref`
  from every schema that has a physical quantity. Note that `$ref`
  to a sibling sub-schema is fine; it does not introduce branch-
  dependent defaults.
