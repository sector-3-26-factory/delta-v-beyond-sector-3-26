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

use bevy::prelude::*;
use delta_v_core::{FloatingOrigin, FloatingOriginConfig, FloatingOriginEligible};

/// System that checks if the origin needs to be recentered.
///
/// Per ADR-0007, this runs in `FixedUpdate` before physics integration.
/// If the player ship exceeds the threshold distance from the current origin,
/// the origin is recentered and all eligible entities are translated.
#[allow(clippy::needless_pass_by_value)]
pub fn check_and_recenter_origin_system(
    mut commands: Commands<'_, '_>,
    query: Query<'_, '_, (Entity, &Transform, &mut FloatingOriginEligible)>,
    origin: Res<'_, FloatingOrigin>,
    config: Res<'_, FloatingOriginConfig>,
) {
    let distance_from_origin = origin.offset.length();

    // Check if we need to recenter
    if distance_from_origin <= config.recenter_threshold_m {
        return; // No recentering needed
    }

    // Collect entity data first (positions and translation needed)
    let mut entity_data: Vec<(Entity, Vec3)> = Vec::new();
    let mut first = true;
    let mut player_pos = Vec3::ZERO;

    for (entity, transform, _) in query.iter() {
        if first {
            player_pos = transform.translation;
            first = false;
        }
        entity_data.push((entity, transform.translation));
    }

    if first {
        return; // No entities
    }

    let translation = origin.offset - player_pos;

    info!(
        threshold = config.recenter_threshold_m,
        current_offset = origin.offset.length(),
        "Recentering origin"
    );

    // Update the origin resource
    commands.insert_resource(FloatingOrigin::new(player_pos));

    // Translate all entities using commands
    for (entity, old_pos) in entity_data {
        commands
            .entity(entity)
            .insert(Transform::from_translation(old_pos - translation));
    }
}

/// System that marks newly spawned entities as eligible for floating origin translation.
///
/// This runs in `Update` during `InGame` state.
#[allow(clippy::needless_pass_by_value)]
pub fn mark_new_entities_system(
    mut commands: Commands<'_, '_>,
    query: Query<'_, '_, Entity, (Added<Transform>, Without<FloatingOriginEligible>)>,
    existing: Query<'_, '_, Entity>,
) {
    for entity in &query {
        // Check if the entity still exists in the world (it may have been despawned
        // between the query and the command, e.g., destroyed by boundary system).
        if existing.get(entity).is_ok() {
            commands.entity(entity).insert(FloatingOriginEligible);
        }
    }
}
