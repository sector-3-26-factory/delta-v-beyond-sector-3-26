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

//! Boundary checking systems.
//!
//! Systems that check entity positions against sector boundaries.

use bevy::prelude::*;

use crate::{PlayerShipEntity, SectorBoundaryResource};

/// System that checks player ship position against sector boundaries.
///
/// Runs in `Update` schedule during `InGame` state.
/// Applies the configured boundary behavior (wrap, clamp, or destroy).
#[allow(clippy::needless_pass_by_value)]
pub fn check_sector_boundary_system(
    player_entity: Res<'_, PlayerShipEntity>,
    mut query: Query<'_, '_, &mut Transform>,
    resource: Res<'_, SectorBoundaryResource>,
) {
    let boundary = &resource.boundary;

    // Only check the player ship entity
    let Ok(mut transform) = query.get_mut(player_entity.0) else {
        return;
    };

    let new_pos = boundary.apply(transform.translation);

    if let Some(pos) = new_pos {
        transform.translation = pos;
    }
    // If None, the ship should be destroyed - but for now we just don't update
    // the position. In the future, this could despawn the player ship.
}
