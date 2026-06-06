// See AGENTS.md
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

//! Tests for floating origin management.

use super::*;

#[test]
fn test_floating_origin_default() {
    let origin = FloatingOrigin::default();
    assert_eq!(origin.offset, Vec3::ZERO);
}

#[test]
fn test_floating_origin_new() {
    let offset = Vec3::new(1000.0, 2000.0, 3000.0);
    let origin = FloatingOrigin::new(offset);
    assert_eq!(origin.offset, offset);
}

#[test]
fn test_to_absolute() {
    let origin = FloatingOrigin::new(Vec3::new(1000.0, 0.0, 0.0));
    let local_pos = Vec3::new(100.0, 0.0, 0.0);
    let absolute = origin.to_absolute(local_pos);
    assert_eq!(absolute, Vec3::new(1100.0, 0.0, 0.0));
}

#[test]
fn test_to_local() {
    let origin = FloatingOrigin::new(Vec3::new(1000.0, 0.0, 0.0));
    let absolute_pos = Vec3::new(1100.0, 0.0, 0.0);
    let local = origin.to_local(absolute_pos);
    assert_eq!(local, Vec3::new(100.0, 0.0, 0.0));
}

#[test]
fn test_roundtrip_conversion() {
    let origin = FloatingOrigin::new(Vec3::new(5000.0, -3000.0, 1000.0));
    let local_pos = Vec3::new(100.0, 200.0, 300.0);
    let absolute = origin.to_absolute(local_pos);
    let back_to_local = origin.to_local(absolute);
    assert_eq!(local_pos, back_to_local);
}

#[test]
fn test_origin_threshold_default() {
    let threshold = OriginThreshold::default();
    assert!((threshold.threshold - 5_000.0).abs() < f32::EPSILON);
}

#[test]
fn test_origin_threshold_new() {
    let threshold = OriginThreshold::new(10_000.0);
    assert!((threshold.threshold - 10_000.0).abs() < f32::EPSILON);
}
