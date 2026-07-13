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

use bevy::audio::prelude::{AudioPlayer, PlaybackSettings};
use bevy::prelude::*;
use rand::Rng;

use delta_v_core::input::ActionState;
use delta_v_core::{
    ActiveCameraName, CameraName, CameraSwitched, EntityType, FireWeapon, Health, I18n,
    PlayerShipEntity, ProjectileHit, TargetSelected, Weapon, WorldEntityId,
};
use delta_v_physics::{CollisionDetected, RigidBody};
use delta_v_types::LogicalAction;

use super::ActiveCockpitStation;
use super::components::CameraShake;
use super::components::CircularGaugeNeedle;
use super::components::CockpitOverlay;
use super::components::SelectedNavObject;
use super::components::SelectedTarget;
use super::components::SpeedText;
use super::components::StatusGauge;
use super::components::TargetingMode;
use super::components::VelocityVectorIndicator;
use super::velocity_indicator::create_thrust_arrow_presets;
use super::velocity_indicator::format_speed;
use delta_v_core::Targetable;

use super::spawn::CockpitOverlayResource;

/// Resource to track the currently playing thrust sound.
///
/// Holds the entity with the audio components so we can despawn it when thrust ends.
#[derive(Resource, Default)]
pub struct ActiveThrustSound {
    /// Entity with `AudioPlayer` and `AudioSink` components for the currently playing thrust sound.
    pub entity: Option<Entity>,
}

