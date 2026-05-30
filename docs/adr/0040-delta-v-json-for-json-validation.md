# ADR-0040: delta-v-json as the standard JSON validation and defaults library

- **Status**: Accepted
- **Date**: 2026-01-20
- **Deciders**: Cute-Donkey

## Context

ADR-0012 and ADR-0013 define a strict JSON loading discipline:

1. Schemas are the single source of truth for defaults ([ADR-0014](0014-engine-constants-vs-gameplay-values.md)).
2. Schema validation is mandatory; validation failures must abort with precise errors.
3. Defaults are applied as a separate, explicit pass after validation.
4. No branch-dependent combinators (`oneOf`, `anyOf`, `if-then-else`) can declare differing defaults.
5. Every field with a meaningful default must be declared in the schema.

This discipline is **mechanically enforceable** but requires careful implementation:

- A schema-aware validator that reports errors with file path, JSON pointer, and expected vs. actual values.
- A deterministic defaults-filling algorithm that walks the schema and parsed JSON in parallel, inserting `"default"` values for omitted fields at every nesting level.
- Re-validation after defaults insertion (to catch bugs in the defaults algorithm itself).
- Integration with `serde` for deserialisation into Rust structs.

The `delta-v-json` crate provides exactly this: a complete, tested implementation of the ADR-0012/0013 discipline. It handles validation, defaults filling, re-validation, and produces clean error messages with precise JSON pointers.

Agents and developers often re-implement this logic per-type or add `#[serde(default = "...")]` attributes instead of using the schema. This violates ADR-0013 and creates maintenance burden.

## Decision

1. **The `delta-v-json` crate is the standard library** for all JSON loading in the project.
2. Every schema and every loader uses `delta-v-json` to:
   - Validate against the schema (catching typos and invalid values).
   - Fill defaults from the schema (enforcing ADR-0012/0013).
   - Re-validate (ensuring defaults are correct).
   - Deserialise into Rust structs.
3. **No per-type default logic** in Rust code:
   - ❌ No `#[serde(default = "fn_name")]` on struct fields.
   - ❌ No `impl Default` for types that must be loaded from JSON.
   - ❌ No custom defaults functions `fn default_X()`.
   - ✅ All defaults in `*.schema.json` files only.
4. Error messages from `delta-v-json` are propagated directly to users:
   - File path, JSON pointer, schema location, and description.
   - No wrapper or translation; the library's messages are precise enough.
5. New loaders start from a schema and call `delta_v_json::load::<T>(schema, json_string)` or equivalent. The schema is the only authority.

## Consequences

Positive:

- ADR-0012 and ADR-0013 are mechanically enforced; violations are compile-time or load-time errors, not code review surprises.
- Developers and agents do not need to re-implement validation or defaults logic; the library does it.
- Error messages are consistent and precise across all JSON loading.
- Schemas are the single source of truth; no hidden defaults in code.
- The discipline becomes so lightweight that it is easier to follow than to violate.

Negative:

- A new dependency (`delta-v-json`). Mitigated by the fact that it is essential to the architecture (ADR-0012/0013 without it are incomplete).
- Developers must learn the `delta-v-json` API. Mitigated by clear examples in early loaders and documentation.

Follow-up:

- A linter or CI check verifies that no Rust struct used for JSON loading carries `#[serde(default = "...")]` or custom `Default` implementations. See [ADR-0039](0039-enforcement-of-json-only-defaults.md).
- Documentation and examples showing how to use `delta-v-json` for each kind of content (keybindings, ship types, sectors, ...).

## References

- [ADR-0012: JSON schema validation](0012-json-schema-validation.md) — the schema discipline.
- [ADR-0013: No silent fallbacks](0013-no-silent-fallbacks.md) — the defaults and validation contract.
- [ADR-0014: Engine constants vs gameplay values](0014-engine-constants-vs-gameplay-values.md) — why defaults belong in schemas.
- [ADR-0039: Enforcement of JSON-only defaults](0039-enforcement-of-json-only-defaults.md) — tooling to prevent Rust-side defaults.