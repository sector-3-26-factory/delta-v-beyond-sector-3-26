// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Sphere-box collision detection tests.

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

const fn sphere_shape(radius: f32) -> CollisionShape {
    CollisionShape(CollisionShapeData::sphere(radius, Vec3::ZERO))
}

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

fn test_sphere_box_overlap(_app: &mut App) {
    let a = sphere_shape(3.0);
    let b = box_shape(Vec3::new(1.0, 1.0, 1.0));
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(3.0, 0.0, 0.0);
    let result = check_unrotated(pos_a, &a, pos_b, &b);
    assert!(result.is_some(), "sphere and box should collide");
    let (_normal, penetration) = result.unwrap();
    assert!(penetration > 0.0, "penetration should be positive");
}

fn test_sphere_box_no_overlap(_app: &mut App) {
    let a = sphere_shape(1.0);
    let b = box_shape(Vec3::new(1.0, 1.0, 1.0));
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(5.0, 0.0, 0.0);
    let result = check_unrotated(pos_a, &a, pos_b, &b);
    assert!(result.is_none(), "sphere and box should not collide");
}

fn test_sphere_box_normal_direction(_app: &mut App) {
    let a = sphere_shape(3.0);
    let b = box_shape(Vec3::new(1.0, 1.0, 1.0));
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(3.0, 0.0, 0.0);
    let result = check_unrotated(pos_a, &a, pos_b, &b);
    assert!(result.is_some(), "sphere and box should collide");
    let (normal, _penetration) = result.unwrap();
    assert!(
        normal.x > 0.9,
        "normal should point +X from sphere to box, got {normal}"
    );
}

fn test_sphere_inside_box(_app: &mut App) {
    let a = sphere_shape(0.5);
    let b = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::ZERO;
    let result = check_unrotated(pos_a, &a, pos_b, &b);
    assert!(result.is_some(), "sphere inside box should collide");
}

pub fn init_registry() {
    do_register("physics::sphere_box_overlap", test_sphere_box_overlap);
    do_register("physics::sphere_box_no_overlap", test_sphere_box_no_overlap);
    do_register(
        "physics::sphere_box_normal_direction",
        test_sphere_box_normal_direction,
    );
    do_register("physics::sphere_inside_box", test_sphere_inside_box);
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
