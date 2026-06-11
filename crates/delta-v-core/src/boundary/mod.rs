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

//! Sector boundary management.
//!
//! Defines sector boundaries and behavior when entities cross them.
//! Per ADR-0006, the coordinate system is right-handed, +Y up.

use bevy::prelude::*;

pub mod systems;

// Re-export the system for convenience
pub use systems::check_sector_boundary_system;

/// Behavior to apply when an entity crosses a sector boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum BoundaryBehavior {
    /// Wrap position to the opposite side of the sector.
    Wrap,
    /// Clamp position to the boundary edge.
    Clamp,
    /// Destroy the entity.
    Destroy,
}

/// Sector boundary definition.
///
/// Defines the min/max bounds of a sector in world coordinates.
/// The sector is a cuboid centered at the origin by default.
#[derive(Debug, Clone, Reflect)]
pub struct SectorBoundary {
    /// Minimum XYZ coordinates of the sector.
    pub min: Vec3,
    /// Maximum XYZ coordinates of the sector.
    pub max: Vec3,
    /// Behavior to apply when an entity crosses the boundary.
    pub behavior: BoundaryBehavior,
}

impl SectorBoundary {
    /// Creates a new sector boundary with the given size and behavior.
    ///
    /// The sector is centered at the origin.
    pub const fn new(size: f32, behavior: BoundaryBehavior) -> Self {
        let half = size / 2.0;
        Self {
            min: Vec3::new(-half, -half, -half),
            max: Vec3::new(half, half, half),
            behavior,
        }
    }

    /// Creates a new sector boundary with explicit min/max and wrap behavior.
    pub const fn new_cubic(min: Vec3, max: Vec3) -> Self {
        Self {
            min,
            max,
            behavior: BoundaryBehavior::Wrap,
        }
    }

    /// Checks if a position is within the sector bounds.
    pub fn contains(&self, pos: Vec3) -> bool {
        pos.x >= self.min.x
            && pos.x <= self.max.x
            && pos.y >= self.min.y
            && pos.y <= self.max.y
            && pos.z >= self.min.z
            && pos.z <= self.max.z
    }

    /// Applies boundary behavior to a position.
    ///
    /// Returns `Some(new_position)` if the position was modified,
    /// or `None` if the entity should be destroyed.
    pub fn apply(&self, mut pos: Vec3) -> Option<Vec3> {
        // Handle each axis independently
        for i in 0..3 {
            if pos[i] < self.min[i] {
                match self.behavior {
                    BoundaryBehavior::Wrap => {
                        pos[i] = self.max[i];
                    }
                    BoundaryBehavior::Clamp => {
                        pos[i] = self.min[i];
                    }
                    BoundaryBehavior::Destroy => return None,
                }
            } else if pos[i] > self.max[i] {
                match self.behavior {
                    BoundaryBehavior::Wrap => {
                        pos[i] = self.min[i];
                    }
                    BoundaryBehavior::Clamp => {
                        pos[i] = self.max[i];
                    }
                    BoundaryBehavior::Destroy => return None,
                }
            }
        }

        Some(pos)
    }
}

/// Resource containing the sector boundary configuration.
///
/// Loaded from the world definition and used by boundary checking systems.
// allow-default: Bevy Resource trait bound requires Default
#[derive(Debug, Clone, Resource)]
pub struct SectorBoundaryResource {
    /// The sector boundary definition.
    pub boundary: SectorBoundary,
}

impl Default for SectorBoundaryResource {
    fn default() -> Self {
        Self {
            boundary: SectorBoundary::new(10_000.0, BoundaryBehavior::Wrap),
        }
    }
}
