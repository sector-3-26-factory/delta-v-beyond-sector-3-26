// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! ADR-0057: a body without a relevance value is always tested.
//!
//!   xvfb-run -a cargo test --test `collision_body_without_lazy_mesh_is_always_tested` -p delta-v-physics

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
use delta_v_physics::collision::{
    CollisionDetected, CollisionLayersComponent, CollisionShape, CollisionShapeData, DynamicBody,
};
use delta_v_physics::rigid_body::RigidBody;
use delta_v_test_framework::{check, make_app, run_fixed_update, run_harness};
use delta_v_types::collision::layers;

const SHIFT: Vec3 = Vec3::new(10.0, 0.0, 0.0);

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

const fn lazy_mesh(screen_radius_px: f32) -> delta_v_physics::celestial::LazyLoadMesh {
    delta_v_physics::celestial::LazyLoadMesh {
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

fn test(app: &mut App) {
    app.world_mut().spawn((
        RigidBody::new(10.0, 1.0),
        Transform::from_translation(SHIFT),
        sphere_shape(2.0),
        CollisionLayersComponent::new(layers::SHIP),
        DynamicBody,
    ));
    spawn_relevance_asteroid(app, SHIFT + Vec3::new(3.0, 0.0, 0.0), 0.1);
    let events = run_detection_and_collect(app);
    check(
        events.len() == 1,
        &format!(
            "a body without a relevance value must still be tested against everything, got {} events",
            events.len()
        ),
    );
}

fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(&mut app, &[("test", test)]);
}
