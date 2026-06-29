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

//! Cockpit-related systems.

use bevy::prelude::*;

use delta_v_core::input::ActionState;
use delta_v_core::{ActiveCameraName, CameraName, CameraSwitched, I18n, PlayerShipEntity};
use delta_v_physics::RigidBody;
use delta_v_types::LogicalAction;

use super::ActiveCockpitStation;
use super::components::CockpitOverlay;
use super::components::SpeedText;
use super::components::VelocityVectorIndicator;
use super::velocity_indicator::create_thrust_arrow_presets;
use super::velocity_indicator::format_speed;

use super::spawn::CockpitOverlayResource;

/// Tracks which actions were already consumed to prevent repeated firing.
// allow-default: Bevy requires Default on resources for init_resource. This
// resource tracks key press state for edge detection; it starts false.
#[derive(Resource, Default)]
pub struct CockpitCycleState {
    /// Whether `CockpitCycleNext` was active last tick.
    next_active: bool,
    /// Whether `CockpitCyclePrev` was active last tick.
    prev_active: bool,
}

/// Cycles to the next or previous cockpit station when the player presses the key.
///
/// Runs in `Update` during `AppState::InGame`.
/// Reads `ActionState<LogicalAction>` for `CockpitCycleNext` or `CockpitCyclePrev`.
/// Cycles through stations in JSON order (wrapping around), loading the new station's PNG texture.
///
/// Uses edge detection to fire only once per key press, not every frame while held.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn cockpit_station_cycle_system(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    cockpit: Res<'_, CockpitOverlayResource>,
    mut active_station: ResMut<'_, ActiveCockpitStation>,
    query: Query<'_, '_, (Entity, &'static Children), With<CockpitOverlay>>,
    mut image_node_query: Query<'_, '_, &'static mut ImageNode>,
    action_state: Res<'_, ActionState<LogicalAction>>,
    mut cycle_state: ResMut<'_, CockpitCycleState>,
) {
    let next_active = action_state.pressed(&LogicalAction::CockpitCycleNext);
    let prev_active = action_state.pressed(&LogicalAction::CockpitCyclePrev);

    // Edge detection: only fire on the frame the key is first pressed
    let direction: i32 = if next_active && !cycle_state.next_active {
        1
    } else if prev_active && !cycle_state.prev_active {
        -1
    } else {
        cycle_state.next_active = next_active;
        cycle_state.prev_active = prev_active;
        return;
    };

    cycle_state.next_active = next_active;
    cycle_state.prev_active = prev_active;

    let Ok((entity, children)) = query.single() else {
        tracing::warn!("[cockpit] cycle: no CockpitOverlay entity found");
        return;
    };

    let current_idx = cockpit
        .stations
        .iter()
        .position(|s| s.id == active_station.station_id)
        .unwrap_or(0);

    let station_count = cockpit.stations.len();
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let new_idx = ((current_idx as i32 + direction).rem_euclid(station_count as i32)) as usize;
    let Some(new_station) = cockpit.stations.get(new_idx) else {
        return;
    };

    let texture_path = format!("{}/{}", cockpit.template_path, new_station.texture);
    let texture_handle = asset_server.load(&texture_path);

    for child in children {
        if let Ok(mut image_node) = image_node_query.get_mut(*child) {
            image_node.image = texture_handle.clone();
        }
    }

    commands.entity(entity).insert(CockpitOverlay {
        texture: texture_handle,
    });

    active_station.station_id.clone_from(&new_station.id);

    tracing::debug!(
        "[cockpit] switched to station '{}' ({})",
        new_station.id,
        new_station.texture
    );
}

