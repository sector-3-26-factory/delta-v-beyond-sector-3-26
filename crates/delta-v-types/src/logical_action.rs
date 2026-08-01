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
    /// Select weapon slot 1.
    SelectWeapon1,
    /// Select weapon slot 2.
    SelectWeapon2,
    /// Select weapon slot 3.
    SelectWeapon3,
    /// Select weapon slot 4.
    SelectWeapon4,
    /// Select weapon slot 5.
    SelectWeapon5,
    /// Select weapon slot 6.
    SelectWeapon6,
    /// Select weapon slot 7.
    SelectWeapon7,
    /// Select weapon slot 8.
    SelectWeapon8,
    /// Select weapon slot 9.
    SelectWeapon9,
    /// Select weapon slot 10 (key 0).
    SelectWeapon10,
    /// Select propulsion slot 1.
    SelectPropulsion1,
    /// Select propulsion slot 2.
    SelectPropulsion2,
    /// Select propulsion slot 3.
    SelectPropulsion3,
    /// Select propulsion slot 4.
    SelectPropulsion4,
    /// Select propulsion slot 5.
    SelectPropulsion5,
    /// Select propulsion slot 6.
    SelectPropulsion6,
    /// Select propulsion slot 7.
    SelectPropulsion7,
    /// Select propulsion slot 8.
    SelectPropulsion8,
    /// Select propulsion slot 9.
    SelectPropulsion9,
    /// Select propulsion slot 10 (key 0).
    SelectPropulsion10,
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
            Self::SelectWeapon1 => "select_weapon_1",
            Self::SelectWeapon2 => "select_weapon_2",
            Self::SelectWeapon3 => "select_weapon_3",
            Self::SelectWeapon4 => "select_weapon_4",
            Self::SelectWeapon5 => "select_weapon_5",
            Self::SelectWeapon6 => "select_weapon_6",
            Self::SelectWeapon7 => "select_weapon_7",
            Self::SelectWeapon8 => "select_weapon_8",
            Self::SelectWeapon9 => "select_weapon_9",
            Self::SelectWeapon10 => "select_weapon_10",
            Self::SelectPropulsion1 => "select_propulsion_1",
            Self::SelectPropulsion2 => "select_propulsion_2",
            Self::SelectPropulsion3 => "select_propulsion_3",
            Self::SelectPropulsion4 => "select_propulsion_4",
            Self::SelectPropulsion5 => "select_propulsion_5",
            Self::SelectPropulsion6 => "select_propulsion_6",
            Self::SelectPropulsion7 => "select_propulsion_7",
            Self::SelectPropulsion8 => "select_propulsion_8",
            Self::SelectPropulsion9 => "select_propulsion_9",
            Self::SelectPropulsion10 => "select_propulsion_10",
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
            Self::SelectWeapon1,
            Self::SelectWeapon2,
            Self::SelectWeapon3,
            Self::SelectWeapon4,
            Self::SelectWeapon5,
            Self::SelectWeapon6,
            Self::SelectWeapon7,
            Self::SelectWeapon8,
            Self::SelectWeapon9,
            Self::SelectWeapon10,
            Self::SelectPropulsion1,
            Self::SelectPropulsion2,
            Self::SelectPropulsion3,
            Self::SelectPropulsion4,
            Self::SelectPropulsion5,
            Self::SelectPropulsion6,
            Self::SelectPropulsion7,
            Self::SelectPropulsion8,
            Self::SelectPropulsion9,
            Self::SelectPropulsion10,
        ]
    }
}

#[cfg(test)]
#[path = "logical_action_tests.rs"]
mod logical_action_tests;
