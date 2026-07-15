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

//! Floating origin recentering systems.
//!
//! Per ADR-0007, these systems manage the floating origin technique:
//! tracking the player ship's distance from the current origin and
//! recentering when the threshold is exceeded.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

/// System that checks if the origin needs to be recentered.
///
/// Per ADR-0007, this runs in `FixedUpdate` before physics integration.
/// If the player ship exceeds the threshold distance from the current origin,
/// the origin is recentered and all top-level entities with Transform are translated.
/// Child entities (with `ChildOf` component) are excluded since they follow their parent.
/// Only entities in the Gameplay render layer (layer 0) are translated.
/// This excludes UI elements (`CockpitBackground`, `CockpitForeground`, `Menu`) and lights.
/// Entities without a `RenderLayers` component are treated as Gameplay layer.
#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub fn check_and_recenter_origin_system(
    mut commands: Commands<'_, '_>,
    player_entity: Res<'_, PlayerShipEntity>,
    query: Query<
        '_,
        '_,
        (
            Entity,
            &Transform,
            Option<&ChildOf>,
            Option<&RenderLayers>,
            Option<&DirectionalLight>,
            Option<&AmbientLight>,
        ),
    >,
    config: Res<'_, FloatingOriginConfig>,
    current_origin: Res<'_, FloatingOrigin>,
) {
    // Get the player ship's position
    let Ok((_, player_transform, _, _, _, _)) = query.get(player_entity.0) else {
        return; // Player ship not found
    };
    let player_pos = player_transform.translation;

    // Check if the player ship's distance from the current origin exceeds threshold
    // The player's distance from origin is the length of their position (since origin is at 0,0,0 in local space)
    let distance_from_origin = player_pos.length();

    // Check if we need to recenter
    if distance_from_origin <= config.recenter_threshold_m {
        return; // No recentering needed
    }

    // Collect entity data for all top-level entities with Transform.
    // Child entities (with ChildOf) are excluded since they follow their parent's Transform.
    // Light entities (DirectionalLight, AmbientLight) are excluded since they don't have
    // meaningful world positions that need recentering.
    // Only entities in the Gameplay render layer (layer 0) are translated.
    // Entities without RenderLayers are treated as Gameplay layer (per apply_gameplay_render_layers).
    // This excludes UI elements (`CockpitBackground`, `CockpitForeground`, `Menu`) and lights.
    // We also collect rotation to preserve it during recentering.
    let entity_data: Vec<(Entity, Vec3, Quat)> = query
        .iter()
        .filter(
            |(_, _, parent, render_layers, directional_light, ambient_light)| {
                // Exclude child entities
                if parent.is_some() {
                    return false;
                }
                // Exclude light entities
                if directional_light.is_some() || ambient_light.is_some() {
                    return false;
                }
                // Include entities in Gameplay layer (layer 0)
                // Entities without RenderLayers are treated as Gameplay layer
                render_layers.is_none_or(|layers| layers.intersects(&RenderLayers::layer(0)))
            },
        )
        .map(|(entity, transform, _, _, _, _)| (entity, transform.translation, transform.rotation))
        .collect();

    // The translation to apply: move all entities so the player is at the origin
    // Old positions are relative to the old origin (0,0,0)
    // New positions should be relative to the player's position
    // So: new_pos = old_pos - player_pos
    let translation = player_pos;

    info!(
        threshold = config.recenter_threshold_m,
        player_distance = distance_from_origin,
        entity_count = entity_data.len(),
        "Recentering origin to player position"
    );

    // Calculate the new cumulative offset from the absolute world origin.
    // The player's local position is relative to the current origin.
    // Adding it to the current offset gives us the new absolute offset.
    let new_offset = current_origin.offset + translation;

    // Update the origin resource to the cumulative offset.
    // This makes the player the new origin (local position 0,0,0).
    commands.insert_resource(FloatingOrigin::new(new_offset));

    // Translate all top-level entities using commands.
    // Each entity's new position = old position - player's local position.
    // Rotation is preserved to avoid resetting entity orientations.
    for (entity, old_pos, rotation) in entity_data {
        commands.entity(entity).insert(Transform {
            translation: old_pos - translation,
            rotation,
            ..default()
        });
    }
}
