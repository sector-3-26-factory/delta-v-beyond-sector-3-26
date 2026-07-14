# ADR-0054: Player-controlled targeting without auto-switching

- **Status**: Accepted
- **Date**: 2026-07-13
- **Deciders**: AI agent
- **Supersedes**: None
- **Superseded by**: None

## Context

The M6 milestone introduces a targeting and navigation system for the player ship. The system needs to decide how targets are selected and when the selection changes.

In many space sims, auto-targeting is a common feature where the game automatically selects the nearest hostile target or switches targets when the current target is destroyed. However, this can lead to frustration when the player's intended target is unexpectedly deselected.

The project follows ADR-0016 (error handling) and ADR-0013 (no silent fallbacks), which emphasize explicit behavior and clear state management. The targeting system should align with these principles.

## Decision

We use **explicit player selection only** for targeting and navigation. The game never auto-switches targets when a nearer entity appears or when the current target is destroyed.

- The player explicitly selects a target or navigation object via the navigation list UI.
- The `T` key cycles forward through the sorted list of targets.
- `ShiftLeft + T` cycles backward through the list.
- `N` toggles the navigation list visibility.
- `Tab` toggles between Combat and Nav modes.
- When the selected target is destroyed, the `SelectedTarget` resource is cleared and the reticle disappears.
- The navigation list updates to reflect the current state of targetable/navigable entities.

This design follows the principle that the player's intent should be respected and not overridden by the game.

## Consequences

### Positive consequences

- The player maintains control over target selection at all times.
- No unexpected target switching during combat.
- Clear mental model: what you select is what you get.
- Simpler implementation: no need to track "auto-target" state or handle edge cases.

### Negative consequences

- The player must manually re-select a target if the current one is destroyed.
- Slightly more input required for target management.
- The navigation list must be kept open or toggled frequently to manage targets.

### Follow-up work

- None. This decision is complete and implemented.

## Notes

This design decision is documented in the M6 plan under section 5.7 (No auto-targeting — player-controlled selection).