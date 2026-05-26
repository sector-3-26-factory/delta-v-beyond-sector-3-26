# ADR-0005: Plugin architecture

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Bevy already provides a plugin system: a `Plugin` is a unit of
registration that adds systems, resources, events and sub-plugins to a
Bevy `App`. We need a project convention for how to use it consistently
across our workspace.

Without convention, one ends up with a single mega-plugin that knows
about everything, or with cross-cutting registration scattered across
the binary.

## Decision

**Each domain crate exports exactly one Bevy `Plugin` (or a
`PluginGroup` if it contains sub-plugins).** Registration discipline:

1. The plugin type is named after the crate's domain:
   `PhysicsPlugin`, `ShipsPlugin`, `WeaponsPlugin`, ...
2. The plugin is the *only* `pub` API surface a domain crate needs to
   expose for App composition. Other public items (components,
   resources, events) are exposed only for cross-crate use, never to
   bypass the plugin's setup.
3. The plugin registers all systems, resources, events and asset
   loaders that belong to its domain. It does **not** register
   anything that belongs to another domain.
4. The binary (`crates/delta-v`) is the single place that composes the
   full set of plugins. Composition order is explicit and documented.
5. Plugins must be safe to add to a minimal `App` for tests; they do
   not assume the presence of plugins they do not declare as
   dependencies (via Bevy's `App::is_plugin_added` or simple
   documentation, depending on need).
6. Cross-domain communication uses **events** or shared **components**,
   not function calls between plugin internals.

## Consequences

Positive:

- Each domain is self-contained and testable in isolation by adding
  only its plugin to a Bevy `App`.
- The binary's `App::new()...add_plugins(...)` chain is a readable
  table of contents for the whole game.
- Removing or replacing a domain is a one-line change at the binary
  level.

Negative:

- Some boilerplate per crate (a plugin struct with an empty `build()`
  for stubs).
- The discipline of "no cross-domain function calls" must be enforced
  by review; the compiler will not stop a determined developer.

Follow-up:

- The composition site in `crates/delta-v` is documented and kept in
  a stable order: technical plugins first, then domain plugins.
- A diagnostic plugin (FPS, frame time) is added per
  [ADR-0022](0022-performance-instrumentation.md).