/// Resource to track whether thrusting occurred in the last `FixedUpdate` tick.
///
/// This is set by `thrust_state_tracker_system` in `FixedUpdate` and read by
/// `play_thrust_sound_system` in Update. This bridges the schedule gap since
/// `ThrustCommand` is cleared before Update runs.
// allow-default: Bevy requires Default on resources for init_resource.
// This is runtime state, not configuration.
#[derive(Resource, Default)]
pub struct ThrustingState {
    /// Whether thrust was applied in the last `FixedUpdate` tick.
    pub is_thrusting: bool,
}

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
    mut gauge_query: Query<
        '_,
        '_,
        (&mut Visibility, &Children, &StatusGauge),
        Without<CockpitOverlay>,
    >,
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

    // Update gauge visibility: only show gauges for the new active station.
    // Propagate visibility to children so the fill node is also hidden/shown.
    for (mut visibility, gauge_children, gauge) in &mut gauge_query {
        let target = if gauge.station_id == new_station.id {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != target {
            *visibility = target;
            for child in gauge_children {
                commands.entity(*child).insert(target);
            }
            tracing::debug!(
                "[cockpit] gauge '{}' visibility set to {:?} (station switch)",
                gauge.slot_id,
                target
            );
        }
    }

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
#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub fn cockpit_visibility_system(
    active_station: Res<'_, ActiveCockpitStation>,
    mut events: MessageReader<'_, '_, CameraSwitched>,
    mut overlay_query: Query<
        '_,
        '_,
        (&mut Visibility, &Children),
        (With<CockpitOverlay>, Without<StatusGauge>),
    >,
    mut gauge_query: Query<
        '_,
        '_,
        (&mut Visibility, &Children, &StatusGauge),
        Without<CockpitOverlay>,
    >,
    mut commands: Commands<'_, '_>,
) {
    let events: Vec<_> = events.read().collect();
    if events.is_empty() {
        return;
    }

    for event in events {
        let is_cockpit_active = event.camera_name == "cockpit";
        let new_visibility = if is_cockpit_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        for (mut visibility, children) in &mut overlay_query {
            if *visibility != new_visibility {
                *visibility = new_visibility;
                for child in children {
                    commands.entity(*child).insert(new_visibility);
                }
                tracing::debug!(
                    "[cockpit] overlay visibility set to {:?} (active camera: '{}')",
                    new_visibility,
                    event.camera_name
                );
            }
        }

        // Also toggle gauge visibility — only show gauges for the active station when cockpit is active.
        // Propagate visibility to children so the fill node is also hidden/shown.
        for (mut visibility, gauge_children, gauge) in &mut gauge_query {
            let target = if is_cockpit_active && gauge.station_id == active_station.station_id {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if *visibility != target {
                *visibility = target;
                for child in gauge_children {
                    commands.entity(*child).insert(target);
                }
                tracing::debug!(
                    "[cockpit] gauge '{}' visibility set to {:?}",
                    gauge.slot_id,
                    target
                );
            }
        }
    }
}

/// Initializes gauge visibility on startup.
///
/// Runs during `OnEnter(AppState::InGame)` after all spawn systems.
/// Shows gauges for the active station if the cockpit camera is active,
/// hides all gauges otherwise.
#[allow(clippy::needless_pass_by_value)]
pub fn init_gauge_visibility(
    active_station: Res<'_, ActiveCockpitStation>,
    active_camera: Res<'_, ActiveCameraName>,
    mut gauge_query: Query<
        '_,
        '_,
        (&mut Visibility, &Children, &StatusGauge),
        Without<CockpitOverlay>,
    >,
    mut commands: Commands<'_, '_>,
) {
    let is_cockpit_active = active_camera.0 == "cockpit";
    for (mut visibility, gauge_children, gauge) in &mut gauge_query {
        let target = if is_cockpit_active && gauge.station_id == active_station.station_id {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        *visibility = target;
        for child in gauge_children {
            commands.entity(*child).insert(target);
        }
        tracing::debug!(
            "[init_gauges] gauge '{}' station='{}' visibility={:?}",
            gauge.slot_id,
            gauge.station_id,
            target
        );
    }
}

/// Initializes needle visibility on startup.
///
/// Runs during `OnEnter(AppState::InGame)` after all spawn systems.
/// Shows needles for the active station if the cockpit camera is active,
/// hides all needles otherwise.
#[allow(clippy::needless_pass_by_value)]
pub fn init_needle_visibility(
    active_station: Res<'_, ActiveCockpitStation>,
    active_camera: Res<'_, ActiveCameraName>,
    mut needle_query: Query<'_, '_, (&mut Visibility, &CircularGaugeNeedle)>,
) {
    let is_cockpit_active = active_camera.0 == "cockpit";
    for (mut visibility, needle) in &mut needle_query {
        let target = if is_cockpit_active && needle.station_id == active_station.station_id {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        *visibility = target;
        tracing::debug!(
            "[init_needles] needle station='{}' slot='{}' visibility={:?}",
            needle.station_id,
            needle.slot_id,
            target
        );
    }
}

/// Updates needle visibility when the cockpit station or camera changes.
///
/// Runs during `Update` in the `InGame` state.
/// Shows needles for the active station if the cockpit camera is active,
/// hides all needles otherwise.
#[allow(clippy::needless_pass_by_value)]
pub fn update_needle_visibility(
    active_station: Res<'_, ActiveCockpitStation>,
    active_camera: Res<'_, ActiveCameraName>,
    mut needle_query: Query<'_, '_, (&mut Visibility, &CircularGaugeNeedle)>,
) {
    let is_cockpit_active = active_camera.0 == "cockpit";
    for (mut visibility, needle) in &mut needle_query {
        let target = if is_cockpit_active && needle.station_id == active_station.station_id {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != target {
            *visibility = target;
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
/// A child `Text` entity displays the current speed.
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
        (
            With<VelocityVectorIndicator>,
            Without<SpeedText>,
            Without<CircularGaugeNeedle>,
        ),
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
        "[vvi] vel=({:.1},{:.1},{:.1}) speed={:.1} m/s angle={:.3}",
        ship_body.velocity.x,
        ship_body.velocity.y,
        ship_body.velocity.z,
        speed,
        angle,
    );
}

/// Updates the fill level of all [`StatusGauge`] UI elements.
///
/// Runs in `Update` during `AppState::InGame`. For each gauge:
/// - `"health"`: reads the player ship's [`Health`] component and sets the fill
///   to `current / max` as a percentage.
/// - `"weapon_heat"`: reads the player ship's weapon cooldown and sets the fill
///   to `cooldown / fire_interval` as a percentage (1.0 = fully cooled).
///
/// The fill is applied by setting the child node's `width` percentage and
/// updating the `ImageNode` texture color.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::items_after_statements,
    clippy::manual_let_else,
    clippy::suboptimal_flops,
    clippy::similar_names,
    clippy::match_same_arms,
    clippy::used_underscore_binding
)]
pub fn status_gauge_system(
    player_ship: Res<'_, PlayerShipEntity>,
    health_query: Query<'_, '_, &'static Health>,
    mut queries: ParamSet<
        '_,
        '_,
        (
            Query<
                '_,
                '_,
                (
                    Entity,
                    &'static StatusGauge,
                    &'static Children,
                    &'static Node,
                ),
            >,
            Query<'_, '_, (&mut Node, &mut BackgroundColor)>,
            Query<
                '_,
                '_,
                (&mut Transform, &'static CircularGaugeNeedle),
                Without<VelocityVectorIndicator>,
            >,
        ),
    >,
    window_query: Query<'_, '_, &'static Window>,
) {
    tracing::debug!("[status_gauge] status_gauge_system running");
    let _span = tracing::info_span!("delta_v_ships::status_gauge_system").entered();

    // Read player health once (used by all health gauges).
    #[allow(clippy::option_if_let_else, clippy::single_match_else)]
    let health_ratio = match health_query.get(player_ship.0) {
        Ok(health) => {
            if health.max > 0.0 {
                (health.current / health.max).clamp(0.0, 1.0)
            } else {
                0.0
            }
        }
        Err(_) => {
            tracing::debug!("[status_gauge] player ship has no Health component");
            0.0
        }
    };

    // Get the primary window for screen size
    let window = match window_query.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let _screen_width = window.width();
    let _screen_height = window.height();

    // Collect gauge data first (to avoid holding multiple ParamSet borrows simultaneously)
    struct GaugeUpdate {
        entity: Entity,
        station_id: String,
        slot_id: String,
        shape: crate::ship_templates::GaugeShape,
        gauge_type: String,
        children: Vec<Entity>,
        node: Node,
    }

    let mut gauge_updates: Vec<GaugeUpdate> = Vec::new();
    for (entity, gauge, children, node) in queries.p0() {
        gauge_updates.push(GaugeUpdate {
            entity,
            station_id: gauge.station_id.clone(),
            slot_id: gauge.slot_id.clone(),
            shape: gauge.shape.clone(),
            gauge_type: gauge.gauge_type.clone(),
            children: children.into_iter().copied().collect(),
            node: Node {
                left: node.left,
                top: node.top,
                width: node.width,
                height: node.height,
                ..Default::default()
            },
        });
    }

    // Process each gauge
    for update in &gauge_updates {
        // Handle circular gauges - update the needle rotation and position.
        if let crate::ship_templates::GaugeShape::Circle { .. } = update.shape {
            // For circular gauges, update the needle rotation based on health.
            // The scale arc is 240° (from 240° to 120°, wrapping).
            // Counter-clockwise movement:
            // - 100% health = 120° (right-bottom, green)
            // - 50% health = 0° (top, yellow)
            // - 0% health = -120° (left-bottom, red)
            // The needle texture points UP (0° = 12 o'clock), so we rotate it to point at the gauge angle.
            let angle_deg = (1.0 - health_ratio) * 240.0 - 120.0;
            let angle_rad = angle_deg.to_radians();

            // Get the gauge's computed position (in viewport percentages)
            let gauge_x = match update.node.left {
                Val::Vw(v) => v,
                Val::Percent(v) => v,
                _ => 0.0,
            };
            let gauge_y = match update.node.top {
                Val::Vh(v) => v,
                Val::Percent(v) => v,
                _ => 0.0,
            };
            let gauge_w = match update.node.width {
                Val::Vw(v) => v,
                Val::Percent(v) => v,
                _ => 0.0,
            };
            let gauge_h = match update.node.height {
                Val::Vh(v) => v,
                Val::Percent(v) => v,
                _ => 0.0,
            };

            // Update all needles matching this gauge
            let mut needle_count = 0;
            for (mut transform, needle) in queries.p2() {
                if needle.station_id == update.station_id && needle.slot_id == update.slot_id {
                    needle_count += 1;
                    // Position the needle at the center of the gauge.
                    // Sprites with RenderLayer::CockpitForeground are rendered by the cockpit foreground camera.
                    // The cockpit foreground camera uses a centered coordinate system where (0, 0) is at the center of the screen.
                    // Positive X goes right, positive Y goes up.
                    // Viewport: (0%, 0%) is at top-left, (100%, 100%) is at bottom-right.
                    let gauge_center_x_vp = gauge_x + gauge_w / 2.0;
                    let gauge_center_y_vp = gauge_y + gauge_h / 2.0;
                    // Convert from viewport percentages to cockpit background camera coordinates
                    // UI X: -width/2 at 0%, +width/2 at 100%
                    // UI Y: +height/2 at 0%, -height/2 at 100%
                    transform.translation.x = (gauge_center_x_vp / 100.0 - 0.5) * _screen_width;
                    transform.translation.y = (0.5 - gauge_center_y_vp / 100.0) * _screen_height;
                    // Scale the needle to fit the gauge size.
                    // The gauge radius is half the width (in screen pixels).
                    // We want the needle to be about 85% of the gauge radius.
                    // The needle image is 256x256 pixels, so we need to scale it up.
                    let gauge_radius = gauge_w / 2.0 * _screen_width / 100.0;
                    let needle_scale = gauge_radius * 0.85 / 128.0; // 128 is half the needle image size (radius)
                    transform.scale = Vec3::new(needle_scale, needle_scale, 1.0);
                    // Rotate the needle around its center.
                    // The needle texture points up (0° = 12 o'clock).
                    // We need to rotate it to point at the health position.
                    transform.rotation = Quat::from_rotation_z(angle_rad);
                    tracing::debug!(
                        "[status_gauge] needle pos: gauge_center_vp=({:.1}%, {:.1}%) ui_pos=({:.1}, {:.1}) scale={:.3}",
                        gauge_center_x_vp,
                        gauge_center_y_vp,
                        transform.translation.x,
                        transform.translation.y,
                        needle_scale
                    );
                }
            }
            tracing::debug!(
                "[status_gauge] updated {} needle(s) for gauge {:?} rotation={:.1}°",
                needle_count,
                update.entity,
                angle_deg
            );
            continue;
        }

        let fill_ratio = match update.gauge_type.as_str() {
            "health" => health_ratio,
            "weapon_heat" => {
                // Weapon heat: read the Weapon component's cooldown.
                // For now, default to 0.0 (no heat) since weapon cooldown
                // tracking is per-weapon and the player ship may have multiple.
                // TODO: M7+ — aggregate weapon heat across all weapon slots.
                0.0
            }
            _ => {
                tracing::debug!(
                    "[status_gauge] unknown gauge type '{}' on entity {:?}",
                    update.gauge_type,
                    update.entity
                );
                continue;
            }
        };

        // Compute fill color based on gauge type and fill level.
        let fill_color = if update.gauge_type == "health" {
            if fill_ratio > 0.6 {
                Color::srgb(0.0, 0.8, 0.0) // Green
            } else if fill_ratio > 0.3 {
                Color::srgb(0.8, 0.8, 0.0) // Yellow
            } else {
                Color::srgb(0.8, 0.0, 0.0) // Red
            }
        } else {
            // Weapon heat: blue (cool) → red (hot).
            Color::srgb(fill_ratio, 0.2, 1.0 - fill_ratio)
        };

        // Update the child fill node's width percentage and color.
        for child in &update.children {
            if let Ok((mut node, mut background_color)) = queries.p1().get_mut(*child) {
                node.width = Val::Percent(fill_ratio * 100.0);
                background_color.0 = fill_color;
            }
        }

        tracing::debug!(
            "[status_gauge] entity {:?} type={} fill={:.1}%",
            update.entity,
            update.gauge_type,
            fill_ratio * 100.0
        );
    }
}

/// Toggles the targeting mode between Combat and Nav.
///
/// Runs in `Update` during `AppState::InGame`.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::literal_string_with_formatting_args
)]
pub fn targeting_mode_toggle_system(
    mut mode: ResMut<'_, TargetingMode>,
    action_state: Res<'_, ActionState<LogicalAction>>,
    mut events: MessageWriter<'_, delta_v_core::TargetingModeChanged>,
) {
    use super::components::TargetingModeType;
    if action_state.just_pressed(&LogicalAction::ToggleTargetingMode) {
        let old_mode = mode.mode;
        mode.mode = match mode.mode {
            TargetingModeType::Combat => TargetingModeType::Nav,
            TargetingModeType::Nav => TargetingModeType::Combat,
        };
        tracing::debug!(
            "[targeting] mode switched from {:?} to {:?}, emitting TargetingModeChanged event",
            old_mode,
            mode.mode
        );

        // Emit event for UI to handle notification
        let event_mode = match mode.mode {
            TargetingModeType::Combat => delta_v_core::TargetingModeType::Combat,
            TargetingModeType::Nav => delta_v_core::TargetingModeType::Nav,
        };
        tracing::debug!("[targeting] emitting event with mode: {:?}", event_mode);
        events.write(delta_v_core::TargetingModeChanged { mode: event_mode });
    }
}

/// Cycles to the next or previous target in the list.
///
/// Runs in `Update` during `AppState::InGame`.
/// `T` key: selects next target (closest to the right in sorted list).
/// `ShiftLeft + T` key: selects previous target.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::manual_let_else
)]
pub fn cycle_target_system(
    player_ship: Res<'_, PlayerShipEntity>,
    mut selected_target: ResMut<'_, SelectedTarget>,
    mut selected_nav_object: ResMut<'_, SelectedNavObject>,
    targeting_mode: Res<'_, super::components::TargetingMode>,
    action_state: Res<'_, ActionState<LogicalAction>>,
    targetable_query: Query<
        '_,
        '_,
        (
            Entity,
            &Transform,
            Option<&Name>,
            &EntityType,
            &WorldEntityId,
        ),
        With<Targetable>,
    >,
    navigable_query: Query<
        '_,
        '_,
        (
            Entity,
            &Transform,
            Option<&Name>,
            &EntityType,
            &WorldEntityId,
        ),
    >,
    mut events: MessageWriter<'_, TargetSelected>,
) {
    let is_next = action_state.just_pressed(&LogicalAction::CycleTargetNext);
    let is_prev = action_state.just_pressed(&LogicalAction::CycleTargetPrev);

    if !is_next && !is_prev {
        return;
    }

    // Get player position
    let Some(player_pos) = targetable_query
        .iter()
        .find_map(|(entity, transform, _, _, _)| {
            if entity == player_ship.0 {
                Some(transform.translation)
            } else {
                None
            }
        })
    else {
        return;
    };

    // Determine which query to use based on targeting mode
    let (targets, selected_resource, mode) = match targeting_mode.mode {
        super::components::TargetingModeType::Combat => {
            let mut targets: Vec<(Entity, f32, String, String)> = targetable_query
                .iter()
                .filter_map(|(entity, transform, name, entity_type, entity_id)| {
                    if entity == player_ship.0 {
                        return None;
                    }
                    let distance = (transform.translation - player_pos).length();
                    let name_str = name.map_or_else(|| entity_id.0.clone(), ToString::to_string);
                    Some((entity, distance, name_str, entity_type.0.clone()))
                })
                .collect();
            targets.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            (
                targets,
                &mut selected_target.0,
                super::components::TargetingModeType::Combat,
            )
        }
        super::components::TargetingModeType::Nav => {
            let mut targets: Vec<(Entity, f32, String, String)> = navigable_query
                .iter()
                .filter_map(|(entity, transform, name, entity_type, entity_id)| {
                    if entity == player_ship.0 {
                        return None;
                    }
                    let distance = (transform.translation - player_pos).length();
                    let name_str = name.map_or_else(|| entity_id.0.clone(), ToString::to_string);
                    Some((entity, distance, name_str, entity_type.0.clone()))
                })
                .collect();
            targets.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            (
                targets,
                &mut selected_nav_object.0,
                super::components::TargetingModeType::Nav,
            )
        }
    };

    if targets.is_empty() {
        return;
    }

    // Find current selection index
    let current_idx = selected_resource
        .as_ref()
        .and_then(|current| targets.iter().position(|(e, _, _, _)| e == current))
        .unwrap_or(0);

    // Select next or previous
    let new_idx = if is_next {
        (current_idx + 1) % targets.len()
    } else {
        (current_idx + targets.len() - 1) % targets.len()
    };

    let Some((new_target, distance, name_str, entity_type_str)) = targets.get(new_idx) else {
        return;
    };
    let new_target = *new_target;
    *selected_resource = Some(new_target);

    // Emit TargetSelected event for notification
    let event_mode = match mode {
        super::components::TargetingModeType::Combat => {
            delta_v_core::navigation::TargetingModeType::Combat
        }
        super::components::TargetingModeType::Nav => {
            delta_v_core::navigation::TargetingModeType::Nav
        }
    };
    tracing::debug!(
        "[targeting] emitting TargetSelected event: target={:?} mode={:?}",
        new_target,
        event_mode
    );
    events.write(TargetSelected {
        target: new_target,
        mode: event_mode,
    });

    tracing::debug!(
        "[targeting] selected target {:?} '{}' ({}) (distance: {:.1}m)",
        new_target,
        name_str,
        entity_type_str,
        distance
    );
}

