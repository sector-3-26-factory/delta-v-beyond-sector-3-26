// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for boundary module.

#![allow(clippy::unwrap_used, clippy::panic, clippy::float_cmp)]
use bevy::prelude::*;

use crate::{BoundaryBehavior, SectorBoundary, SectorBoundaryResource};

#[test]
fn test_sector_boundary_contains() {
    let boundary = SectorBoundary::new(10_000.0, BoundaryBehavior::Wrap);

    // Origin is inside
    assert!(boundary.contains(Vec3::ZERO));

    // Corners are inside
    assert!(boundary.contains(Vec3::new(4999.0, 4999.0, 4999.0)));
    assert!(boundary.contains(Vec3::new(-4999.0, -4999.0, -4999.0)));

    // Outside
    assert!(!boundary.contains(Vec3::new(5001.0, 0.0, 0.0)));
    assert!(!boundary.contains(Vec3::new(0.0, 5001.0, 0.0)));
    assert!(!boundary.contains(Vec3::new(0.0, 0.0, 5001.0)));
}

#[test]
fn test_sector_boundary_apply_wrap() {
    let boundary = SectorBoundary::new(10_000.0, BoundaryBehavior::Wrap);

    // Wrap from +X to -X
    let result = boundary.apply(Vec3::new(5001.0, 0.0, 0.0));
    assert_eq!(result.unwrap().x, -5000.0);

    // Wrap from -X to +X
    let result = boundary.apply(Vec3::new(-5001.0, 0.0, 0.0));
    assert_eq!(result.unwrap().x, 5000.0);

    // Wrap from +Y to -Y
    let result = boundary.apply(Vec3::new(0.0, 5001.0, 0.0));
    assert_eq!(result.unwrap().y, -5000.0);

    // Wrap from +Z to -Z
    let result = boundary.apply(Vec3::new(0.0, 0.0, 5001.0));
    assert_eq!(result.unwrap().z, -5000.0);
}

#[test]
fn test_sector_boundary_apply_clamp() {
    let boundary = SectorBoundary::new(10_000.0, BoundaryBehavior::Clamp);

    // Clamp to max
    let result = boundary.apply(Vec3::new(5001.0, 0.0, 0.0));
    assert_eq!(result.unwrap().x, 5000.0);

    // Clamp to min
    let result = boundary.apply(Vec3::new(-5001.0, 0.0, 0.0));
    assert_eq!(result.unwrap().x, -5000.0);
}

#[test]
fn test_sector_boundary_apply_destroy() {
    let boundary = SectorBoundary::new(10_000.0, BoundaryBehavior::Destroy);

    // Inside is OK
    assert!(boundary.apply(Vec3::new(0.0, 0.0, 0.0)).is_some());

    // Outside returns None
    assert!(boundary.apply(Vec3::new(5001.0, 0.0, 0.0)).is_none());
    assert!(boundary.apply(Vec3::new(-5001.0, 0.0, 0.0)).is_none());
}

#[test]
fn test_sector_boundary_resource_default() {
    let resource = SectorBoundaryResource::default();
    assert!(resource.boundary.contains(Vec3::ZERO));
}
