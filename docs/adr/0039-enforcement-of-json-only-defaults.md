# ADR-0039: Enforcement of JSON-only defaults

- **Status**: Accepted
- **Date**: 2026-05-28
- **Deciders**: Cute-Donkey
- **Supersedes**: None (reinforces ADR-0013, ADR-0014)

## Context

ADR-0013 (No silent fallbacks) and ADR-0014 (Engine constants vs gameplay
values) establish that:

1. All defaults for JSON-loaded data must live in JSON schemas.
2. Rust code must not create silent defaults via `#[serde(default)]` or
   custom default functions.
3. Missing required fields must be hard errors at load time, not hidden
   in Rust.

However, this rule is frequently violated in practice. Agents and
contributors commonly add:

- `#[serde(default = "default_rotation")]` on struct fields
- `Option<T>` wrappers for fields that should always be present
- Custom `fn default_X() -> T` functions

These violations are **silent fallbacks** that violate ADR-0013 and
increate debugging difficulty when schemas and Rust code disagree on
defaults.

This ADR explicitly forbids these patterns and provides an enforcement
mechanism.

## Decision

We enforce the following rules **strictly**:

### Rule 1: No `#[serde(default)]` on JSON-backed struct fields

**Forbidden:**
```rust
#[derive(Deserialize)]
pub struct Entity {
    pub position: Vec3,
    #[serde(default = "default_rotation")]  // FORBIDDEN
    pub rotation: Quat,
}

fn default_rotation() -> Quat { Quat::IDENTITY }
```

**Allowed (with justification):**
```rust
#[serde(default)]  // Only for #[serde(skip)] or non-JSON fields
pub internal_cache: Vec<Cached>,
```

**Rationale:** The schema must be the single source of truth for defaults.
If a field is missing and you want a default, that default must be in
`*.schema.json`, applied by the `delta-v-json` validation pass, and then
loaded into Rust as an explicit value. No exceptions.

### Rule 2: No `Option<T>` for fields that should always exist

**Forbidden:**
```rust
pub struct EntitySpawn {
    pub position: Vec3,
    pub rotation: Option<Quat>,  // FORBIDDEN if the schema provides a default
}
```

**Allowed:**
```rust
pub struct EntitySpawn {
    pub position: Vec3,
    pub rotation: Quat,  // Schema guarantees this exists after load
}
```

**Exception:** Use `Option<T>` **only** if the field is logically optional
and the schema reflects this (no default provided, field is not required).

**Rationale:** `Option` is for data that may legitimately be absent. If the
schema provides a default, the field is not optional—it's guaranteed to
exist. Using `Option` masks this guarantee and leads to unnecessary `.unwrap()`
or `if let` checks.

### Rule 3: All defaults in `*.schema.json` only

**Required in schema:**
```json
{
  "properties": {
    "rotation": {
      "description": "Unit quaternion...",
      "type": "object",
      "properties": { "x": {...}, "y": {...}, "z": {...}, "w": {...} },
      "default": { "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 }
    }
  }
}
```

**Forbidden in Rust:** No `fn default_rotation()`, no `#[serde(default)]`.

### Rule 4: Hard error on validation failure

If a required field is missing and the schema has no default, the JSON
validation must **fail** and produce a clear error. This is correct
behavior per ADR-0013.

## Code Review Checklist

All PRs touching JSON deserialization must pass this checklist:

- [ ] **No `#[serde(default = "...")]` attributes** on JSON-backed structs?
  - Exception: Only allowed on `#[serde(skip)]` fields with documented reason.
- [ ] **No custom default functions** like `fn default_X() -> T`?
- [ ] **No unnecessary `Option<T>` fields** for data that should always exist?
  - If using `Option`, is there a comment explaining why the field is logically optional?
- [ ] **All defaults in schema files** (`*.schema.json`)?
  - Verify `"default"` keys in schema for fields with defaults.
- [ ] **Struct fields match schema** (required ↔ not `Option`, optional ↔ `Option`)?
- [ ] **Test covers missing required field** to ensure hard error?

## Consequences

Positive:

- Single source of truth: schema is authoritative for defaults.
- No silent fallbacks: missing required fields fail validation, not Rust code.
- Clearer Rust code: no Option checks for fields that are always present.
- Better error messages: schema validation errors tell the user exactly what's wrong.

Negative:

- More boilerplate in schemas (but that's intentional—explicit is better).
- Slightly more friction when adding new fields (must update schema first).
- Agents/contributors must learn the rule and follow the checklist.

Follow-up work:

- Update AGENTS.md with a pointer to this ADR.
- Add code review checklist to PR templates (if applicable).
- Audit existing code for violations and fix them in a separate PR.

## Notes

- This ADR reinforces ADR-0013 (No silent fallbacks) and ADR-0014 (Engine
  constants vs gameplay values), but is more specific and prescriptive.
- The checklist is mandatory for all JSON deserialization code (delta-v-config,
  delta-v-world, delta-v-json, etc.).
- Violations are code review failures; they do not merge until fixed.