/// Toggles the visibility of the cockpit overlay based on camera switch events.
///
/// The cockpit overlay should only be visible when the ship camera "cockpit" is chosen.
/// This system listens for `CameraSwitched` messages and updates visibility accordingly.
///
/// Runs in `Update` during `AppState::InGame`.
pub fn cockpit_visibility_system(
    mut events: MessageReader<'_, '_, CameraSwitched>,
    mut query: Query<'_, '_, (&mut Visibility, &Children), With<CockpitOverlay>>,
    mut commands: Commands<'_, '_>,
) {
    let events: Vec<_> = events.read().collect();
    if events.is_empty() {
        return;
    }

    tracing::debug!("[cockpit] received {} camera switch events", events.len());

    for event in events {
        let is_cockpit_active = event.camera_name == "cockpit";
        let new_visibility = if is_cockpit_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        for (mut visibility, children) in &mut query {
            if *visibility != new_visibility {
                *visibility = new_visibility;
                for child in children {
                    commands.entity(*child).insert(new_visibility);
                }
                tracing::debug!(
                    "[cockpit] visibility set to {:?} (active camera: '{}')",
                    new_visibility,
                    event.camera_name
                );
            }
        }
    }
}

/// Cached preset arrow texture handles for different thrust levels.
#[derive(Resource, Default)]
pub struct ArrowTextureCache {
    /// Preset handles keyed by (level, direction).
    /// direction: "forward" (green), "backward" (red), "white" (no thrust).
    handles:
        std::collections::HashMap<(usize, &'static str), bevy::asset::Handle<bevy::image::Image>>,
}

impl ArrowTextureCache {
    /// Gets a cached handle for the given level and direction.
    pub fn get(
        &self,
        level: usize,
        direction: &'static str,
    ) -> Option<&bevy::asset::Handle<bevy::image::Image>> {
        self.handles.get(&(level, direction))
    }

