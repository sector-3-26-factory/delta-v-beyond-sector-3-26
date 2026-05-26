# ADR-0011: Keybindings configuration

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Keybindings are the first user-facing configuration the game needs
and a concrete test case for [ADR-0010](0010-configuration-system.md).
Players want different bindings; left-handed players want different
defaults; mouse, keyboard, joystick, gamepad each need separate
schemes; localised keyboard layouts add another dimension.

## Decision

Default keybindings live at:

```
assets/config/keybindings.json
```

User overrides live at:

```
$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/keybindings.json
```

with platform-appropriate equivalents on Windows and macOS.

The file describes **logical actions** (e.g. `thrust_forward`,
`pitch_up`) and binds them to physical inputs (keyboard scancodes,
mouse buttons, joystick axes). Gameplay code never refers to keys
directly; it only reads logical actions.

Multiple input devices coexist:

- Each action may bind to several inputs of different kinds. For
  example, `thrust_forward` can map to keyboard `W`, gamepad
  right-trigger and joystick throttle simultaneously.
- A separate calibration block, also in JSON, holds joystick dead
  zones, axis inversions and ranges. Defaults are conservative; the
  user layer overrides per device.

Loading and merging follow [ADR-0010](0010-configuration-system.md).
With `--features dev`, edits to either file take effect without
restart (per [ADR-0035](0035-hot-reload-of-configs.md)).

The schema for the keybindings file lives at:

```
assets/json/schema/keybindings.schema.json
```

It is the canonical source of the set of logical actions, allowed
input kinds and the default key map. Per
[ADR-0013](0013-no-silent-fallbacks.md), a missing or invalid
defaults file is a hard error; missing user file is fine.

In-game rebinding UI is deferred to a later milestone; until then,
users edit the JSON by hand, which is acceptable because the file is
small, schema-validated and well-documented.

## Consequences

Positive:

- Gameplay code is decoupled from physical input. Adding gamepad
  support later does not touch gameplay systems.
- Players can change bindings without recompiling.
- The schema doubles as documentation of available actions.

Negative:

- One more file format to maintain.
- Hand-editing JSON is friction for non-technical players until the
  in-game UI exists. Mitigated by the fact that the file is small
  and the schema explains every field.

Follow-up:

- An in-game rebinding UI is a future milestone task; once present,
  it writes the full merged state to the user file (see
  [ADR-0010](0010-configuration-system.md)).
- Localisation of action names for the UI is part of the strings
  configuration, not the keybindings file.
