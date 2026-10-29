// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Box-box collision detection tests.
//!
//! Registered as a `[[test]]` target with `harness = false` so that
//! `cargo test --test box_box_tests` runs `main()` on the
//! **main thread**, which is required by Bevy's `WinitPlugin`.
//!
//!   xvfb-run -a cargo test --test `box_box_tests` -p delta-v-physics

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
use delta_v_test_framework::{check, make_app, run_harness};

fn build_test_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-physics/tests/fixtures/assets".to_string(),
        world_name: Some("floating_origin".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
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

fn test_box_box_overlap(_app: &mut App) {
    let a = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let b = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(3.0, 0.0, 0.0);

    let result = check_unrotated(pos_a, &a, pos_b, &b);
    check(result.is_some(), "boxes should collide");
    let (normal, penetration) = result.unwrap();
    check(
        (penetration - 1.0).abs() < 0.01,
        &format!("penetration should be ~1.0, got {penetration}"),
    );
    check(
        (normal.x - 1.0).abs() < 0.01,
        &format!("normal should point +X, got {normal}"),
    );
}

fn test_box_box_no_overlap(_app: &mut App) {
    let a = box_shape(Vec3::new(1.0, 1.0, 1.0));
    let b = box_shape(Vec3::new(1.0, 1.0, 1.0));
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(5.0, 0.0, 0.0);

    let result = check_unrotated(pos_a, &a, pos_b, &b);
    check(result.is_none(), "boxes should not collide");
}

fn test_box_box_minimum_penetration_axis(_app: &mut App) {
    let a = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let b = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(3.0, 1.0, 0.0);

    let result = check_unrotated(pos_a, &a, pos_b, &b);
    check(result.is_some(), "boxes should collide");
    let (normal, penetration) = result.unwrap();
    check(
        (penetration - 1.0).abs() < 0.01,
        &format!("min penetration should be 1.0 on X, got {penetration}"),
    );
    check(
        (normal.x - 1.0).abs() < 0.01,
        &format!("normal should be +X (min penetration axis), got {normal}"),
    );
}

fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(
        &mut app,
        &[
            ("box_box_overlap", test_box_box_overlap),
            ("box_box_no_overlap", test_box_box_no_overlap),
            (
                "box_box_minimum_penetration_axis",
                test_box_box_minimum_penetration_axis,
            ),
        ],
    );
}
