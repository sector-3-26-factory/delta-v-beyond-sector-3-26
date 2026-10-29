// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Sphere-sphere collision detection tests.
//!
//! Each test function is registered in the global registry via
//! [`init_registry`] and called by the binary's `run_test_function()`
//! when `--test-function <name>` is passed.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code,
    dead_code
)]

use bevy::prelude::*;
use delta_v_physics::collision::{CollisionShape, CollisionShapeData};

const fn sphere_shape(radius: f32) -> CollisionShape {
    CollisionShape(CollisionShapeData::sphere(radius, Vec3::ZERO))
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

fn test_sphere_sphere_overlap(_app: &mut App) {
    let a = sphere_shape(2.0);
    let b = sphere_shape(2.0);
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(3.0, 0.0, 0.0);

    let result = check_unrotated(pos_a, &a, pos_b, &b);
    assert!(result.is_some(), "spheres should collide");
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

fn test_sphere_sphere_no_overlap(_app: &mut App) {
    let a = sphere_shape(1.0);
    let b = sphere_shape(1.0);
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(5.0, 0.0, 0.0);

    let result = check_unrotated(pos_a, &a, pos_b, &b);
    assert!(result.is_none(), "spheres should not collide");
}

fn test_sphere_sphere_touching_exactly(_app: &mut App) {
    let a = sphere_shape(1.0);
    let b = sphere_shape(1.0);
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(2.0, 0.0, 0.0);

    let result = check_unrotated(pos_a, &a, pos_b, &b);
    assert!(
        result.is_none(),
        "exactly touching spheres should not collide",
    );
}

fn test_sphere_sphere_coincident_centers(_app: &mut App) {
    let a = sphere_shape(2.0);
    let b = sphere_shape(3.0);
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::ZERO;

    let result = check_unrotated(pos_a, &a, pos_b, &b);
    assert!(result.is_some(), "coincident spheres should collide");
    let (normal, penetration) = result.unwrap();
    assert!(
        (penetration - 5.0).abs() < 0.01,
        "penetration should be 5.0, got {penetration}"
    );
    assert!(
        (normal.z - 1.0).abs() < 0.01,
        "normal should be +Z for coincident, got {normal}"
    );
}

/// Register all test functions in the global registry.
///
/// Called from [`super::super::init_registry`] (the crate-level
/// `real_app_ITs::init_registry`).
pub fn init_registry() {
    do_register("physics::sphere_sphere_overlap", test_sphere_sphere_overlap);
    do_register(
        "physics::sphere_sphere_no_overlap",
        test_sphere_sphere_no_overlap,
    );
    do_register(
        "physics::sphere_sphere_touching_exactly",
        test_sphere_sphere_touching_exactly,
    );
    do_register(
        "physics::sphere_sphere_coincident_centers",
        test_sphere_sphere_coincident_centers,
    );
}

/// Register a test function in the global registry.
///
/// The entry is leaked so it lives for the lifetime of the process.
fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
