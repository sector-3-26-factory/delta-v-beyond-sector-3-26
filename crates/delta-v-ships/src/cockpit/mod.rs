// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Cockpit overlay system for ship-specific HUD views.
//!
//! This module handles:
//! - Cockpit overlay PNG rendering with alpha transparency
//! - Station switching (F2 / Shift+F2)
//! - Velocity vector indicator (direction of ship movement)
//! - Status gauges (health, weapon heat) with slot-based positioning
//! - Player-controlled targeting & navigation list
//!
//! See M6 -- HUD and Feel plan.

use bevy::prelude::*;
use delta_v_core::AppState;

pub mod components;
pub mod navigation_list;
pub mod spawn;
pub mod systems;
pub mod velocity_indicator;

pub use components::*;
pub use spawn::CockpitOverlayResource;

/// Plugin for cockpit overlay systems.
pub struct CockpitPlugin;

impl Plugin for CockpitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<systems::ArrowTextureCache>()
            .init_resource::<systems::CockpitCycleState>()
            .init_resource::<components::TargetingMode>()
            .init_resource::<components::SelectedTarget>()
            .init_resource::<components::SelectedNavObject>()
            .init_resource::<delta_v_core::NavigationListData>()
            .add_systems(OnEnter(AppState::InGame), spawn::spawn_cockpit_overlay)
            .add_systems(
                OnEnter(AppState::InGame),
                spawn::spawn_velocity_vector_indicator,
            )
            .add_systems(OnEnter(AppState::InGame), spawn::spawn_bearing_indicator)
            .add_systems(OnEnter(AppState::InGame), spawn::spawn_target_reticle)
            .add_systems(
                OnEnter(AppState::InGame),
                spawn::spawn_status_gauges.after(spawn::spawn_cockpit_overlay),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                systems::init_gauge_visibility.after(spawn::spawn_status_gauges),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                systems::init_needle_visibility.after(spawn::spawn_status_gauges),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                navigation_list::init_navigation_list_system.after(systems::init_needle_visibility),
            )
            .add_systems(OnEnter(AppState::InGame), systems::init_audio_availability)
            .add_systems(
                Update,
                (
                    systems::cockpit_station_cycle_system,
                    systems::cockpit_visibility_system,
                    systems::velocity_vector_system,
                    systems::status_gauge_system,
                    systems::update_needle_visibility,
                    systems::targeting_mode_toggle_system,
                    systems::cycle_target_system,
                    systems::bearing_indicator_system,
                    systems::target_reticle_system,
                    systems::camera_shake_system,
                    systems::trigger_camera_shake_system,
                    systems::play_thrust_sound_system,
                    systems::play_fire_sound_system,
                    systems::play_hit_sound_system,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    navigation_list::update_navigation_list_system,
                    navigation_list::update_selection_system,
                    navigation_list::handle_target_selected_system,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                navigation_list::update_navigation_list_distances_system
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
