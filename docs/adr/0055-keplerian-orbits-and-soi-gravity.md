# ADR-0055: Keplerian orbits for celestial bodies and Sphere of Influence gravity for dynamic entities

- **Status**: Accepted
- **Date**: 2026-09-26
- **Deciders**: Cute-Donkey
- **Supersedes**: ADR-0009

## Context

ADR-0009 established Newtonian physics with gravitational interaction between all simulated bodies, using a cutoff radius to limit O(N²) cost. However, with the addition of 286 moons to the solar system (289 total gravity sources), the pairwise gravity calculations between celestial bodies became computationally unsustainable (~83,000 pairs per physics tick).

The current approach has three fundamental problems:

1. **Performance**: O(N²) gravity between celestial bodies does not scale. 289 gravity sources = ~83,000 pairwise calculations per fixed timestep, causing severe frame rate drops and system freezes.

2. **Orbital stability**: N-body integration of celestial bodies causes orbital drift and decay over time. Moons and planets should follow stable, predictable Keplerian orbits defined by their orbital parameters (semi-major axis, eccentricity, inclination, period), not emerge from numerical integration.

3. **Gameplay irrelevance**: Players cannot perceive mutual gravitational perturbations between distant moons. The visual and gameplay difference between Keplerian and N-body celestial motion is negligible at human scales.

The existing orbital hierarchy system (parent-child relationships with orbital parameters in `orbital_parameters`) already provides deterministic, analytical positioning for celestial bodies. This system should be the authoritative source of celestial positions.

For dynamic entities (ships, asteroids, debris), a Sphere of Influence (SOI) / Patched Conic model provides physically plausible gravity with O(1) cost per entity.

## Decision

We transition to a two-tier gravity architecture:

### 1. Celestial Bodies (Stars, Planets, Moons) — Keplerian Orbits Only

- **No N-body gravity calculations between celestial bodies.** The physics engine does not compute pairwise gravitational forces between stars, planets, or moons.
- **Positions determined analytically** from orbital parameters (`orbital_parent`, `orbital_distance`, `orbital_period`, `orbital_eccentricity`, `orbital_inclination`, `initial_orbital_angle`) at simulation time `t`.
- **Orbital hierarchy is authoritative**: A moon's position is computed relative to its parent planet; a planet's position relative to the sun. This is evaluated deterministically each frame/tick.
- **Celestial bodies remain gravity sources** for dynamic entities (ships, asteroids) within their SOI.
- **`is_gravity_source: true`** in templates is retained for SOI calculations, but the physics engine ignores celestial-celestial gravity.

### 2. Dynamic Entities (Ships, Asteroids, Debris) — Sphere of Influence (SOI) Gravity

- **Patched Conic approximation**: Each dynamic entity resides in exactly one body's SOI at a time.
- **SOI radius** calculated as `r_soi = a * (m/M)^(2/5)` where `a` is orbital semi-major axis, `m` is body mass, `M` is parent mass. Configurable per body in JSON with a sensible default.
- **Gravity applied only from the dominant SOI body**: `g = G * M / r²` toward the SOI parent. No summation of multiple gravity sources.
- **SOI transitions**: When a dynamic entity crosses an SOI boundary, gravity source switches instantaneously (patched conic). Hysteresis or blending may be added later for smoothness.
- **Player and AI ships share identical SOI logic** for gameplay parity.

### 3. Asteroid Belt Physics

- **Idle state**: Asteroids follow Keplerian orbits around their parent body (same as moons). No mutual N-body attraction between asteroids.
- **Disturbed state**: On collision, explosion, or weapon impact, affected asteroids/debris switch to standard **Rigid Body Impulse Physics** (linear/angular momentum, collider collisions).
- **No mutual gravitational attraction** between asteroid fragments, ever.
- **Re-stabilization**: Optionally, disturbed asteroids can return to Keplerian orbits after a timeout or when velocity drops below threshold.

### 4. Future Extensibility — Dynamic Gravity Attractors

- **Explicit exception for special objects**: Black holes, gravity bombs, tractor beams, or other exotic gameplay objects may act as **temporary dynamic point-attractors**.
- These are **not** celestial bodies; they are spawned entities with a `gravity_attractor` component.
- They apply gravity to dynamic entities within a configurable radius, additive to or overriding the SOI gravity.
- This does not re-introduce celestial N-body; it is a scoped gameplay mechanic.

## Consequences

### Positive

- **CPU overhead for celestial positioning drops to near 0 ms/frame** — analytical evaluation is O(N) with tiny constant factor.
- **Stable, predictable orbits** — no numerical drift, no 3-body chaos, orbits match orbital parameters exactly.
- **O(1) gravity per dynamic entity** — scales to thousands of ships/asteroids without performance cliff.
- **Deterministic by construction** — analytical orbits + single SOI source = no summation order issues.
- **Enables 289+ moons** — visual completeness without physics cost.

### Negative

- **No emergent Lagrange points, resonances, or tidal effects** between celestial bodies. These were not gameplay-relevant at current scale.
- **SOI model is an approximation** — real gravity doesn't have hard boundaries. Acceptable for gameplay; blending can be added if needed.
- **Celestial bodies no longer perturb each other** — e.g., Jupiter's moons don't gravitationally interact. This matches the "Keplerian hierarchy" design.
- **Migration effort** — physics crate gravity systems must be refactored; orbital system must become authoritative for celestial positions.

### Follow-up Work

1. **Update `delta-v-physics`**: Remove celestial-celestial gravity summation. Retain SOI gravity for dynamic entities.
2. **Ensure orbital system runs before physics** — celestial positions updated analytically each tick, then physics uses those positions for SOI queries.
3. **Add SOI radius to celestial templates** (JSON schema) with auto-calculated default.
4. **Update `ARCHITECTURAL_RULES.md`** to reflect new gravity architecture.
5. **Add debug overlay** for SOI boundaries (deferred).

## Notes

- ADR-0009 is **superseded**, not deleted. Its rationale for "why gravity at all" remains valid for dynamic entities.
- The cutoff radius concept from ADR-0009 is replaced by SOI radius — conceptually similar but hierarchically structured.
- This ADR should be read alongside `docs/physics.md` and the orbital hierarchy implementation in `delta-v-core`/`delta-v-spawn`.