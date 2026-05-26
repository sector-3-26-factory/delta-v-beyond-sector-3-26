# ADR-0018: State management

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

The game has clearly distinct phases: showing a menu, loading a
world, simulating gameplay, paused, displaying a game-over screen.
Mixing the rules of these phases into one set of always-running
systems leads to flag-checking spaghetti.

Bevy provides a first-class `States` mechanism: an enum becomes a
state machine, systems can be scoped to run only in certain states,
transitions trigger `OnEnter` / `OnExit` system sets.

## Decision

State management uses **Bevy `States`**. We define a top-level
application state and, where useful, nested sub-states for game
phases.

Initial sketch (refined as we implement them):

```rust
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Boot,            // initial frame, before any plugin has run setup
    LoadingDefaults, // loading default world & configs
    MainMenu,
    LoadingWorld,
    InGame,
    Paused,
    Shutdown,
}
```

Rules:

- Each plugin scopes its systems to the states it cares about, using
  Bevy's `.run_if(in_state(...))` or system-set scoping.
- Transitions between states happen through explicit events or
  resource changes, not by setting `NextState` from arbitrary
  systems. A small `state_transitions` module documents who triggers
  what.
- The state enum is exhaustive: every state is handled by every
  plugin that cares, and Rust's `match` exhaustiveness checker is
  our friend.
- Sub-states (e.g. `InGameSubstate::FlightAssistOff` /
  `FlightAssistOn`) are introduced only when behaviour actually
  diverges; we do not pre-emptively model every imaginable mode.

The initial development world loads automatically on startup, so the
state path is
`Boot -> LoadingDefaults -> LoadingWorld -> InGame`. The
main menu is introduced when there is a reason for it (later
milestone).

## Consequences

Positive:

- System scheduling is declarative: "this runs in InGame, not in
  the menu" is a one-line annotation.
- Adding a new top-level mode (Multiplayer lobby? Replay viewer?)
  is a new variant plus the systems it needs.
- Tests can drive transitions explicitly without simulating user
  interaction.

Negative:

- A change to the top-level state enum can ripple through many
  plugins. We accept this as the price of having a single source of
  truth.
- Sub-states need discipline; they tempt over-modelling. We resist
  by introducing them on demand only.

Follow-up:

- The list of states is kept short in `docs/architecture.md` for
  quick reference.
- A state-aware logging hook emits an `INFO` line on every state
  transition (per [ADR-0015](0015-logging-strategy.md)).
