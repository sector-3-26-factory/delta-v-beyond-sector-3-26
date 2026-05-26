# ADR-0008: Physical units in JSON

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Authors of JSON content (ships, planets, sectors, stars, ...) think in
units appropriate to the object: metres for ship dimensions,
kilometres for orbital distances, astronomical units or parsecs for
interstellar maps, kilograms for projectiles, solar masses for stars.

Forcing every value into engine-base units (m, kg, s) would make
content unreadable ("the ship is 4.7e16 m from the sun"). Allowing
implicit units is worse: a number with no unit is a bug waiting to
happen.

## Decision

Numeric quantities in JSON that represent a physical magnitude are
expressed as an **explicit object** with a `value` and a `unit`:

```json
{
  "distance_from_star": { "value": 1.0, "unit": "AU" },
  "mass":               { "value": 2.0, "unit": "M_sun" },
  "length":             { "value": 75,  "unit": "m" }
}
```

Allowed units (the canonical list lives in
`assets/json/schema/units.schema.json`):

- **Length**: `m`, `km`, `Mm`, `Gm`, `AU`, `ly`, `pc`, `kpc`, `Mpc`.
- **Mass**: `kg`, `t`, `M_earth`, `M_sun`.
- **Time**: `s`, `min`, `h`, `d`, `a` (Julian year).
- **Angle**: `rad`, `deg`.

Conversion to engine-base units (m / kg / s / rad, per
[ADR-0006](0006-coordinate-system-and-units.md)) happens once at load
time. Internally the engine never carries the original unit. If a
displayed value should re-acquire a human-friendly unit (HUD, debug
overlay), the formatting code chooses the appropriate one.

Constraints:

- `unit` is required. There is no implicit default unit. Missing
  `unit` is a schema validation failure
  (per [ADR-0012](0012-json-schema-validation.md) and the no-silent-
  fallbacks rule in [ADR-0013](0013-no-silent-fallbacks.md)).
- `value` must be finite. `NaN`, `+Infinity`, `-Infinity` are
  rejected at load time.
- The set of allowed units per field is constrained by the schema.
  A `length` field will not accept a mass unit.

## Consequences

Positive:

- JSON content is self-documenting.
- Mistakes (writing `1.0` meaning AU into a field expecting metres)
  are caught at load time by the schema.
- The interstellar-vs-local-scale tension noted in
  [ADR-0007](0007-floating-origin.md) is solved at the content level
  without bleeding into engine code.

Negative:

- JSON values are wordier than bare numbers.
- A small loader and validator must understand the unit set.
- Tooling (AI assistants generating content) must produce the object
  form, not bare numbers.

Follow-up:

- `delta-v-config` (or `delta-v-core`) provides typed wrappers
  (`Length`, `Mass`, `Duration`, `Angle`) whose `Deserialize`
  implementation accepts the `{value, unit}` form and converts to
  base units.
- The `units.schema.json` file is the single source of truth for the
  allowed unit set.
