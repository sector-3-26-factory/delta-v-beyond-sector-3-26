// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Collision normal direction tests.

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

fn test_collision_normal_points_from_a_to_b(_app: &mut App) {
    let a = sphere_shape(2.0);
    let b = sphere_shape(2.0);
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(0.0, 3.0, 0.0);
    let result =
        delta_v_physics::check_collision(pos_a, Quat::IDENTITY, &a, pos_b, Quat::IDENTITY, &b);
    assert!(result.is_some(), "spheres should collide");
    let (normal, _penetration) = result.unwrap();
    assert!(
        (normal.y - 1.0).abs() < 0.01,
        "normal should point +Y from A to B, got {normal}"
    );
}

fn test_collision_normal_negative_direction(_app: &mut App) {
    let a = sphere_shape(2.0);
    let b = sphere_shape(2.0);
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(-3.0, 0.0, 0.0);
    let result =
        delta_v_physics::check_collision(pos_a, Quat::IDENTITY, &a, pos_b, Quat::IDENTITY, &b);
    assert!(result.is_some(), "spheres should collide");
    let (normal, _penetration) = result.unwrap();
    assert!(
        (normal.x + 1.0).abs() < 0.01,
        "normal should point -X from A to B, got {normal}"
    );
}

pub fn init_registry() {
    do_register(
        "physics::collision_normal_points_from_a_to_b",
        test_collision_normal_points_from_a_to_b,
    );
    do_register(
        "physics::collision_normal_negative_direction",
        test_collision_normal_negative_direction,
    );
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
