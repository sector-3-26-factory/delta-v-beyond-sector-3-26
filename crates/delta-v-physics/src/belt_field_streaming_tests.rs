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

//! Unit tests for `belt_field_streaming` module.
//!
//! These are pure-function tests that do not require a Bevy `App` — they
//! exercise `find_nearest_sector_index` directly. The full streaming system
//! (load/despawn behaviour) is covered by the integration tests in
//! `crates/delta-v-world-tests/tests/world_belt_field_tests.rs`.

use bevy::prelude::*;

use crate::belt_field_streaming::find_nearest_sector_index;

/// Test: a player on the +X axis (angle 0) maps to sector index 0.
#[test]
fn test_find_nearest_sector_index_at_angle_zero() {
    let idx = find_nearest_sector_index("sun", 4.04e11, 5.0e7, Vec3::new(4.04e11, 0.0, 0.0));
    assert_eq!(idx, 0, "player on +X axis should map to sector 0");
}

/// Test: a player on the +Y axis (angle PI/2) maps to a non-zero sector.
#[test]
fn test_find_nearest_sector_index_at_angle_pi_half() {
    let idx = find_nearest_sector_index("sun", 4.04e11, 5.0e7, Vec3::new(0.0, 4.04e11, 0.0));
    assert!(idx > 0, "player on +Y axis should map to a non-zero sector");
}

/// Test: the function is deterministic — the same input yields the same index.
#[test]
fn test_find_nearest_sector_index_is_deterministic() {
    let pos = Vec3::new(4.04e11, 0.0, 0.0);
    let idx1 = find_nearest_sector_index("sun", 4.04e11, 5.0e7, pos);
    let idx2 = find_nearest_sector_index("sun", 4.04e11, 5.0e7, pos);
    assert_eq!(
        idx1, idx2,
        "find_nearest_sector_index must be deterministic"
    );
}

/// Test: adjacent positions map to the same or neighbouring sectors, never a
/// jump across the ring.
#[test]
fn test_find_nearest_sector_index_adjacent_positions_neighbour() {
    let sector_size = 5.0e7;
    let orbital_distance = 4.04e11;
    let a = find_nearest_sector_index(
        "sun",
        orbital_distance,
        sector_size,
        Vec3::new(4.04e11, 0.0, 0.0),
    );
    let b = find_nearest_sector_index(
        "sun",
        orbital_distance,
        sector_size,
        Vec3::new(4.04e11, 1.0e6, 0.0),
    );
    let diff = (a - b).abs();
    assert!(
        diff <= 1,
        "adjacent positions along the ring should map to the same or neighbouring sector, got {a} and {b}"
    );
}
