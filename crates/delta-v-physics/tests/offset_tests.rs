// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Collision shape offset tests.
//!
//!   xvfb-run -a cargo test --test `offset_tests` -p delta-v-physics

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
use delta_v_physics::collision::{CollisionShape, CollisionShapeData};
use delta_v_test_support::{check, make_app, run_harness};

fn build_test_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-physics/tests/fixtures/assets".to_string(),
        world_name: Some("floating_origin".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
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

fn test_collision_with_offset(_app: &mut App) {
    let a = CollisionShape(CollisionShapeData::sphere(2.0, Vec3::new(2.0, 0.0, 0.0)));
    let b = CollisionShape(CollisionShapeData::sphere(2.0, Vec3::new(-2.0, 0.0, 0.0)));
    // The sphere-sphere branch of `check_collision` uses pos_a/pos_b directly
    // and does NOT add the shape offset, so the effective centre must be
    // computed here (as the original test did from the Transform).
    let pos_a = Vec3::ZERO + a.offset;
    let pos_b = Vec3::ZERO + b.offset;
    let result = check_unrotated(pos_a, &a, pos_b, &b);
    check(
        result.is_none(),
        "spheres with offset should be exactly touching, not overlapping",
    );
}

fn test_collision_with_offset_overlap(_app: &mut App) {
    let a = CollisionShape(CollisionShapeData::sphere(2.0, Vec3::new(1.5, 0.0, 0.0)));
    let b = CollisionShape(CollisionShapeData::sphere(2.0, Vec3::new(-1.5, 0.0, 0.0)));
    let pos_a = Vec3::ZERO + a.offset;
    let pos_b = Vec3::ZERO + b.offset;
    let result = check_unrotated(pos_a, &a, pos_b, &b);
    check(result.is_some(), "spheres with offset should overlap");
    let (_normal, penetration) = result.unwrap();
    check(
        (penetration - 1.0).abs() < 0.01,
        &format!("penetration should be ~1.0, got {penetration}"),
    );
}

fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(
        &mut app,
        &[
            ("collision_with_offset", test_collision_with_offset),
            (
                "collision_with_offset_overlap",
                test_collision_with_offset_overlap,
            ),
        ],
    );
}
