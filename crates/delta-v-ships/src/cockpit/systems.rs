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
use delta_v_core::{ActiveCameraName, CameraName, CameraSwitched, Health, I18n, PlayerShipEntity};
use delta_v_physics::RigidBody;
use delta_v_types::LogicalAction;

use super::ActiveCockpitStation;
use super::components::CircularGaugeNeedle;
use super::components::CockpitOverlay;
use super::components::SelectedTarget;
use super::components::SpeedText;
use super::components::StatusGauge;
use super::components::Targetable;
use super::components::TargetingMode;
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
    mut events: MessageWriter<'_, delta_v_core::events::TargetingModeChanged>,
) {
    use super::components::TargetingModeType;
    if action_state.just_pressed(&LogicalAction::ToggleTargetingMode) {
        mode.mode = match mode.mode {
            TargetingModeType::Combat => TargetingModeType::Nav,
            TargetingModeType::Nav => TargetingModeType::Combat,
        };
        tracing::debug!("[targeting] mode switched to {:?}", mode.mode);

        // Emit event for UI to handle notification
        events.write(delta_v_core::events::TargetingModeChanged {
            mode: match mode.mode {
                TargetingModeType::Combat => delta_v_core::events::TargetingModeType::Combat,
                TargetingModeType::Nav => delta_v_core::events::TargetingModeType::Nav,
            },
        });
    }
}

/// Cycles to the next or previous target in the list.
///
/// Runs in `Update` during `AppState::InGame`.
/// `T` key: selects next target (closest to the right in sorted list).
/// `ShiftLeft + T` key: selects previous target.
#[allow(clippy::needless_pass_by_value)]
pub fn cycle_target_system(
    player_ship: Res<'_, PlayerShipEntity>,
    mut selected_target: ResMut<'_, SelectedTarget>,
    action_state: Res<'_, ActionState<LogicalAction>>,
    query: Query<'_, '_, (Entity, &Transform), With<Targetable>>,
) {
    let is_next = action_state.just_pressed(&LogicalAction::CycleTargetNext);
    let is_prev = action_state.just_pressed(&LogicalAction::CycleTargetPrev);

    if !is_next && !is_prev {
        return;
    }

    // Collect all targetable entities with their distances from player
    let Some(player_pos) = query.iter().find_map(|(entity, transform)| {
        if entity == player_ship.0 {
            Some(transform.translation)
        } else {
            None
        }
    }) else {
        return;
    };

    // Collect all targetable entities (excluding player) with their distances
    let mut targets: Vec<(Entity, f32)> = query
        .iter()
        .filter_map(|(entity, transform)| {
            if entity == player_ship.0 {
                return None;
            }
            let distance = (transform.translation - player_pos).length();
            Some((entity, distance))
        })
        .collect();

    // Sort by distance
    targets.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    if targets.is_empty() {
        return;
    }

    // Find current selection index
    let current_idx = selected_target
        .0
        .and_then(|current| targets.iter().position(|(e, _)| *e == current))
        .unwrap_or(0);

    // Select next or previous
    let new_idx = if is_next {
        (current_idx + 1) % targets.len()
    } else {
        (current_idx + targets.len() - 1) % targets.len()
    };

    let Some((new_target, distance)) = targets.get(new_idx) else {
        return;
    };
    let new_target = *new_target;
    selected_target.0 = Some(new_target);
    tracing::debug!(
        "[targeting] selected target {:?} (distance: {:.1}m)",
        new_target,
        distance
    );
}

/// Updates bearing indicator visibility and position.
///
/// Shows an arrow at screen edge pointing to the selected target when off-screen.
#[allow(clippy::missing_const_for_fn)]
pub fn bearing_indicator_system(
    _selected_target: Res<'_, SelectedTarget>,
    _active_camera: Res<'_, ActiveCameraName>,
) {
    // TODO: Implement bearing indicator logic
    // This requires projecting the target position to screen space
    // and computing the edge position and rotation.
}

/// Updates the on-screen reticle for the selected target.
///
/// Shows a bracket/circle around the target when it's on-screen.
#[allow(clippy::missing_const_for_fn)]
pub fn target_reticle_system(_selected_target: Res<'_, SelectedTarget>) {
    // TODO: Implement reticle logic
    // This requires projecting the target's 3D position to 2D screen coordinates.
}
