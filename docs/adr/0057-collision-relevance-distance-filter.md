# ADR-0057: Collision relevance as a distance test

- **Status**: Accepted
- **Date**: 2026-10-02
- **Deciders**: Cute-Donkey
- **Supersedes**: None
- **Superseded by**: None

## Context

`collision_detection_system` in `delta-v-physics` collects every entity carrying
`RigidBody + Transform + CollisionShape + CollisionLayersComponent` and tests all pairs:

```rust
for i in 0..len {
    for j in (i + 1)..len {
```

The pair count is `n * (n - 1) / 2` and grows with the **square** of the body count. There
is no broadphase and no pre-filter outside the loop. This is a property of the algorithm,
not of any particular machine.

The body count is about to stop being a small number. Asteroid belts generate their
asteroids as concrete entities near the player, so the loaded body count becomes thousands
rather than the current few hundred, and the quadratic loop becomes the dominant cost of the
simulation.

The loop cannot be made sub-quadratic by a different algorithm without a broadphase — a
uniform spatial hash, a BVH, or swept volumes. Each of those is a real structure that must be
built, maintained across the floating-origin recentres of ADR-0007, and kept correct while
bodies move. Before paying that cost we ask a cheaper question: how many of those pairs can
never produce a collision that matters?

A pair can only matter if the two bodies are close enough to overlap. Bodies are not close to
each other over most of a belt, and most of a belt is not close to the player. Both facts are
already computed for other reasons.

`lazy_load_celestial_meshes` computes, for every loaded celestial body every frame, the body's
apparent size:

```
screen_radius_px = focal_px * radius_m / distance_m
```

One divide and one multiply per body, already paid for, and it is stored per body in
`LazyLoadMesh::current_screen_radius_px`.

**This number is not a visibility measurement.** It answers "is this body near enough to
matter", and it deliberately ignores where the player is looking. A body behind the player
scores the same as one ahead. An earlier attempt decided visibility by asking the camera what
it could see; that approach was rejected because it makes a body enter and leave the set as
the player turns, which produces no stable threshold and no usable cost bound. There is no
camera, frustum or view direction in this decision.

## Decision

We use a **collision relevance filter**: a body is *collision-relevant* when

```
current_screen_radius_px >= collision_relevance_px
```

and the pair loop skips pairs that cannot matter, as follows.

- **A body with no `LazyLoadMesh` is always tested.** It has no
  `current_screen_radius_px`, so no relevance value can be read. Ships, projectiles and
  stations do not carry the component, and they must keep behaving exactly as they do today.
  A body that lacks the component is never treated as irrelevant.
- **Ship, projectile and station pairs are tested when at least one side is relevant.**
- **Asteroid/asteroid pairs are tested only when both sides are relevant.** Both are always
  asteroids, so both carry the component and both have a value.

**The bodies are partitioned once per tick, before any pair is examined.** Not filtered
inside the pair loop — partitioned ahead of it:

- `relevant_asteroids` — on the asteroid layer and above the threshold
- `irrelevant_asteroids` — on the asteroid layer and below the threshold
- `others` — everything else: ships, projectiles, planets, moons

Exactly four pair groups are then formed, and the other two are never formed at all:

| Pair group | tested? |
|---|---|
| relevant asteroid x relevant asteroid | yes |
| relevant asteroid x non-asteroid | yes |
| irrelevant asteroid x non-asteroid | yes |
| non-asteroid x non-asteroid | yes |
| relevant asteroid x irrelevant asteroid | **never formed** |
| irrelevant asteroid x irrelevant asteroid | **never formed** |

An irrelevant asteroid is still walked, because it must collide with ships. It is walked only
against non-asteroids, never against the whole belt.

**The position of the filter is the whole point, and it was got wrong first.** Checking
relevance inside the existing `for i { for j in (i+1)..len }` loop still runs `n²/2`
iterations and skips only the shape test at the end of each. That measured 3.4x at 8,000
bodies, because the loop overhead dominated and the shape test was the cheap part. Moving
the same test above the loop took the same measurement to 801x. See Measured.

The threshold lives as a field on `LazyLoadMesh`, named `collision_relevance_px`, initialised
to `COLLISION_RELEVANCE_PX` at spawn. It is an engine constant in `delta-v-physics`
(`constants.rs`), not a gameplay value, so per ADR-0014 it is a Rust `const` rather than a
JSON field. No new component, no new system and no new crate are introduced.

**`COLLISION_RELEVANCE_PX` is 8.0.** The 1-pixel figure already used for mesh loading is the
point at which a body first becomes resolvable; at 8 pixels a body is roughly eight times
further away than that. Every collision a player could observe therefore happens inside the
relevance radius, and the only collisions not tested are ones no player can perceive.

