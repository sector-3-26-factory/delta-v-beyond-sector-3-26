// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Convex hull fallback tests.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_physics::collision::{CollisionShape, CollisionShapeData, CollisionShapeType};

fn test_convex_hull_fallback_uses_sphere_approximation(_app: &mut App) {
    let a = CollisionShape(CollisionShapeData {
        shape_type: CollisionShapeType::ConvexHull,
        offset: Vec3::ZERO,
    });
    let b = CollisionShape(CollisionShapeData {
        shape_type: CollisionShapeType::ConvexHull,
        offset: Vec3::ZERO,
    });
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(1.5, 0.0, 0.0);
    let result =
        delta_v_physics::check_collision(pos_a, Quat::IDENTITY, &a, pos_b, Quat::IDENTITY, &b);
    assert!(
        result.is_some(),
        "convex hull fallback should detect collision via sphere approx"
    );
}

pub fn init_registry() {
    do_register(
        "physics::convex_hull_fallback_uses_sphere_approximation",
        test_convex_hull_fallback_uses_sphere_approximation,
    );
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
