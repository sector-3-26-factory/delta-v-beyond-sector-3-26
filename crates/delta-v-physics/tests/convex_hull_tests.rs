// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Convex hull fallback tests.
//!
//!   xvfb-run -a cargo test --test `convex_hull_tests` -p delta-v-physics

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
use delta_v_physics::collision::{CollisionShape, CollisionShapeData, CollisionShapeType};
use delta_v_test_support::{check, make_app, run_harness};

fn build_test_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-physics/tests/fixtures/assets".to_string(),
        world_name: Some("floating_origin".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
}

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
    check(
        result.is_some(),
        "convex hull fallback should detect collision via sphere approx",
    );
}

fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(
        &mut app,
        &[(
            "convex_hull_fallback_uses_sphere_approximation",
            test_convex_hull_fallback_uses_sphere_approximation,
        )],
    );
}