The relevance value is computed once per frame by the system that already computes it. The
collision system does not recompute apparent sizes, does not read the camera, and does not
take a camera or frustum as a parameter.

## Consequences

### Positive consequences

- The pair loop no longer scales with the square of the body count. Two of the six possible
  pair groups are never formed, so the asteroid-to-asteroid work is `n_relevant²/2` instead of
  `n_asteroid²/2`, and an irrelevant asteroid costs `n_others` instead of `n_total`.

- Nothing observable is lost. The threshold sits eight times beyond the distance at which a
  body first becomes a pixel, so every perceptible collision is inside it.
- No new data is computed. The relevance value is a field the lazy loader already writes every
  frame.
- No new structure is maintained. There is no spatial hash to rebuild across floating-origin
  recentres, and no extra memory per body beyond one float.
- Bodies without the component are explicitly unaffected, so weapons, player flight and
  station docking cannot regress.

### Negative consequences

- A collision between two bodies that are both below the threshold is not detected. This is
  accepted deliberately: such a pair is beyond the distance at which either body is more than
  a few pixels across.
- The asteroid/asteroid rule requires both sides to be relevant, which means two relevant
  asteroids still test against every body in the world, including ships. This is the price of
  not giving ships and projectiles a special case.
- The filter is a distance cut, so it is a heuristic on apparent size rather than a spatial
  query. It cannot become a replacement for a broadphase; a world dense enough to make
  `n_relevant²` itself unaffordable would still need one.
- The threshold is a constant. If the default is ever lowered below the 1-pixel figure, the
  filter would begin to drop collisions the player can see. Any change to it is a gameplay
  change and needs review.
- **Partitioning allocates three `Vec`s and fills them every tick.** That is linear in `n` and
  is the price of the change, paid whatever the relevance distribution. It is small against
  what it saves, but it is not free, and it cannot be avoided by making the filter cheaper —
  only by making the population smaller, which is what belt streaming does.

### Measured

`benchmark_collision_relevance` measures the unfiltered loop and two filtered populations at
the same body count and the same layout, in one run. Release build; the number quoted is the
ratio of the unfiltered median to the filtered median, which is what an `approach-comparison`
benchmark exists to produce and is a property of the approach rather than of the machine.

| Asteroids | 10% relevant | ratio | 1% relevant | ratio |
|---|---|---|---|---|
| 500 | 0.051 ms | 38.3x | 0.061 ms | 31.8x |
| 1,000 | 0.107 ms | 43.5x | 0.044 ms | 107.0x |
| 2,000 | 0.350 ms | 38.7x | 0.069 ms | 197.1x |
| 4,000 | 1.234 ms | 39.2x | 0.140 ms | 345.9x |
| 8,000 | 4.294 ms | 45.7x | 0.245 ms | 801.5x |

A repeat run of the same build gave 669.9x at 8,000 bodies and 1% relevant, against the
801.5x above. Run-to-run variance is of that order, so the **decimals are not meaningful and
the ratio is not a gate** (ADR-0056); what carries the argument is the shape down the table
and the two orders of magnitude between the in-loop and partitioned versions.

**The 1% ratio grows with body count, which is the shape the arithmetic predicts.** Cutting
the relevant population from 10% to 1% removes a factor of 100 from the pair count, so the
gap between the two filtered variants should itself grow with `n`. At 500 bodies there is
nothing left to separate and the two are indistinguishable. By 8,000 the 1% variant is 17.5x
faster than the 10% variant, which is the pair arithmetic showing through.

**The contrast with the in-loop version is the finding.** The same filter applied inside
`for i { for j }` measured 3.4x at 8,000 bodies, and its 10% and 1% variants were
*identical* — proof that the loop overhead, not the shape test, was the cost. Moving the same
test above the loop took the identical measurement from 3.4x to 801x. Nothing about the
filter's rule changed; only its position did.

### Follow-up work

- The benchmark in `crates/delta-v-physics/src/approach_comparison/` gains a relevance
  dimension so the relative cost of the filtered and unfiltered loops is measured rather than
  assumed, per ADR-0056 (`approach-comparison`, baseline `sibling-variants-same-run`).
- ADR-0052 (proposed) describes a multi-phase collision pipeline whose midphase is a spatial
  structure. This ADR does not pre-empt it: the relevance filter is a cheap pre-filter that
  the midphase would sit behind, not an alternative to it.

## Notes

- Apparent size is `focal_px * radius_m / distance_m`, so a fixed pixel threshold is a fixed
  multiple of `radius_m²`: the relevance sphere of a 1 km asteroid at 8 px is about 5,216 km.
  Larger asteroids are relevant from further away, which is the correct behaviour — a large
  body is still worth colliding with when it is far.
- At the real density of the main asteroid belt, no second asteroid is ever inside the
  relevance sphere, so a realistic belt has no collision cost at any population.