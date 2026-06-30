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

//! Cockpit overlay spawning systems.

use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;

use crate::ship_templates::CockpitStation;
use delta_v_core::RenderLayer;

use super::ActiveCockpitStation;
use super::components::SpeedText;
use super::components::StatusGauge;
use super::components::VelocityVectorIndicator;
use super::velocity_indicator::create_arrow_image;

/// Spawns the cockpit overlay for the player ship.
///
/// The cockpit overlay is always spawned, but its visibility is controlled by
/// [`cockpit_visibility_system`] based on the active camera.
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn spawn_cockpit_overlay(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    cockpit: Res<'_, CockpitOverlayResource>,
) {
    let Some(station) = cockpit.stations.first() else {
        tracing::error!("cockpit.stations must contain at least one station");
        return;
    };

    // The template_path is the asset directory relative to the assets/ root,
    // e.g., "templates/ships/space-fighter-comrade1280".
    // The station texture is relative to that directory, e.g., "cockpit/default.png".
    // The asset server loads from "assets/" + template_path + "/" + texture.
    let texture_path = format!("{}/{}", cockpit.template_path, station.texture);
    let texture_handle = asset_server.load::<Image>(texture_path);

    // Spawn a full-screen UI node with an ImageNode for the cockpit overlay.
    // ImageNode with Stretch mode fills the entire viewport regardless of image size.
    // Visibility starts as Visible, but will be toggled by cockpit_visibility_system.
    let overlay_entity = commands
        .spawn((
            Node {
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            Transform::default(),
            RenderLayer::Ui.render_layers(),
            Visibility::Visible,
            super::CockpitOverlay {
                texture: texture_handle.clone(),
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode {
                    image: texture_handle,
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                Visibility::Visible,
            ));
        })
        .id();

    commands.insert_resource(ActiveCockpitStation {
        station_id: station.id.clone(),
    });

    // Store the overlay entity so gauges can be parented to it.
    commands.insert_resource(CockpitOverlayEntityResource(overlay_entity));
}

/// Resource holding the cockpit overlay entity ID.
/// Used by [`spawn_status_gauges`] to parent gauge nodes to the overlay.
#[derive(Resource)]
pub struct CockpitOverlayEntityResource(pub Entity);

/// Spawns the velocity vector indicator as a Sprite + Text on the Ui render layer.
///
/// `Sprite` in Bevy 0.18 is rendered by the UI camera (orthographic).
/// The sprite is centered on screen. The `velocity_vector_system` rotates
/// it via `Transform::rotation` each frame to point in the velocity direction.
/// A `Text` child entity displays the current speed.
///
/// Runs during `OnEnter(AppState::InGame)`.
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn spawn_velocity_vector_indicator(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
) {
    let arrow_image = create_arrow_image();
    let arrow_handle = asset_server.add(arrow_image);

    commands.spawn((
        Sprite {
            image: arrow_handle,
            ..default()
        },
        Transform::default(),
        RenderLayer::Ui.render_layers(),
        Visibility::Visible,
        VelocityVectorIndicator,
    ));

    // Spawn speed text as a separate entity (not a child of the rotating sprite).
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(20.0),
                top: Val::Px(-8.0),
                ..default()
            },
            ..default()
        },
        RenderLayer::Ui.render_layers(),
        Visibility::Visible,
        SpeedText,
    ));

    tracing::info!("velocity vector indicator sprite + text spawned");
}

/// Resource holding the cockpit overlay definition for the player ship.
#[derive(Resource)]
pub struct CockpitOverlayResource {
    /// Template asset directory path relative to the assets/ root.
    pub template_path: String,
    /// List of cockpit stations.
    pub stations: Vec<CockpitStation>,
    /// Width of the current station's texture in pixels.
    /// Used to scale slot coordinates (which are in texture pixel space) to viewport percentages.
    pub texture_width: f32,
    /// Height of the current station's texture in pixels.
    pub texture_height: f32,
}