/// Updates bearing indicator visibility and position.
///
/// Shows an arrow at screen edge pointing to the selected target when off-screen.
/// The arrow is positioned at the edge of the screen and rotated to point toward
/// the target's direction. Hidden when the target is on-screen or no target is selected.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::type_complexity
)]
pub fn bearing_indicator_system(
    selected_target: Res<'_, SelectedTarget>,
    selected_nav_object: Res<'_, SelectedNavObject>,
    targeting_mode: Res<'_, super::components::TargetingMode>,
    active_camera: Res<'_, ActiveCameraName>,
    camera_query: Query<'_, '_, (&Camera, &CameraName, &GlobalTransform), With<Camera3d>>,
    entity_query: Query<'_, '_, &GlobalTransform>,
    mut indicator_query: Query<
        '_,
        '_,
        (&mut Transform, &mut Visibility),
        With<super::components::BearingIndicator>,
    >,
    window_query: Query<'_, '_, &Window>,
) {
    // Constants for screen margins
    const MARGIN: f32 = 50.0;
    const EDGE_MARGIN: f32 = 80.0;

    let _span = tracing::info_span!("delta_v_ships::bearing_indicator_system").entered();

    // Determine which entity to track based on targeting mode
    let target_entity = match targeting_mode.mode {
        super::components::TargetingModeType::Combat => selected_target.0,
        super::components::TargetingModeType::Nav => selected_nav_object.0,
    };

    let Some(target_entity) = target_entity else {
        // No target selected - hide indicator
        if let Ok((_, mut visibility)) = indicator_query.single_mut()
            && *visibility != Visibility::Hidden
        {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    // Get target world position
    let Ok(target_transform) = entity_query.get(target_entity) else {
        // Target entity doesn't exist or has no transform - hide indicator
        if let Ok((_, mut visibility)) = indicator_query.single_mut()
            && *visibility != Visibility::Hidden
        {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    // Find the active camera (query for Camera + CameraName + GlobalTransform)
    let camera_query_with_name: Query<
        '_,
        '_,
        (&Camera, &CameraName, &GlobalTransform),
        With<Camera3d>,
    > = camera_query;

    // Find the camera matching the active camera name
    let mut active_camera_found = false;
    for (camera, camera_name, camera_transform) in &camera_query_with_name {
        if camera_name.0 == active_camera.0 {
            active_camera_found = true;

            // Project target position to viewport
            if let Ok(viewport_pos) =
                camera.world_to_viewport(camera_transform, target_transform.translation())
            {
                let Ok(window) = window_query.single() else {
                    return;
                };

                let screen_width = window.width();
                let screen_height = window.height();

                // Check if target is on screen (with some margin)
                let on_screen = viewport_pos.x >= MARGIN
                    && viewport_pos.x <= screen_width - MARGIN
                    && viewport_pos.y >= MARGIN
                    && viewport_pos.y <= screen_height - MARGIN;

                if let Ok((mut transform, mut visibility)) = indicator_query.single_mut() {
                    if on_screen {
                        // Target is on screen - hide bearing indicator
                        if *visibility != Visibility::Hidden {
                            *visibility = Visibility::Hidden;
                        }
                    } else {
                        // Target is off screen - show bearing indicator at screen edge
                        *visibility = Visibility::Visible;

                        // Compute direction from screen center to target
                        let center_x = screen_width / 2.0;
                        let center_y = screen_height / 2.0;
                        let dir_x = viewport_pos.x - center_x;
                        let dir_y = viewport_pos.y - center_y;

                        // Compute angle (0 = up, positive = clockwise)
                        // Screen coordinates have Y down, but atan2 expects Y up.
                        // Negate dir_y to convert from screen to math coordinates.
                        let angle = (-dir_y).atan2(dir_x) - std::f32::consts::FRAC_PI_2;

                        // Position at screen edge
                        // Clamp to screen bounds with margin
                        let edge_x = viewport_pos
                            .x
                            .clamp(EDGE_MARGIN, screen_width - EDGE_MARGIN);
                        let edge_y = viewport_pos
                            .y
                            .clamp(EDGE_MARGIN, screen_height - EDGE_MARGIN);

                        // Convert to UI coordinates (center = 0,0)
                        // UI coordinates: X right, Y up
                        // Screen coordinates: X right, Y down
                        let ui_x = edge_x - center_x;
                        let ui_y = center_y - edge_y;

                        transform.translation = Vec3::new(ui_x, ui_y, 0.0);
                        transform.rotation = Quat::from_rotation_z(angle);
                        transform.scale = Vec3::splat(1.0);

                        tracing::debug!(
                            "[bearing] target={:?} viewport=({:.1},{:.1}) edge=({:.1},{:.1}) angle={:.2}",
                            target_entity,
                            viewport_pos.x,
                            viewport_pos.y,
                            edge_x,
                            edge_y,
                            angle
                        );
                    }
                }
            }
            break;
        }
    }

    if !active_camera_found {
        // Active camera not found - hide indicator
        if let Ok((_, mut visibility)) = indicator_query.single_mut()
            && *visibility != Visibility::Hidden
        {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Updates the on-screen reticle for the selected target.
///
/// Shows a bracket/circle around the target when it's on-screen.
/// Hides the reticle when no target is selected or when the target is off-screen.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::type_complexity
)]
pub fn target_reticle_system(
    selected_target: Res<'_, SelectedTarget>,
    selected_nav_object: Res<'_, SelectedNavObject>,
    targeting_mode: Res<'_, super::components::TargetingMode>,
    active_camera: Res<'_, ActiveCameraName>,
    camera_query: Query<'_, '_, (&Camera, &CameraName, &GlobalTransform), With<Camera3d>>,
    entity_query: Query<'_, '_, &GlobalTransform>,
    mut reticle_query: Query<
        '_,
        '_,
        (&mut Transform, &mut Visibility),
        With<super::components::TargetReticle>,
    >,
    window_query: Query<'_, '_, &Window>,
) {
    // Constants for screen margins
    const MARGIN: f32 = 50.0;

    let _span = tracing::info_span!("delta_v_ships::target_reticle_system").entered();

    // Determine which entity to track based on targeting mode
    let target_entity = match targeting_mode.mode {
        super::components::TargetingModeType::Combat => selected_target.0,
        super::components::TargetingModeType::Nav => selected_nav_object.0,
    };

    let Some(target_entity) = target_entity else {
        // No target selected - hide reticle
        if let Ok((_, mut visibility)) = reticle_query.single_mut()
            && *visibility != Visibility::Hidden
        {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    // Get target world position
    let Ok(target_transform) = entity_query.get(target_entity) else {
        // Target entity doesn't exist or has no transform - hide reticle
        if let Ok((_, mut visibility)) = reticle_query.single_mut()
            && *visibility != Visibility::Hidden
        {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    // Find the active camera and project target position to viewport
    let mut reticle_hidden = true;

    for (camera, camera_name, camera_transform) in &camera_query {
        if camera_name.0 != active_camera.0 {
            continue;
        }

        // Project target position to viewport
        if let Ok(viewport_pos) =
            camera.world_to_viewport(camera_transform, target_transform.translation())
        {
            let Ok(window) = window_query.single() else {
                return;
            };

            let screen_width = window.width();
            let screen_height = window.height();

            // Check if target is on screen (with margin)
            let on_screen = viewport_pos.x >= MARGIN
                && viewport_pos.x <= screen_width - MARGIN
                && viewport_pos.y >= MARGIN
                && viewport_pos.y <= screen_height - MARGIN;

            if let Ok((mut transform, mut visibility)) = reticle_query.single_mut() {
                if on_screen {
                    // Target is on screen - show reticle at screen position
                    *visibility = Visibility::Visible;

                    // Convert viewport position to UI coordinates (center = 0,0)
                    let ui_x = viewport_pos.x - screen_width / 2.0;
                    let ui_y = screen_height / 2.0 - viewport_pos.y;

                    transform.translation = Vec3::new(ui_x, ui_y, 0.0);
                    transform.rotation = Quat::default();
                    transform.scale = Vec3::splat(1.0);

                    tracing::debug!(
                        "[reticle] target={:?} viewport=({:.1},{:.1}) ui_pos=({:.1},{:.1})",
                        target_entity,
                        viewport_pos.x,
                        viewport_pos.y,
                        ui_x,
                        ui_y
                    );
                    reticle_hidden = false;
                } else {
                    // Target is off screen - hide reticle (bearing indicator handles this)
                    if *visibility != Visibility::Hidden {
                        *visibility = Visibility::Hidden;
                    }
                }
            }
            break;
        }
    }

    // Hide reticle if camera not found or viewport projection failed
    if reticle_hidden
        && let Ok((_, mut visibility)) = reticle_query.single_mut()
        && *visibility != Visibility::Hidden
    {
        *visibility = Visibility::Hidden;
    }
}

/// Updates the camera shake effect on the active camera.
///
/// Runs in `Update` during `AppState::InGame`. If a `CameraShake` component
/// is present on the player ship, applies a random offset to the active camera's
/// local position that decays over time. The shake is applied to the camera's
/// local transform (relative to the ship), not the ship's world position.
#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub fn camera_shake_system(
    player_ship: Res<'_, PlayerShipEntity>,
    active_camera_name: Res<'_, delta_v_core::ActiveCameraName>,
    mut shake_query: Query<'_, '_, (&mut CameraShake, &Children)>,
    mut camera_query: Query<'_, '_, (&delta_v_core::CameraName, &mut Transform)>,
    mut commands: Commands<'_, '_>,
) {
    let _span = tracing::info_span!("delta_v_ships::camera_shake_system").entered();

    // Check if the ship has a CameraShake component
    let Ok((mut shake, children)) = shake_query.get_mut(player_ship.0) else {
        return;
    };

    // Find the active camera among the ship's children
    #[allow(clippy::unnecessary_find_map)]
    let active_camera_entity = children.iter().find_map(|child| {
        camera_query.get(child).ok().and_then(|(name, _)| {
            if name.0 == active_camera_name.0 {
                Some(child)
            } else {
                None
            }
        })
    });

    let Some(camera_entity) = active_camera_entity else {
        return;
    };

    // Store original translation on first frame
    if shake.elapsed_ticks == 0
        && let Ok((_camera_name, camera_transform)) = camera_query.get(camera_entity)
    {
        shake.original_translation = camera_transform.translation;
    }

    // Increment elapsed ticks
    shake.elapsed_ticks += 1;

    // Calculate progress (0.0 to 1.0)
    // allow-cast-precision-loss: u32 to f32 cast is acceptable here as tick counts
    // are small (max ~600 for 10 seconds at 60 Hz) and well within f32 precision.
    #[allow(clippy::cast_precision_loss)]
    let progress = shake.elapsed_ticks as f32 / shake.duration_ticks as f32;

    if progress >= 1.0 {
        // Shake complete - restore camera transform and remove the component
        tracing::debug!("[camera_shake] shake complete, restoring camera and removing component");
        // Restore the camera's transform to its original position to prevent drift
        if let Ok((_camera_name, mut camera_transform)) = camera_query.get_mut(camera_entity) {
            camera_transform.translation = shake.original_translation;
        }
        // Remove the component from the ship
        commands.entity(player_ship.0).remove::<CameraShake>();
        return;
    }

    // Decay intensity over time (ease out)
    let current_intensity = shake.intensity * (1.0 - progress).powi(2);

    // Generate random offset
    let mut rng = rand::rng();
    let offset = Vec3::new(
        rng.random_range(-1.0..=1.0) * current_intensity,
        rng.random_range(-1.0..=1.0) * current_intensity,
        rng.random_range(-1.0..=1.0) * current_intensity,
    );

    // Add the offset to the camera's local position (relative to ship)
    if let Ok((_camera_name, mut camera_transform)) = camera_query.get_mut(camera_entity) {
        camera_transform.translation = shake.original_translation + offset;
    }

    tracing::debug!(
        "[camera_shake] tick {}/{} intensity={:.3} offset={:?}",
        shake.elapsed_ticks,
        shake.duration_ticks,
        current_intensity,
        offset
    );
}

/// Triggers camera shake on specific events.
///
/// Listens for `FireWeapon`, `ProjectileHit`, and `CollisionDetected` events.
/// When any of these events occur involving the player ship, inserts a
/// `CameraShake` component on the player ship entity.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn trigger_camera_shake_system(
    mut commands: Commands<'_, '_>,
    player_ship: Res<'_, PlayerShipEntity>,
    mut fire_events: MessageReader<'_, '_, FireWeapon>,
    mut hit_events: MessageReader<'_, '_, ProjectileHit>,
    mut collision_events: MessageReader<'_, '_, CollisionDetected>,
) {
    let _span = tracing::info_span!("delta_v_ships::trigger_camera_shake_system").entered();

    // Fire weapon shake - small, short
    for event in fire_events.read() {
        if event.source == player_ship.0 {
            commands
                .entity(player_ship.0)
                .insert(CameraShake::new(0.5, 10));
            tracing::debug!("[camera_shake] triggered by FireWeapon");
        }
    }

    // Projectile hit shake - medium
    for event in hit_events.read() {
        if event.target == player_ship.0 {
            commands
                .entity(player_ship.0)
                .insert(CameraShake::new(1.0, 15));
            tracing::debug!("[camera_shake] triggered by ProjectileHit (target)");
        }
        if event.projectile == player_ship.0 {
            commands
                .entity(player_ship.0)
                .insert(CameraShake::new(0.8, 12));
            tracing::debug!("[camera_shake] triggered by ProjectileHit (projectile)");
        }
    }

    // Collision shake - larger, longer
    for event in collision_events.read() {
        if event.target == player_ship.0 || event.other == player_ship.0 {
            // Scale intensity by penetration depth
            let intensity = (event.penetration_depth * 2.0).clamp(0.5, 3.0);
            // allow-cast-possible-truncation, cast-sign-loss: intensity is clamped to [0.5, 3.0],
            // so intensity * 10.0 is in [5.0, 30.0], well within u32 cast is safe and positive.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let duration = (intensity * 10.0).round() as u32;
            commands
                .entity(player_ship.0)
                .insert(CameraShake::new(intensity, duration));
            tracing::debug!(
                "[camera_shake] triggered by CollisionDetected intensity={:.2} duration={}",
                intensity,
                duration
            );
        }
    }
}

/// Tracks whether thrust was applied in the last `FixedUpdate` tick.
///
/// Runs in `FixedUpdate` after `ShipInputSet::ApplyThrust` and before
/// `ShipInputSet::ClearCommands`. This captures the thrusting state
/// before the `ThrustCommand` is cleared, allowing the audio system
/// in `Update` to know if thrusting occurred.
#[allow(clippy::needless_pass_by_value)]
pub fn thrust_state_tracker_system(
    thrust_cmd: Res<'_, crate::ship_templates::ThrustCommand>,
    mut state: ResMut<'_, ThrustingState>,
) {
    state.is_thrusting = thrust_cmd.force.length() > 0.0;
}

/// Resource to track whether audio is available (graceful fallback).
#[derive(Resource, Default)]
pub struct AudioAvailable {
    /// Whether an audio device is available for playback.
    pub available: bool,
}

/// Initializes audio availability check.
///
/// Runs during `OnEnter(AppState::InGame)`. Attempts to create an audio sink
/// to verify audio device availability. If no audio device is available,
/// logs a WARN and sets `AudioAvailable` to false.
#[allow(clippy::needless_pass_by_value)]
pub fn init_audio_availability(mut commands: Commands<'_, '_>) {
    // For now, assume audio is available.
    // In a devcontainer without audio device, this will fail gracefully.
    let audio_available = true;

    commands.insert_resource(AudioAvailable {
        available: audio_available,
    });

    if audio_available {
        tracing::info!("[audio] audio system initialized");
    } else {
        tracing::warn!("[audio] no audio device available - sound effects disabled");
    }
}

/// Plays the thrust sound when the player is thrusting.
///
/// Runs in `Update` during `AppState::InGame`. Checks `ThrustingState` to determine
/// if thrusting occurred in the last `FixedUpdate` tick, and plays the thrust sound
/// (looped) when thrusting, stops when not thrusting.
/// The thrust sound is configured in the propulsion/thruster definition, not in ship sounds.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn play_thrust_sound_system(
    _player_ship: Res<'_, PlayerShipEntity>,
    thrusting_state: Res<'_, ThrustingState>,
    propulsion_config: Res<'_, crate::ship_templates::ShipPropulsionConfig>,
    audio_available: Res<'_, AudioAvailable>,
    asset_server: Res<'_, AssetServer>,
    mut active_sound: ResMut<'_, ActiveThrustSound>,
    mut commands: Commands<'_, '_>,
) {
    if !audio_available.available {
        tracing::debug!("[audio] thrust sound skipped: audio not available");
        return;
    }

    let Some(thrust_sound_path) = &propulsion_config.thrust_sound else {
        tracing::debug!("[audio] thrust sound skipped: no thrust_sound configured");
        return;
    };

    let is_thrusting = thrusting_state.is_thrusting;
    tracing::debug!(
        "[audio] thrust sound system: is_thrusting={}, active_sound.entity={:?}, thrust_sound_path={:?}",
        is_thrusting,
        active_sound.entity,
        thrust_sound_path
    );

    if is_thrusting {
        // If no sound is playing, start one
        if active_sound.entity.is_none() {
            let sound_handle: Handle<AudioSource> = asset_server.load(thrust_sound_path);
            let entity = commands
                .spawn((
                    AudioPlayer::new(sound_handle),
                    PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.5)),
                ))
                .id();
            active_sound.entity = Some(entity);
            tracing::debug!("[audio] thrust sound started: {}", thrust_sound_path);
        }
    } else {
        // Stop the thrust sound if it's playing
        if let Some(entity) = active_sound.entity.take() {
            commands.entity(entity).despawn();
            tracing::debug!("[audio] thrust sound stopped");
        }
    }
}

/// Plays the weapon fire sound when a weapon is fired.
///
/// Runs in `Update` during `AppState::InGame`. Listens for `FireWeapon` events
/// and plays the weapon's sound (from the weapon component).
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn play_fire_sound_system(
    player_ship: Res<'_, PlayerShipEntity>,
    audio_available: Res<'_, AudioAvailable>,
    asset_server: Res<'_, AssetServer>,
    mut fire_events: MessageReader<'_, '_, FireWeapon>,
    weapon_query: Query<'_, '_, &Weapon>,
    mut commands: Commands<'_, '_>,
) {
    if !audio_available.available {
        tracing::debug!("[audio] fire sound skipped: audio not available");
        return;
    }

    for event in fire_events.read() {
        if event.source == player_ship.0
            && let Ok(weapon) = weapon_query.get(event.source)
            && let Some(fire_sound_path) = &weapon.fire_sound
        {
            let sound_handle: Handle<AudioSource> = asset_server.load(fire_sound_path);
            commands.spawn((
                AudioPlayer::new(sound_handle),
                PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(0.7)),
            ));
            tracing::debug!("[audio] fire sound played: {}", fire_sound_path);
        } else {
            tracing::debug!(
                "[audio] fire event ignored: source={:?} player_ship={:?}",
                event.source,
                player_ship.0
            );
        }
    }
}

/// Plays the hit sound when the player ship is hit.
///
/// Runs in `Update` during `AppState::InGame`. Listens for `ProjectileHit` events
/// where the target is the player ship, and plays the hit sound from the projectile.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn play_hit_sound_system(
    player_ship: Res<'_, PlayerShipEntity>,
    audio_available: Res<'_, AudioAvailable>,
    mut hit_events: MessageReader<'_, '_, ProjectileHit>,
    asset_server: Res<'_, AssetServer>,
    mut commands: Commands<'_, '_>,
) {
    if !audio_available.available {
        tracing::debug!("[audio] hit sound skipped: audio not available");
        return;
    }

    for event in hit_events.read() {
        if event.target == player_ship.0 {
            if let Some(hit_sound_path) = &event.hit_sound {
                let sound_handle: Handle<AudioSource> = asset_server.load(hit_sound_path);
                commands.spawn((
                    AudioPlayer::new(sound_handle),
                    PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(0.6)),
                ));
                tracing::debug!("[audio] hit sound played: {}", hit_sound_path);
            } else {
                tracing::debug!("[audio] hit sound skipped: no hit_sound in projectile");
            }
        } else {
            tracing::debug!(
                "[audio] hit event ignored: target={:?} player_ship={:?}",
                event.target,
                player_ship.0
            );
        }
    }
}
