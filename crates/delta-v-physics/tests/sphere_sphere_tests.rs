// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Sphere-sphere collision detection tests.
//!
//! Registered as a `[[test]]` target with `harness = false` so that
//! `cargo test --test sphere_sphere_tests` runs `main()` on the
//! **main thread**, which is required by Bevy's `WinitPlugin`.
//!
//!   xvfb-run -a cargo test --test `sphere_sphere_tests` -p delta-v-physics

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
    check(result.is_some(), "spheres should collide");
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

fn test_sphere_sphere_no_overlap(_app: &mut App) {
    let a = sphere_shape(1.0);
    let b = sphere_shape(1.0);
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(5.0, 0.0, 0.0);

    let result = check_unrotated(pos_a, &a, pos_b, &b);
    check(result.is_none(), "spheres should not collide");
}

fn test_sphere_sphere_touching_exactly(_app: &mut App) {
    let a = sphere_shape(1.0);
    let b = sphere_shape(1.0);
    let pos_a = Vec3::ZERO;
    let pos_b = Vec3::new(2.0, 0.0, 0.0);

    let result = check_unrotated(pos_a, &a, pos_b, &b);
    check(
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
    check(result.is_some(), "coincident spheres should collide");
    let (normal, penetration) = result.unwrap();
    check(
        (penetration - 5.0).abs() < 0.01,
        &format!("penetration should be 5.0, got {penetration}"),
    );
    check(
        (normal.z - 1.0).abs() < 0.01,
        &format!("normal should be +Z for coincident, got {normal}"),
    );
}

fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(
        &mut app,
        &[
            ("sphere_sphere_overlap", test_sphere_sphere_overlap),
            ("sphere_sphere_no_overlap", test_sphere_sphere_no_overlap),
            (
                "sphere_sphere_touching_exactly",
                test_sphere_sphere_touching_exactly,
            ),
            (
                "sphere_sphere_coincident_centers",
                test_sphere_sphere_coincident_centers,
            ),
        ],
    );
}
