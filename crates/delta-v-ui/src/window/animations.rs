// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Window animation types and components.
//!
//! Provides reusable animation infrastructure for window entities. Animations
//! are configured via [`super::spawn::WindowConfig::animation`] and applied
//! automatically when a window is spawned.
//!
//! New animation types are added as variants on the [`WindowAnimation`] enum.

use bevy::prelude::*;

/// Animation component for windows.
///
/// Attach to a window root entity (alongside `WindowRoot`) to enable
/// animation. The [`super::systems::window_animation_system`] reads this
/// component and updates the panel's `BackgroundColor` opacity.
///
/// Windows without this component are not animated -- animation is opt-in
/// via `WindowConfig::animation`. Extensible: add new variants as needed.
#[derive(Component, Clone)]
pub enum WindowAnimation {
    /// Flicker animation: oscillating opacity that converges in, holds, then
    /// converges out.
    Flicker(WindowAnimationFlicker),
}

/// Flicker animation state.
#[derive(Debug, Clone)]
pub struct WindowAnimationFlicker {
    /// Current animation phase.
    pub phase: WindowAnimationFlickerPhase,
    /// Elapsed time within the current phase (seconds).
    pub phase_elapsed: f32,
    /// Duration of the flicker-in phase (seconds).
    pub flicker_in_duration: f32,
    /// Duration of the hold phase (seconds).
    pub hold_duration: f32,
    /// Duration of the flicker-out phase (seconds).
    pub flicker_out_duration: f32,
    /// Number of flicker oscillations during flicker-in/out.
    pub flicker_count: u32,
}

/// Animation phases for the flicker effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowAnimationFlickerPhase {
    /// Flicker in: rapid opacity oscillation converging to full visibility.
    FlickerIn,
    /// Hold: fully visible.
    Hold,
    /// Flicker out: rapid opacity oscillation converging to zero.
    FlickerOut,
    /// Done: entity should be despawned.
    Done,
}

impl WindowAnimationFlicker {
    /// Creates a new flicker animation with the given durations.
    #[must_use]
    pub const fn new(flicker_in: f32, hold: f32, flicker_out: f32, flicker_count: u32) -> Self {
        Self {
            phase: WindowAnimationFlickerPhase::FlickerIn,
            phase_elapsed: 0.0,
            flicker_in_duration: flicker_in,
            hold_duration: hold,
            flicker_out_duration: flicker_out,
            flicker_count,
        }
    }
}

impl WindowAnimation {
    /// Creates a new flicker animation variant.
    #[must_use]
    pub const fn flicker(flicker_in: f32, hold: f32, flicker_out: f32, flicker_count: u32) -> Self {
        Self::Flicker(WindowAnimationFlicker::new(
            flicker_in,
            hold,
            flicker_out,
            flicker_count,
        ))
    }
}