    /// Inserts a handle for the given level and direction.
    pub fn insert(
        &mut self,
        level: usize,
        direction: &'static str,
        handle: bevy::asset::Handle<bevy::image::Image>,
    ) {
        self.handles.insert((level, direction), handle);
    }
}

/// Speed threshold below which the velocity vector indicator is hidden.
const MIN_SPEED_THRESHOLD: f32 = 0.1;

/// Updates the velocity vector indicator (sprite + speed text).
///
/// The indicator is a `Sprite` on the `Ui` render layer, centered on screen.
/// The system sets `Transform::rotation` to rotate the arrow to point in the
/// velocity direction relative to the active camera's view plane.
/// A child `Text` entity displays the formatted speed.
///
/// `Sprite` is not a layout element, so the UI layout system does not modify
/// its `Transform`. The rotation and scale are preserved.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::collapsible_if
)]
pub fn velocity_vector_system(
    active_camera: Res<'_, ActiveCameraName>,
    camera_query: Query<'_, '_, (&CameraName, &'static GlobalTransform), With<Camera3d>>,
    player_ship: Res<'_, PlayerShipEntity>,
    ship_query: Query<'_, '_, &'static RigidBody>,
    i18n: Res<'_, I18n>,
    action_state: Res<'_, ActionState<LogicalAction>>,
    asset_server: Res<'_, AssetServer>,
    mut cache: ResMut<'_, ArrowTextureCache>,
    mut indicator_query: Query<
        '_,
        '_,
        (&mut Transform, &mut Visibility, &mut Sprite),
        (With<VelocityVectorIndicator>, Without<SpeedText>),
    >,
    mut text_query: Query<
        '_,
        '_,
        (&'static mut Text, &'static mut Visibility),
        (With<SpeedText>, Without<VelocityVectorIndicator>),
    >,
) {
    let _span = tracing::info_span!("delta_v_ships::velocity_vector_system").entered();
    let Ok(ship_body) = ship_query.get(player_ship.0) else {
        return;
    };

    let speed = ship_body.velocity.length();

    let (mut transform, mut visibility, mut sprite) = match indicator_query.single_mut() {
        Ok(result) => result,
        Err(e) => {
            tracing::debug!("[vvi] indicator query failed: {e}");
            return;
        }
    };

    if speed < MIN_SPEED_THRESHOLD {
        if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
        if let Ok((_, mut text_visibility)) = text_query.single_mut() {
            if *text_visibility != Visibility::Hidden {
                *text_visibility = Visibility::Hidden;
            }
        }
        return;
    }

    if *visibility != Visibility::Visible {
        *visibility = Visibility::Visible;
    }

    // Find the active camera's orientation.
    let mut camera_forward = Vec3::NEG_Z;
    let mut camera_right = Vec3::X;
    for (name, gtransform) in &camera_query {
        if name.0 == active_camera.0 {
            camera_forward = gtransform.forward().as_vec3();
            camera_right = gtransform.right().as_vec3();
            break;
        }
    }

    // Project velocity onto camera basis.
    let vel_forward = ship_body.velocity.dot(camera_forward);
    let vel_right = ship_body.velocity.dot(camera_right);

    // Compute rotation angle around Z axis (screen-space rotation).
    // The arrow texture points "up" by default.
    // Negate vel_right so the arrow points in the correct direction.
    let angle = (-vel_right).atan2(vel_forward);
    transform.rotation = Quat::from_rotation_z(angle);

    // Scale based on speed.
    let scale = (speed / 300.0).mul_add(1.5, 0.5);
    transform.scale = Vec3::new(scale, scale, scale);

    // Update arrow texture based on thrust using preset textures.
    // Use ActionState to detect thrust keys directly (ThrustCommand is cleared before Update).
    let forward_pressed = action_state.pressed(&LogicalAction::ThrustForward);
    let backward_pressed = action_state.pressed(&LogicalAction::ThrustBackward);
    let (thrust_ratio, is_forward) = if forward_pressed {
        (1.0, true)
    } else if backward_pressed {
        (1.0, false)
    } else {
        (0.0, true)
    };
    if thrust_ratio > 0.0 {
        let level = (thrust_ratio * 10.0f32).round() as usize;
        let cache_key = if is_forward { "forward" } else { "backward" };
        if let Some(handle) = cache.get(level, cache_key) {
            sprite.image = handle.clone();
        } else {
            let color = if is_forward { [0, 255, 0] } else { [255, 0, 0] };
            let presets = create_thrust_arrow_presets(color);
            let handle = asset_server.add(presets[level].clone());
            cache.insert(level, cache_key, handle.clone());
            sprite.image = handle;
        }
    } else {
        // No thrust: use default white arrow (level 0).
        if let Some(handle) = cache.get(0, "white") {
            sprite.image = handle.clone();
        } else {
            let new_image = super::velocity_indicator::create_arrow_image();
            let handle = asset_server.add(new_image);
            cache.insert(0, "white", handle.clone());
            sprite.image = handle;
        }
    }

    // Update speed text.
    let decimal_sep = i18n
        .number_format
        .decimal_separator
        .chars()
        .next()
        .unwrap_or('.');
    let thousands_sep = i18n
        .number_format
        .thousands_separator
        .chars()
        .next()
        .unwrap_or(',');
    let speed_text = format_speed(
        speed,
        decimal_sep,
        thousands_sep,
        &i18n.speed.unit_ms,
        &i18n.speed.unit_kmh,
        &i18n.speed.unit_kms,
        &i18n.speed.unit_pch,
        &i18n.speed.unit_c,
    );

    // Update the speed text entity.
    if let Ok((mut text, mut text_visibility)) = text_query.single_mut() {
        if *text_visibility != Visibility::Visible {
            *text_visibility = Visibility::Visible;
        }
        text.0 = speed_text;
    }

    tracing::debug!(
        "[vvi] vel=({:.1},{:.1},{:.1}) speed={:.1} m/s angle={:.3} scale={:.2}",
        ship_body.velocity.x,
        ship_body.velocity.y,
        ship_body.velocity.z,
        speed,
        angle,
        scale,
    );
}
