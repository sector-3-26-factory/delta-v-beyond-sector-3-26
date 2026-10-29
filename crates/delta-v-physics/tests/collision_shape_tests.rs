// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Collision shape constructor tests.
//!
//!   xvfb-run -a cargo test --test `collision_shape_tests` -p delta-v-physics

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_app::{AppConfig, build_app};
use delta_v_physics::collision::{CollisionShape, CollisionShapeType};
use delta_v_test_framework::{check, make_app, run_harness};

fn build_test_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-physics/tests/fixtures/assets".to_string(),
        world_name: Some("floating_origin".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
}

fn test_collision_shape_sphere_constructor(_app: &mut App) {
    let shape = CollisionShape::sphere(5.0, Vec3::new(1.0, 2.0, 3.0));
    match shape.shape_type {
        CollisionShapeType::Sphere { radius } => {
            check((radius - 5.0).abs() < 0.01, "sphere radius should be 5.0");
        }
        _ => check(false, "expected sphere shape"),
    }
    check(
        shape.offset == Vec3::new(1.0, 2.0, 3.0),
        &format!("offset should be (1,2,3), got {:?}", shape.offset),
    );
}

fn test_collision_shape_box_constructor(_app: &mut App) {
    let he = Vec3::new(1.0, 2.0, 3.0);
    let offset = Vec3::new(0.5, 0.5, 0.5);
    let shape = CollisionShape::box_shape(he, offset);
    match shape.shape_type {
        CollisionShapeType::Box { half_extents } => check(
            half_extents == he,
            &format!("half_extents should be {he:?}, got {half_extents:?}"),
        ),
        _ => check(false, "expected box shape"),
    }
    check(
        shape.offset == offset,
        &format!("offset should be {offset:?}, got {:?}", shape.offset),
    );
}

fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(
        &mut app,
        &[
            (
                "collision_shape_sphere_constructor",
                test_collision_shape_sphere_constructor,
            ),
            (
                "collision_shape_box_constructor",
                test_collision_shape_box_constructor,
            ),
        ],
    );
}
