// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Logical input actions.
//!
//! This module defines the [`LogicalAction`] enum — the canonical set of
//! player-performable actions. The keybindings schema uses these names as
//! string keys (see [`LogicalAction::as_str`]).
//!
//! See ADR-0011 (Keybindings configuration) and ADR-0046 (Shared types crate).

use bevy::prelude::Reflect;
use leafwing_input_manager::prelude::Actionlike;

/// All logical actions the player can perform.
///
/// This enum is the canonical set of actions. The keybindings schema
/// uses these names as string keys (see [`LogicalAction::as_str`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect, Actionlike)]
pub enum LogicalAction {
    /// Thrust along the ship's forward (-Z) axis.
    ThrustForward,
    /// Thrust along the ship's backward (+Z) axis.
    ThrustBackward,
    /// Pitch nose up (rotate around local +X).
    PitchUp,
    /// Pitch nose down (rotate around local -X).
    PitchDown,
    /// Yaw nose left (rotate around local +Y).
    YawLeft,
    /// Yaw nose right (rotate around local -Y).
    YawRight,
    /// Roll left (rotate around local -Z).
    RollLeft,
    /// Roll right (rotate around local +Z).
    RollRight,
    /// Strafe left along local -X.
    StrafeLeft,
    /// Strafe right along local +X.
    StrafeRight,
    /// Strafe up along local +Y.
    StrafeUp,
    /// Strafe down along local -Y.
    StrafeDown,
    /// Toggle flight assist (inertial damping) on/off.
    ToggleFlightAssist,
    /// Fire the primary weapon.
    FirePrimary,
    /// Cycle to the next cockpit station.
    CockpitCycleNext,
    /// Cycle to the previous cockpit station.
    CockpitCyclePrev,
    /// Switch to the next ship camera.
    CameraSwitchNext,
    /// Switch to the previous ship camera.
    CameraSwitchPrev,
    /// Toggle the navigation menu (open/close).
    ToggleNavigationMenu,
    /// Toggle the keybindings menu (open/close).
    ToggleKeybindingsMenu,
    /// Cycle to the next target in the list.
    CycleTargetNext,
    /// Cycle to the previous target in the list.
    CycleTargetPrev,
    /// Toggle between Combat and Nav targeting modes.
    ToggleTargetingMode,
}

impl LogicalAction {
    /// The string key used in `keybindings.json` for this action.
    ///
    /// Must match the `snake_case` key in `assets/config/keybindings.json`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ThrustForward => "thrust_forward",
            Self::ThrustBackward => "thrust_backward",
            Self::PitchUp => "pitch_up",
            Self::PitchDown => "pitch_down",
            Self::YawLeft => "yaw_left",
            Self::YawRight => "yaw_right",
            Self::RollLeft => "roll_left",
            Self::RollRight => "roll_right",
            Self::StrafeLeft => "strafe_left",
            Self::StrafeRight => "strafe_right",
            Self::StrafeUp => "strafe_up",
            Self::StrafeDown => "strafe_down",
            Self::ToggleFlightAssist => "toggle_flight_assist",
            Self::FirePrimary => "fire_primary",
            Self::CockpitCycleNext => "cockpit_cycle_next",
            Self::CockpitCyclePrev => "cockpit_cycle_prev",
            Self::CameraSwitchNext => "camera_switch_next",
            Self::CameraSwitchPrev => "camera_switch_prev",
            Self::ToggleNavigationMenu => "toggle_navigation_menu",
            Self::ToggleKeybindingsMenu => "toggle_keybindings_menu",
            Self::CycleTargetNext => "cycle_target_next",
            Self::CycleTargetPrev => "cycle_target_prev",
            Self::ToggleTargetingMode => "toggle_targeting_mode",
        }
    }

    /// All variants in a stable iteration order.
    ///
    /// Used by the input translation system to iterate without allocation.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::ThrustForward,
            Self::ThrustBackward,
            Self::PitchUp,
            Self::PitchDown,
            Self::YawLeft,
            Self::YawRight,
            Self::RollLeft,
            Self::RollRight,
            Self::StrafeLeft,
            Self::StrafeRight,
            Self::StrafeUp,
            Self::StrafeDown,
            Self::ToggleFlightAssist,
            Self::FirePrimary,
            Self::CockpitCycleNext,
            Self::CockpitCyclePrev,
            Self::CameraSwitchNext,
            Self::CameraSwitchPrev,
            Self::ToggleNavigationMenu,
            Self::ToggleKeybindingsMenu,
            Self::CycleTargetNext,
            Self::CycleTargetPrev,
            Self::ToggleTargetingMode,
        ]
    }
}

#[cfg(test)]
#[path = "logical_action_tests.rs"]
mod logical_action_tests;
