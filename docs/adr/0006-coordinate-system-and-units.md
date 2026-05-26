# ADR-0006: Coordinate system and units

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

A game engine, a physics engine, an asset format and a level-design
workflow must all agree on a coordinate system. Disagreement causes
silent bugs: mirrored meshes, inverted gravity, flipped UVs, "why is
the ship flying backwards" surprises.

Our stack:

- **Bevy** uses a right-handed coordinate system with `+Y = up` and
  `-Z = forward` (i.e. cameras and meshes look down their local `-Z`
  axis by default).
- **glTF 2.0**, our primary asset format, defines exactly the same
  convention: right-handed, `+Y = up`, `-Z = forward`.
- Players will design ships, stations and worlds in JSON + glTF, with
  the help of AI tooling. Any convention we adopt must be the one a
  glTF exporter (Blender, Godot, etc.) produces by default.

## Decision

We adopt **Bevy's and glTF 2.0's coordinate convention unchanged**:

- Right-handed system.
- `+X` points to the right.
- `+Y` points up.
- `-Z` points forward (so a ship's nose points along its local `-Z`).

Units:

- Length: **meters** (`m`) at the engine level.
- Mass: **kilograms** (`kg`) at the engine level.
- Time: **seconds** (`s`).
- Angles: **radians** internally; degrees only for human-facing UI.

JSON content may use other units (`km`, `AU`, `pc`, `ly`, `t`,
`M_earth`, `M_sun`) for human convenience; they are converted to the
engine units above on load. See
[ADR-0008](0008-physical-units-in-json.md).

These conventions are documented as JSON Schema defaults wherever they
appear in content schemas; see [ADR-0012](0012-json-schema-validation.md).

Why not invent our own (e.g. `+Z = forward`):

- Every glTF would need axis swapping on import and export. Mesh
  normals, animations and skeletons all carry coordinate baggage.
- All Bevy tutorials, examples and third-party plugins assume the
  default; deviating multiplies friction permanently.
- Players who design content in standard tools (Blender, Godot,
  Unity-with-glTF-export) would have to remember our exception
  forever.

## Consequences

Positive:

- glTF round-trip works without conversion code.
- Bevy primitives (`Camera3dBundle`, `DirectionalLightBundle`,
  `Transform::looking_at`) behave as documented.
- Player-created content "just works" if exported from standard tools.

Negative:

- Casual descriptions sometimes say "Z is forward/back, X is
  left/right, Y is up/down" without specifying sign. We must remember
  that **forward is `-Z`**, not `+Z`. The schemas and code must be
  explicit about this whenever direction matters.

Follow-up:

- A code-level helper (e.g. `Direction::FORWARD`) and matching schema
  named constants (`"forward"`, `"backward"`, ...) are added in the
  relevant crates so that JSON authors never have to write raw signed
  Z values.
