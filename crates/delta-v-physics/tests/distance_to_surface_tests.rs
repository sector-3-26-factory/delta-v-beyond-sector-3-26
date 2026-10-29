// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Distance-to-surface tests.
//!
//!   xvfb-run -a cargo test --test `distance_to_surface_tests` -p delta-v-physics

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

const fn box_shape(half_extents: Vec3) -> CollisionShape {
    CollisionShape(CollisionShapeData::box_shape(half_extents, Vec3::ZERO))
}

fn test_distance_to_surface_sphere_outside(_app: &mut App) {
    let point = Vec3::new(10.0, 0.0, 0.0);
    let entity_pos = Vec3::ZERO;
    let shape = sphere_shape(3.0);
    let distance = delta_v_physics::distance_to_surface(point, entity_pos, &shape);
    check(
        (distance - 7.0).abs() < 0.01,
        &format!("distance to sphere surface should be 7, got {distance}"),
    );
}

fn test_distance_to_surface_sphere_inside(_app: &mut App) {
    let point = Vec3::new(1.0, 0.0, 0.0);
    let entity_pos = Vec3::ZERO;
    let shape = sphere_shape(3.0);
    let distance = delta_v_physics::distance_to_surface(point, entity_pos, &shape);
    check(
        (distance - 0.0).abs() < 0.01,
        &format!("distance to sphere surface when inside should be 0, got {distance}"),
    );
}

fn test_distance_to_surface_sphere_on_surface(_app: &mut App) {
    let point = Vec3::new(3.0, 0.0, 0.0);
    let entity_pos = Vec3::ZERO;
    let shape = sphere_shape(3.0);
    let distance = delta_v_physics::distance_to_surface(point, entity_pos, &shape);
    check(
        (distance - 0.0).abs() < 0.01,
        &format!("distance to sphere surface when on surface should be 0, got {distance}"),
    );
}

fn test_distance_to_surface_box_outside(_app: &mut App) {
    let point = Vec3::new(5.0, 0.0, 0.0);
    let entity_pos = Vec3::ZERO;
    let shape = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let distance = delta_v_physics::distance_to_surface(point, entity_pos, &shape);
    check(
        (distance - 3.0).abs() < 0.01,
        &format!("distance to box surface should be 3, got {distance}"),
    );
}

fn test_distance_to_surface_box_inside(_app: &mut App) {
    let point = Vec3::new(1.0, 1.0, 1.0);
    let entity_pos = Vec3::ZERO;
    let shape = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let distance = delta_v_physics::distance_to_surface(point, entity_pos, &shape);
    check(
        (distance - 0.0).abs() < 0.01,
        &format!("distance to box surface when inside should be 0, got {distance}"),
    );
}

fn test_distance_to_surface_box_corner(_app: &mut App) {
    let point = Vec3::new(5.0, 5.0, 5.0);
    let entity_pos = Vec3::ZERO;
    let shape = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let distance = delta_v_physics::distance_to_surface(point, entity_pos, &shape);
    let expected = 3.0 * 3.0_f32.sqrt();
    check(
        (distance - expected).abs() < 0.01,
        &format!("distance to box corner should be ~{expected}, got {distance}"),
    );
}

fn test_distance_to_surface_box_with_offset(_app: &mut App) {
    let point = Vec3::new(10.0, 0.0, 0.0);
    let entity_pos = Vec3::new(5.0, 0.0, 0.0);
    let shape = box_shape(Vec3::new(2.0, 2.0, 2.0));
    let distance = delta_v_physics::distance_to_surface(point, entity_pos, &shape);
    check(
        (distance - 3.0).abs() < 0.01,
        &format!("distance to box surface with offset should be 3, got {distance}"),
    );
}

fn test_distance_to_surface_convex_hull_placeholder(_app: &mut App) {
    let point = Vec3::new(10.0, 0.0, 0.0);
    let entity_pos = Vec3::ZERO;
    let shape = CollisionShape(CollisionShapeData {
        shape_type: CollisionShapeType::ConvexHull,
        offset: Vec3::ZERO,
    });
    let distance = delta_v_physics::distance_to_surface(point, entity_pos, &shape);
    check(
        (distance - 10.0).abs() < 0.01,
        &format!("distance to convex hull should be center distance (10), got {distance}"),
    );
}

fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(
        &mut app,
        &[
            (
                "distance_to_surface_sphere_outside",
                test_distance_to_surface_sphere_outside,
            ),
            (
                "distance_to_surface_sphere_inside",
                test_distance_to_surface_sphere_inside,
            ),
            (
                "distance_to_surface_sphere_on_surface",
                test_distance_to_surface_sphere_on_surface,
            ),
            (
                "distance_to_surface_box_outside",
                test_distance_to_surface_box_outside,
            ),
            (
                "distance_to_surface_box_inside",
                test_distance_to_surface_box_inside,
            ),
            (
                "distance_to_surface_box_corner",
                test_distance_to_surface_box_corner,
            ),
            (
                "distance_to_surface_box_with_offset",
                test_distance_to_surface_box_with_offset,
            ),
            (
                "distance_to_surface_convex_hull_placeholder",
                test_distance_to_surface_convex_hull_placeholder,
            ),
        ],
    );
}
