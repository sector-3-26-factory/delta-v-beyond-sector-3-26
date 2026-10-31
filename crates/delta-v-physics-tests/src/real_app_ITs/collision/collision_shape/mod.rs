// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Collision shape constructor tests.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_physics::collision::{CollisionShape, CollisionShapeType};

fn test_collision_shape_sphere_constructor(_app: &mut App) {
    let shape = CollisionShape::sphere(5.0, Vec3::new(1.0, 2.0, 3.0));
    match shape.shape_type {
        CollisionShapeType::Sphere { radius } => {
            assert!((radius - 5.0).abs() < 0.01, "sphere radius should be 5.0");
        }
        _ => unreachable!("expected sphere shape"),
    }
    assert!(
        shape.offset == Vec3::new(1.0, 2.0, 3.0),
        "offset should be (1,2,3), got {:?}",
        shape.offset
    );
}

fn test_collision_shape_box_constructor(_app: &mut App) {
    let he = Vec3::new(1.0, 2.0, 3.0);
    let offset = Vec3::new(0.5, 0.5, 0.5);
    let shape = CollisionShape::box_shape(he, offset);
    match shape.shape_type {
        CollisionShapeType::Box { half_extents } => assert!(
            half_extents == he,
            "half_extents should be {he:?}, got {half_extents:?}"
        ),
        _ => unreachable!("expected box shape"),
    }
    assert!(
        shape.offset == offset,
        "offset should be {offset:?}, got {:?}",
        shape.offset
    );
}

pub fn init_registry() {
    do_register(
        "physics::collision_shape_sphere_constructor",
        test_collision_shape_sphere_constructor,
    );
    do_register(
        "physics::collision_shape_box_constructor",
        test_collision_shape_box_constructor,
    );
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