/// Spawns status gauge UI elements as children of the cockpit overlay.
///
/// Runs during `OnEnter(AppState::InGame)`. Reads all stations' `slots` and
/// spawns a [`StatusGauge`] UI element at each slot's position/size.
/// The gauge type is determined by the slot's `default_gauge` field.
///
/// Gauges are parented to the cockpit overlay entity so they render AFTER
/// the cockpit PNG's `ImageNode`, ensuring correct visual layering regardless
/// of render pass ordering.
///
/// Gauges are only spawned for supported gauge types (`"health"`, `"weapon_heat"`).
/// Unsupported gauge types are logged at DEBUG level and skipped.
/// All gauges start hidden; [`init_gauge_visibility`] sets the correct visibility on startup.
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn spawn_status_gauges(
    mut commands: Commands<'_, '_>,
    _asset_server: Res<'_, AssetServer>,
    cockpit: Res<'_, CockpitOverlayResource>,
    overlay_entity: Res<'_, CockpitOverlayEntityResource>,
) {
    if cockpit.stations.is_empty() {
        tracing::debug!("[status_gauges] no stations with gauge slots");
        return;
    }

    // Convert texture pixel coordinates to viewport percentages.
    // The cockpit texture is stretched to fill the entire viewport (100vw x 100vh),
    // so slot coordinates must be scaled by the same ratio.
    let scale_x = 100.0 / cockpit.texture_width;
    let scale_y = 100.0 / cockpit.texture_height;

    let mut total_gauges = 0;

    for station in &cockpit.stations {
        for (index, slot) in station.slots.iter().enumerate() {
            let gauge_type = &slot.default_gauge;

            // Only spawn gauges for supported types.
            if gauge_type != "health" && gauge_type != "weapon_heat" {
                tracing::debug!(
                    "[status_gauges] station '{}' slot {}: unsupported gauge type '{}', skipping",
                    station.id,
                    index,
                    gauge_type
                );
                continue;
            }

            // Build the gauge node based on the slot shape.
            let (width, height, left, top) = match &slot.shape {
                crate::ship_templates::GaugeShape::Rectangle { x1, y1, x2, y2 } => {
                    let w = (x2 - x1).abs() * scale_x;
                    let h = (y2 - y1).abs() * scale_y;
                    (
                        Val::Vw(w),
                        Val::Vh(h),
                        Val::Vw(*x1 * scale_x),
                        Val::Vh(*y1 * scale_y),
                    )
                }
                crate::ship_templates::GaugeShape::Circle { cx, cy, r } => {
                    let size_w = Val::Vw(r * 2.0 * scale_x);
                    let size_h = Val::Vh(r * 2.0 * scale_y);
                    (
                        size_w,
                        size_h,
                        Val::Vw((cx - r) * scale_x),
                        Val::Vh((cy - r) * scale_y),
                    )
                }
            };

            // Background node (dark, opaque).
            // Use BackgroundColor to avoid asset handle conflicts when station
            // switching reloads textures (which corrupts ImageNode handles).
            // Since the gauge is a child of the cockpit overlay, BackgroundColor
            // renders after the cockpit PNG in the UI hierarchy.
            // Start hidden — init_gauge_visibility will set the correct state.
            commands.entity(overlay_entity.0).with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width,
                            height,
                            position_type: PositionType::Absolute,
                            left,
                            top,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                        RenderLayer::Ui.render_layers(),
                        Visibility::Hidden,
                        StatusGauge {
                            station_id: station.id.clone(),
                            slot_id: index.to_string(),
                            shape: slot.shape.clone(),
                            gauge_type: gauge_type.clone(),
                        },
                    ))
                    .with_children(|fill_parent| {
                        // Fill node (colored, sized by the status_gauge_system each frame).
                        let fill_color = match gauge_type.as_str() {
                            "health" => Color::srgb(0.0, 0.8, 0.0),
                            "weapon_heat" => Color::srgb(0.8, 0.4, 0.0),
                            _ => Color::WHITE,
                        };
                        fill_parent.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(fill_color),
                            Visibility::Visible,
                        ));
                    });
            });

            total_gauges += 1;
            tracing::debug!(
                "[status_gauges] spawned {} gauge for station '{}' slot {} ({:?}, {:?})",
                gauge_type,
                station.id,
                index,
                left,
                top
            );
        }
    }

    tracing::info!(
        "[status_gauges] spawned {} gauge(s) for {} station(s)",
        total_gauges,
        cockpit.stations.len()
    );
}

/// Creates a 1x1 solid color texture as an `Image`.
///
/// Used for gauge backgrounds/fills to ensure they render in the same pass
/// as the cockpit PNG (which uses `ImageNode`). This prevents render-order
/// issues where `BackgroundColor` nodes render in a separate pass.
pub fn solid_color_image(color: Color) -> Image {
    #![allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    let rgba = color.to_srgba();
    let r = (rgba.red * 255.0) as u8;
    let g = (rgba.green * 255.0) as u8;
    let b = (rgba.blue * 255.0) as u8;
    let a = (rgba.alpha * 255.0) as u8;

    let data = vec![r, g, b, a];

    Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
