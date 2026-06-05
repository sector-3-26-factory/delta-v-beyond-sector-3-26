# ADR-0042: Custom asset loader with two-root search

- **Status**: Withdrawn
- **Date**: 2025-05-30
- **Withdrawn**: 2026-06-04
- **Deciders**: Cute-Donkey
- **Supersedes**: None
- **Relates to**: ADR-0019 (asset pipeline and user content), ADR-0013 (no silent fallbacks)

## Context

This ADR proposed a custom two-root asset loader that would search user content
first, then shipped assets, with a `scope` field to control resolution behavior.

The proposal was evaluated against the current directory structure where templates
and meshes are glued together in unified entity directories
(`assets/templates/ships/<name>/` containing both `template.json` and `mesh.glb`).

## Decision

**Withdrawn.** None of the three proposed use cases are currently needed:

1. **User content overrides** (swapping only the mesh while keeping ship
   characteristics): Not a valid use case. A mesh swap changes the ship's
   physical dimensions, which is inconsistent with unchanged mass/inertia/propulsion.
   A different ship should be a different template, not a mesh override.

2. **Portable worlds / two-root loading**: This is a real concern for future
   multiplayer/world sharing (M7+), but the current single-root Bevy AssetServer
   workaround is sufficient for the current milestone. A new ADR can be written
   when multiplayer architecture is designed.

3. **Scope control** (`user_only` / `shipped_only`): No clear use case. Was an
   AI-generated suggestion without a concrete requirement.

## Rationale

The template-mesh gluing (ADR-0038 entity directory convention) means that a
template and its mesh are a single unit. The two-root search concept was designed
for a world where meshes and templates were separate files that could be
independently overridden. That separation no longer exists.

When multiplayer/world sharing is designed (M7+), a new ADR should address
two-root asset loading in the context of the then-current architecture.
