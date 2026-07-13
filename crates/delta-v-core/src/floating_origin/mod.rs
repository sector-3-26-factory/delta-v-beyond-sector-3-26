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

//! Floating origin management for large worlds.
//!
//! Per ADR-0007, this module implements the floating origin technique to
//! maintain `f32` precision within a sector. The origin is recentered when
//! the player ship exceeds a configured threshold.
//!
//! See [`docs/adr/0007-floating-origin.md`](docs/adr/0007-floating-origin.md).

use bevy::prelude::*;

/// The current offset of the floating origin from the world's absolute origin.
///
/// This resource tracks how far the local coordinate system has been shifted
/// from the absolute world origin. When the player ship moves more than
/// [`OriginThreshold`] from the current origin, the origin is recentered
/// and this offset is updated.
///
/// All positions in the game are stored relative to this offset.
// allow-default: FloatingOrigin is runtime state tracking the current origin offset, not JSON-backed config
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct FloatingOrigin {
    /// Current offset from absolute world origin (in metres).
    /// Added to all entity positions to get absolute coordinates.
    pub offset: Vec3,
}

impl FloatingOrigin {
    /// Creates a new floating origin at the given offset.
    #[must_use]
    pub const fn new(offset: Vec3) -> Self {
        Self { offset }
    }

    /// Returns the absolute position given a local position.
    #[inline]
    pub fn to_absolute(&self, local_pos: Vec3) -> Vec3 {
        local_pos + self.offset
    }

    /// Returns the local position given an absolute position.
    #[inline]
    pub fn to_local(&self, absolute_pos: Vec3) -> Vec3 {
        absolute_pos - self.offset
    }
}

/// The threshold distance from the current origin before recentering.
///
/// Per ADR-0007, the origin is recentered when the player ship exceeds
/// this distance from the current origin. Loaded from world definition.
#[derive(Resource, Debug, Clone, Copy)]
pub struct OriginThreshold {
    /// Distance in metres before origin is recentered.
    pub threshold: f32,
}

impl OriginThreshold {
    /// Creates a new origin threshold with the given value.
    #[must_use]
    pub const fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

/// Configuration for floating origin behavior.
///
/// This resource holds runtime configuration for the floating origin system.
/// It can be modified via the config system (ADR-0010).
// allow-default: Bevy Resource trait bound requires Default
#[derive(Resource, Debug, Clone)]
pub struct FloatingOriginConfig {
    /// Distance in metres before origin is recentered.
    pub recenter_threshold_m: f32,
}

impl Default for FloatingOriginConfig {
    fn default() -> Self {
        Self {
            // 5km threshold per ADR-0007
            recenter_threshold_m: 5_000.0,
        }
    }
}
