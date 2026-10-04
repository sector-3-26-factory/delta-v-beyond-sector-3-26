# ADR-0059: JSON is written as readable UTF-8, never with \u escapes

- **Status**: Accepted
- **Date**: 2026-10-04
- **Deciders**: Cute-Donkey
- **Supersedes**: None
- **Superseded by**: None

## Context

JSON permits any non-ASCII character to be written two ways: literally, or as a `\uXXXX`
escape. Both parse to the same string, so an escaped file behaves identically to a literal
one. Nothing breaks, no test fails, and schema validation accepts either.

That is exactly what makes the mistake persistent. `json.dumps` in Python defaults to
`ensure_ascii=True`, so any tool that reads a JSON file and writes it back — a one-off edit, a
migration script, an agent — silently rewrites every `—` as `—` and every `⋅` as
`\u22c5` unless it remembers to pass `ensure_ascii=False`. There is no global setting to
change: it is a keyword argument, so it is correct only where someone typed it.

The cost is readability and consistency, not correctness. It was observed in this repository:

- `assets/worlds/default.world.json`, `inspect.world.json` and `skirmish.world.json` each had
  the em dash in their `name` escaped, so `Sector 3.26 \u2014 Development Default` was the
  on-disk form of a name meant to be read.
- Three `maneuvering-thruster.json` files wrote their torque unit as `N\u22c5m` while
  `assets/json/schema/units.schema.json` — the file that *defines* that unit string — wrote
  it as `N⋅m` in four places.

The second case is the sharper one. A schema `$defs` key is a literal string, and the data
files that must match it were spelled a different way, so grepping `units.schema.json` for
`N⋅m` found the definition and missed every use of it.

## Decision

**Every `.json` file under `assets/` is stored as UTF-8 with literal characters. A `\uXXXX`
escape in shipped JSON is forbidden.**

`scripts/check-forbidden-patterns.sh` enforces it, the same way it enforces ADR-0013 and
ADR-0056. That script already runs in two places that a change cannot skip:

- `.github/workflows/ci.yml`, and
- `.githooks/pre-commit`.

Both are worth restating, because they are what makes the rule hold. Relying on every future
edit remembering a keyword argument is exactly the failure mode this ADR exists to remove; a
check that fails the build does not depend on anyone's memory.

The Rust side already complies and needs no change: `serde_json` writes UTF-8 directly and
does not escape non-ASCII, so saves written per ADR-0020 are unaffected by this rule. The
rule binds the tooling that rewrites the content files.

## Consequences

### Positive consequences

- Content files read the way they are meant to be read. A world name shows as
  `Sector 3.26 — Development Default` in an editor, a diff and a `grep`.
- A string in a data file is spelled the same way as the same string in the schema that
  defines it, so searching for it finds both.
- The failure becomes loud. An escaped file fails the check instead of sitting there looking
  correct.

### Negative consequences

- A tool that rewrites JSON must pass `ensure_ascii=False` explicitly, or run the check and
  fail. That is the intended friction, and it is the whole point of the rule.
- A JSON file that legitimately needs an escape — an encoding of the escape character itself,
  as in `"\\u0041"` — is caught by the same check and needs a hand-reviewed exemption. The
  pattern is simple enough that a false positive is cheap to spot, and no shipped file needs
  one today.

### Follow-up work

- None. The rule, the check and the wiring already exist at the time of writing; this ADR
  records the decision and points at them.

## Notes

- The check is a text scan for the two-character sequence `\u` inside a `.json` file, not a
  JSON parse. That is deliberate: the point is to catch what is written on disk, and a parse
  would normalise the escapes away before the check ever saw them.
- Escapes remain legal JSON. This is a repository convention about how we *write* files, not
  a claim that escaped JSON is invalid.
