# ADR-0053: VFX Exception to No Visual Data in Rust Rule

- **Status**: Accepted
- **Date**: 2026-07-13
- **Deciders**: Project maintainers
- **Supersedes**: ADR-0044
- **Superseded by**: (none)

## Context

ADR-0044 prohibits all visual data handling within Rust code, with specific exceptions for:
1. Debug/diagnostic visual data
2. Camera setup, render layer configuration, and UI rendering infrastructure
3. UI rendering elements (gauge scales, needles, arrow indicators, progress bars, etc.)
4. UI infrastructure types from third-party crates

The current implementation adds muzzle flash and hit VFX (visual effects) to the cockpit systems. These effects:
- Are transient, not persistent game objects
- Are typically simple geometric shapes (sprites, particles)
- Are performance-critical and benefit from procedural generation
- Are not "objects in space" like ships, planets, or stations
- Are spawned in world space, not UI space

The hit flash VFX in `crates/delta-v-ships/src/cockpit/systems.rs` creates a texture procedurally using `create_hit_flash_image()`, which generates a radial gradient image. This is similar to the existing `create_arrow_image()` in `velocity_indicator.rs` which is already exempted as UI rendering infrastructure.

However, the hit flash VFX is spawned in world space (not UI space), so it does not clearly fall under the existing UI rendering infrastructure exception.

## Decision

We add **VFX (visual effects)** as an exception to ADR-0044. This includes:
- Muzzle flash effects (point lights, sprites)
- Hit flash effects (sprites, particles)
- Explosion effects (particles, sprites)
- Other transient visual effects

VFX are exempt from the "no visual data in Rust" rule because:
1. **Transient nature**: VFX are short-lived effects, not persistent assets
2. **Performance**: Procedural VFX avoid asset loading overhead for effects that may spawn frequently
3. **Simplicity**: VFX are typically simple shapes (sprites, gradients) that don't require complex asset files
4. **Not gameplay content**: VFX are feedback effects, not "objects in space" like ships, planets, or stations
5. **Consistency**: This aligns with the existing UI rendering infrastructure exception

## Consequences

### Positive consequences

- **Performance**: VFX can be spawned without asset loading latency
- **Simplicity**: Simple VFX don't require separate asset files
- **Consistency**: Aligns with the existing UI rendering infrastructure exception

### Negative consequences

- **Inconsistency**: VFX in Rust code while other visual data must be external
- **Testing**: Procedural VFX are harder to customize without code changes

### Follow-up work

- Update ARCHITECTURAL_RULES.md to include the VFX exception
- Consider adding a `vfx/` directory under `assets/` for more complex VFX in the future

## Notes

- This exception applies only to simple procedural VFX (sprites, gradients, basic particles)
- Complex VFX (custom textures, complex particle systems) should still use external assets
- See ADR-0044 for the full context of the no-visual-data rule