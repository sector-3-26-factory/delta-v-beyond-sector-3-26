// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Penetration depth tests.

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

fn test_penetration_depth_sphere_sphere(_app: &mut App) {
    let a = sphere_shape(3.0);
    let b = sphere_shape(3.0);
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(4.0, 0.0, 0.0);
    let result =
        delta_v_physics::check_collision(pos_a, Quat::IDENTITY, &a, pos_b, Quat::IDENTITY, &b);
    assert!(result.is_some(), "spheres should collide");
    let (_normal, penetration) = result.unwrap();
    assert!(
        (penetration - 2.0).abs() < 0.01,
        "penetration should be 2.0, got {penetration}"
    );
}

pub fn init_registry() {
    do_register(
        "physics::penetration_depth_sphere_sphere",
        test_penetration_depth_sphere_sphere,
    );
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
