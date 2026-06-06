# ADR-0045: Anti-cheat strategy for client-side JSON configuration

<!--
Agents and contributors: read AGENTS.md at the repository root before
working with this document. AGENTS.md links to the ADR index and to all
rules that govern this project.
-->

- **Status**: Proposed
- **Date**: 2026-06-06
- **Deciders**: Cute-Donkey
- **Supersedes**: (optional, set when status is `Superseded`)
- **Superseded by**: (optional, set when status is `Superseded`)

## Context

Delta-V Beyond Sector 3.26 uses JSON files for ship configurations, world definitions, and gameplay parameters. Players can customize their ships by modifying JSON files in the user content directory (`$XDG_DATA_HOME/delta-v-beyond-sector-3-26/`). This creates a security vulnerability surface where players can modify client-side data to gain unfair advantages.

### The collision_shape Vulnerability

The most critical vulnerability is the `collision_shape` property in ship templates. A player can modify the collision shape to make their ship invulnerable:

```json
{
    "collision_shape": {
        "type": "box",
        "half_extents": { "x": 1000.0, "y": 1000.0, "z": 1000.0 }
    }
}
```

This creates an enormous collision box that can encompass entire asteroid fields, making the ship effectively indestructible while still appearing visually as a small ship.

### Other Client-Side Cheating Vectors

1. **Ship Statistics Manipulation**
   - Mass: Setting mass to extreme values (0.001 kg or 1e12 kg)
   - Inertia scale: Making ships immovable or overly responsive
   - Bounding box: Mismatched visual vs. collision geometry

2. **Propulsion System Cheating**
   - Thrust values: Infinite thrust or negative values
   - Fuel consumption: Zero consumption rates
   - Maneuvering thrusters: Excessive torque or strafe

3. **Weapon and Combat Stats**
   - Damage values: Instant-kill weapons
   - Fire rates: Unlimited firing
   - Range: Infinite weapon range

4. **Position and Movement**
   - World definition: Spawning inside asteroids or at arbitrary positions
   - Scale: Making ships physically larger than intended

5. **Resource Values**
   - Fuel amounts: Infinite fuel
   - Shield/HP values: Invulnerability
   - Energy reserves: Unlimited power

### Current Architecture Constraints

Per ADR-0030 (Proposed), the multiplayer model is peer-to-peer with "player as server" semantics. The host is the authoritative simulator. This means:

- Anti-cheat cannot rely on a central authoritative server
- The host player has significant power to cheat
- Client-side validation is primarily for single-player and to detect accidental corruption

## Decision

We will implement a **layered defense strategy** combining multiple anti-cheat measures:

### Layer 1: Schema Validation and Value Bounds

Enforce strict bounds on all gameplay values in JSON schemas. Values outside acceptable ranges will fail schema validation.

```json
{
    "mass": {
        "type": "object",
        "properties": {
            "value": {
                "type": "number",
                "exclusiveMinimum": 10.0,
                "maximum": 10000000.0
            }
        }
    }
}
```

### Layer 2: Server-Side Validation (Host Authority)

In multiplayer, the host validates all client-submitted ship configurations against:
- Schema validation (already implemented)
- Reasonableness checks (e.g., collision shape must fit within bounding box)
- Template integrity (no missing required fields)

### Layer 3: Cryptographic Signing (Optional)

For single-player and trusted multiplayer, optionally support cryptographic signing of ship templates:
- Developers sign official templates with a private key
- Game ships with public key embedded
- Unsigned or invalidly signed templates show a warning but are still usable

### Layer 4: Server-Authoritative Game State

Per ADR-0030, the host maintains authoritative state. Clients send inputs, host simulates and broadcasts state. This prevents:
- Position cheating (host validates movement)
- Damage cheating (host calculates damage)
- Resource manipulation (host tracks fuel/ammo)

### Layer 5: Collision Shape Integrity

Implement a validation rule: collision shape must be contained within the bounding box with a safety margin. The loader will compute this relationship and reject templates where the collision shape is disproportionately larger than the visual mesh.

## Consequences

### Positive consequences

- **Prevents trivial cheating**: Players cannot make invincible ships by modifying JSON
- **Maintains single-player flexibility**: Players can still create custom ships within reason
- **Layered approach**: Multiple independent defenses provide defense in depth
- **Performance conscious**: Validation happens at load time, not runtime
- **Clear error messages**: Schema validation provides specific feedback on what's wrong

### Negative consequences

- **Reduced customization freedom**: Some extreme but potentially fun configurations are blocked
- **Complexity**: Multiple validation layers increase code complexity
- **False positives**: Legitimate extreme designs might be rejected

### Follow-up work

- Implement collision shape containment validation in template loader
- Add bounds checking to all gameplay-related schemas
- Create a signing tool for official templates
- Implement host-side validation for multiplayer (when networking is active)

## Decision Matrix

| Approach | Complexity (1-5) | Effectiveness (1-5) | Performance Impact | User Experience |
|----------|------------------|---------------------|-------------------|-----------------|
| Schema bounds only | 2 | 3 | None | Good |
| Server-side validation | 3 | 4 | Low (load time) | Good |
| Cryptographic signing | 4 | 4 | None | Mixed (warning for unsigned) |
| Server-authoritative state | 5 | 5 | Medium (network) | Good |
| Collision shape validation | 2 | 4 | None | Good |

**Recommended combination**: Schema bounds + collision shape validation + server-authoritative state (when multiplayer is implemented).

## Related ADRs

- ADR-0008: Physical quantities in JSON (value+unit objects)
- ADR-0012: JSON Schema validation
- ADR-0013: No silent fallbacks (validation failures are hard errors)
- ADR-0019: Asset pipeline and user content
- ADR-0030: Authoritative model (P2P with host authority)
- ADR-0038: Entity template system
- ADR-0039: Enforcement of JSON-only defaults
- ADR-0040: delta-v-json for JSON validation