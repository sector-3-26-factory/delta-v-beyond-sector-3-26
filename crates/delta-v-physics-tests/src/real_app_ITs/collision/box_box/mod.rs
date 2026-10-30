// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Box-box collision detection tests.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_physics::collision::{CollisionShape, CollisionShapeData};

const fn box_shape(half_extents: Vec3) -> CollisionShape {
    CollisionShape(CollisionShapeData::box_shape(half_extents, Vec3::ZERO))
}

fn check_unrotated(
    pos_a: Vec3,
    shape_a: &CollisionShape,
    pos_b: Vec3,
    shape_b: &CollisionShape,
) -> Option<(Vec3, f32)> {
    delta_v_physics::check_collision(
        pos_a,
        Quat::IDENTITY,
        shape_a,
        pos_b,
        Quat::IDENTITY,
        shape_b,
    )
}

fn test_box_box_overlap(_app: &mut App) {
    let a = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let b = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(3.0, 0.0, 0.0);
    let result = check_unrotated(pos_a, &a, pos_b, &b);
    assert!(result.is_some(), "boxes should collide");
    let (normal, penetration) = result.unwrap();
    assert!(
        (penetration - 1.0).abs() < 0.01,
        "penetration should be ~1.0, got {penetration}"
    );
    assert!(
        (normal.x - 1.0).abs() < 0.01,
        "normal should point +X, got {normal}"
    );
}

fn test_box_box_no_overlap(_app: &mut App) {
    let a = box_shape(Vec3::new(1.0, 1.0, 1.0));
    let b = box_shape(Vec3::new(1.0, 1.0, 1.0));
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(5.0, 0.0, 0.0);
    let result = check_unrotated(pos_a, &a, pos_b, &b);
    assert!(result.is_none(), "boxes should not collide");
}

fn test_box_box_minimum_penetration_axis(_app: &mut App) {
    let a = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let b = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(3.0, 1.0, 0.0);
    let result = check_unrotated(pos_a, &a, pos_b, &b);
    assert!(result.is_some(), "boxes should collide");
    let (normal, penetration) = result.unwrap();
    assert!(
        (penetration - 1.0).abs() < 0.01,
        "min penetration should be 1.0 on X, got {penetration}"
    );
    assert!(
        (normal.x - 1.0).abs() < 0.01,
        "normal should be +X (min penetration axis), got {normal}"
    );
}

pub fn init_registry() {
    do_register("physics::box_box_overlap", test_box_box_overlap);
    do_register("physics::box_box_no_overlap", test_box_box_no_overlap);
    do_register(
        "physics::box_box_minimum_penetration_axis",
        test_box_box_minimum_penetration_axis,
    );
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
