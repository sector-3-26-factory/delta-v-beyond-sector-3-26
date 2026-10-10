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

use crate::belt_field_streaming::calculate_sector_index;
use bevy::prelude::*;

#[test]
fn test_calculate_sector_index() {
    let idx = calculate_sector_index("sun", 4.04e11, 5.0e7, Vec3::new(4.04e11, 0.0, 0.0));
    assert_eq!(idx, 0);

    let idx = calculate_sector_index("sun", 4.04e11, 5.0e7, Vec3::new(0.0, 4.04e11, 0.0));
    assert!(idx > 0);
}
