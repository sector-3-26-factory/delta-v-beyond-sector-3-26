// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! ADR-0057: collision relevance distance filter tests.
//!
//! These tests exercise the real `App` built via `build_app` and the
//! collision detection/response systems registered by `PhysicsPlugin`.
//! Each test spawns bodies with controlled `LazyLoadMesh` relevance
//! values and verifies the partitioning behaviour.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_physics::celestial::LazyLoadMesh;
use delta_v_physics::collision::{
    CollisionDetected, CollisionLayersComponent, CollisionShape, CollisionShapeData, DynamicBody,
};
use delta_v_physics::rigid_body::RigidBody;
use delta_v_test_framework::run_fixed_update;
use delta_v_types::collision::layers;

const SHIFT: Vec3 = Vec3::new(10.0, 0.0, 0.0);

const fn sphere_shape(radius: f32) -> CollisionShape {
    CollisionShape(CollisionShapeData::sphere(radius, Vec3::ZERO))
}

const fn lazy_mesh(screen_radius_px: f32) -> LazyLoadMesh {
    LazyLoadMesh {
        min_screen_radius_px: 1.0,
        loaded: true,
        mesh_path: String::new(),
        current_screen_radius_px: screen_radius_px,
        collision_relevance_px: delta_v_physics::constants::COLLISION_RELEVANCE_PX,
        mesh_child_entities: Vec::new(),
    }
}

fn spawn_relevance_asteroid(app: &mut App, position: Vec3, screen_radius_px: f32) -> Entity {
    app.world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(position),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::ASTEROID),
            DynamicBody,
            lazy_mesh(screen_radius_px),
        ))
        .id()
}

fn run_detection_and_collect(app: &mut App) -> Vec<CollisionDetected> {
    run_fixed_update(app);
    app.world_mut()
        .resource_mut::<Messages<CollisionDetected>>()
        .drain()
        .collect()
}

fn test_collision_relevant_asteroid_pair_still_collides(app: &mut App) {
    spawn_relevance_asteroid(app, SHIFT, 20.0);
    spawn_relevance_asteroid(app, SHIFT + Vec3::new(3.0, 0.0, 0.0), 20.0);
    let events = run_detection_and_collect(app);
    assert!(
        events.len() == 1,
        "two relevant asteroids overlapping must still be detected, got {} events",
        events.len()
    );
}

fn test_collision_irrelevant_asteroid_pair_is_skipped(app: &mut App) {
    spawn_relevance_asteroid(app, SHIFT, 0.5);
    spawn_relevance_asteroid(app, SHIFT + Vec3::new(3.0, 0.0, 0.0), 0.5);
    let events = run_detection_and_collect(app);
    assert!(
        events.is_empty(),
        "two irrelevant asteroids must not be tested, got {} events",
        events.len()
    );
}

fn test_collision_asteroid_pair_requires_both_relevant(app: &mut App) {
    spawn_relevance_asteroid(app, SHIFT, 20.0);
    spawn_relevance_asteroid(app, SHIFT + Vec3::new(3.0, 0.0, 0.0), 0.5);
    let events = run_detection_and_collect(app);
    assert!(
        events.is_empty(),
        "asteroid pairs need both sides relevant, got {} events",
        events.len()
    );
}

fn test_collision_body_without_lazy_mesh_is_always_tested(app: &mut App) {
    app.world_mut().spawn((
        RigidBody::new(10.0, 1.0),
        Transform::from_translation(SHIFT),
        sphere_shape(2.0),
        CollisionLayersComponent::new(layers::SHIP),
        DynamicBody,
    ));
    spawn_relevance_asteroid(app, SHIFT + Vec3::new(3.0, 0.0, 0.0), 0.1);
    let events = run_detection_and_collect(app);
    assert!(
        events.len() == 1,
        "a body without a relevance value must still be tested against everything, got {} events",
        events.len()
    );
}

fn test_collision_relevance_threshold_is_inclusive(app: &mut App) {
    let threshold = delta_v_physics::constants::COLLISION_RELEVANCE_PX;
    spawn_relevance_asteroid(app, SHIFT, threshold);
    spawn_relevance_asteroid(app, SHIFT + Vec3::new(3.0, 0.0, 0.0), threshold);
    let events = run_detection_and_collect(app);
    assert!(
        events.len() == 1,
        "a body exactly at the threshold must count as relevant, got {} events",
        events.len()
    );
}

fn test_collision_mixed_pair_needs_only_one_relevant_side(app: &mut App) {
    spawn_relevance_asteroid(app, SHIFT, 20.0);
    app.world_mut().spawn((
        RigidBody::new(10.0, 1.0),
        Transform::from_translation(SHIFT + Vec3::new(3.0, 0.0, 0.0)),
        sphere_shape(2.0),
        CollisionLayersComponent::new(layers::SHIP),
        DynamicBody,
    ));
    let events = run_detection_and_collect(app);
    assert!(
        events.len() == 1,
        "a mixed pair must be tested when either side is relevant, got {} events",
        events.len()
    );
}

pub fn init_registry() {
    do_register(
        "physics::collision_relevant_asteroid_pair_still_collides",
        test_collision_relevant_asteroid_pair_still_collides,
    );
    do_register(
        "physics::collision_irrelevant_asteroid_pair_is_skipped",
        test_collision_irrelevant_asteroid_pair_is_skipped,
    );
    do_register(
        "physics::collision_asteroid_pair_requires_both_relevant",
        test_collision_asteroid_pair_requires_both_relevant,
    );
    do_register(
        "physics::collision_body_without_lazy_mesh_is_always_tested",
        test_collision_body_without_lazy_mesh_is_always_tested,
    );
    do_register(
        "physics::collision_relevance_threshold_is_inclusive",
        test_collision_relevance_threshold_is_inclusive,
    );
    do_register(
        "physics::collision_mixed_pair_needs_only_one_relevant_side",
        test_collision_mixed_pair_needs_only_one_relevant_side,
    );
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
