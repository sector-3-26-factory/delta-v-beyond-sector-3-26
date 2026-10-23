# ADR-0058: Mass belongs to the world, not the template

- **Status**: Accepted
- **Date**: 2026-10-02
- **Deciders**: Cute-Donkey
- **Supersedes**: None
- **Superseded by**: None

## Context

An entity template under `assets/templates/<category>/<name>/` describes one mesh: its
`mesh.glb`, its `collision_shape`, its `bounding_box`, and its visual flags. Per ADR-0038 the
template is one entity type with its own schema, and a world file references templates by
path rather than inlining definitions.

Templates also carried a `mass`. That field cannot belong there, and the codebase shows why.

**One mesh is reused at many sizes.** `moons/shared/mesh_1` is referenced by **288 moon
entities in `solar-system.world.json` at 288 distinct scales**, from Deimos at 52.9 to Ganymed
at 22,490. There is no single scale at which one template number describes the mesh.

**The bodies are not one object scaled up.** The real moons range from 1,479 kg/m³ (Deimos) to
3,528 kg/m³ (Io) — a factor of 2.4 in density, because the small ones are rubble and the
large ones are rock and ice. Mass across the moons therefore does not follow volume.

**The current state is already wrong, and silently so.** Because no world entity carried a
mass, every one of the 288 moons received the template's 5.0e10 kg. Ganymede, whose real mass
is 1.482e23 kg, was wrong by a factor of 3e12. Phobos was wrong by 2e5. Nothing reported the
error: the number was plausible, uniform, and wrong.

## Decision

We move `mass` out of templates and into world entities. **A template describes geometry;
a world file describes bodies.**

- **No template schema carries `mass`.** It is removed from `asteroid.schema.json`,
  `planet.schema.json`, `moon.schema.json`, `sun.schema.json` and `ship.schema.json`, and from
  every template file that has it.
- **Every world entity that produces a body carries its real `mass`.** It states the mass of
  that object in that world, in kilograms or an equivalent unit per ADR-0008.
- **`scale` does not affect mass.** `scale` is size and appearance. It multiplies the mesh,
  the collision shape and the bounding box, and nothing else.
- **`mass` stays optional in `world.schema.json` and is hard-required by the spawn system.**
  A belt or a field entity is a definition rather than a body and has no mass, and ADR-0012
  rule 7 forbids the `if/then/else` or `oneOf` that would express "required unless the
  template is a belt". A body spawned without a mass is a hard error at spawn per ADR-0013,
  naming the entity, and is never defaulted. Belt and field spawners never read the field; a
  generated asteroid's mass comes from its region's `density` and the radius it drew, which is
  the one place density belongs.

## Consequences

### Positive consequences

- **The 288 moons get their real masses.** Ganymede goes from 5.0e10 kg to 1.482e23 kg,
  Io to 8.932e22 kg, Phobos to 1.066e16 kg. The error stops being a factor of 3e12 and
  becomes zero.
- **SOI radii and gravity become correct as a consequence**, because both read the body's
  mass. Jupiter's SOI comes from its own mass rather than from the mass of whatever template
  happened to be reused.
- **A shared mesh stops forcing one density on many objects.** Two entities using the same
  template may now have different masses, which is what lets one mesh serve a rubble-pile
  moon and an icy one.
- **The error becomes loud.** A body with no mass fails at spawn instead of silently
  inheriting a plausible number from a template.

### Negative consequences

- **Every world file grows by a field per entity.** `solar-system.world.json` gains 288 mass
  entries, and `mass` must be kept correct when an entity is resized. That maintenance cost
  is the price of correctness and it is real.
- **A template can no longer be spawned without a world file that states a mass.** Anything
  that constructs a spawn event programmatically must supply one, including tests and the
  benchmark harness.
- **Density has exactly one home.** It belongs on belt and field templates and nowhere else,
  so an object that needs a density-derived mass outside a generated region has no way to ask
  for one. That is deliberate; the alternative was two sources for mass.
- **Removing `mass` from a template is a breaking change for any content authored against the
  old schemas.** No shipped content remains once the world files are updated in the same
  change, but external world files would need migrating.

### Follow-up work

- Belt and field templates gain a `density`, and the generator derives each body's mass as
  `density × 4/3πr³`. That is Phase 7 of the asteroid belt plan and depends on this ADR.
- `world.schema.json`'s description for `mass` is rewritten to state that it is the body's
  real mass in this world, and that it is required for any entity which produces a body.

## Notes

- Real moon masses, semi-major axes, radii and orbital elements are taken from the gathered
  dataset in `tmp/moons_data.json`, which holds 288 records and matches the 288 moon entities
  one-to-one.
- Mass and orbital period are both authored, in the same file, by the same author. They are
  not derived from one another, because there is no coupling to resolve: `scale` no longer
  touches mass, so an authored period cannot contradict an authored mass. Deriving the period
  from Kepler's third law was considered and rejected on that basis.