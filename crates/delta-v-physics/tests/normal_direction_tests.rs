// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Collision normal direction tests.
//!
//! Registered as a `[[test]]` target with `harness = false` so that
//! `cargo test --test normal_direction_tests` runs `main()` on the
//! **main thread**, which is required by Bevy's `WinitPlugin`.
//!
//!   xvfb-run -a cargo test --test `normal_direction_tests` -p delta-v-physics

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
    check(result.is_some(), "spheres should collide");
    let (normal, _penetration) = result.unwrap();
    check(
        (normal.y - 1.0).abs() < 0.01,
        &format!("normal should point +Y from A to B, got {normal}"),
    );
}

fn test_collision_normal_negative_direction(_app: &mut App) {
    let a = sphere_shape(2.0);
    let b = sphere_shape(2.0);
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(-3.0, 0.0, 0.0);

    let result =
        delta_v_physics::check_collision(pos_a, Quat::IDENTITY, &a, pos_b, Quat::IDENTITY, &b);
    check(result.is_some(), "spheres should collide");
    let (normal, _penetration) = result.unwrap();
    check(
        (normal.x + 1.0).abs() < 0.01,
        &format!("normal should point -X from A to B, got {normal}"),
    );
}

fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(
        &mut app,
        &[
            (
                "collision_normal_points_from_a_to_b",
                test_collision_normal_points_from_a_to_b,
            ),
            (
                "collision_normal_negative_direction",
                test_collision_normal_negative_direction,
            ),
        ],
    );
}
