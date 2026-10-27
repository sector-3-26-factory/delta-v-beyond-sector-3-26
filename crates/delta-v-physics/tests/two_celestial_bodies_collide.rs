// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! ADR-0057: two overlapping moons must collide.
//!
//!   xvfb-run -a cargo test --test `two_celestial_bodies_collide` -p delta-v-physics

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
    CollisionDetected, CollisionLayersComponent, CollisionShape, CollisionShapeData, StaticBody,
};
use delta_v_physics::rigid_body::RigidBody;
use delta_v_test_support::{check, make_app, run_fixed_update, run_harness};
use delta_v_types::collision::layers;

const SHIFT: Vec3 = Vec3::new(10_000_000.0, 0.0, 0.0);

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

fn run_detection_and_collect(app: &mut App) -> Vec<CollisionDetected> {
    run_fixed_update(app);
    app.world_mut()
        .resource_mut::<Messages<CollisionDetected>>()
        .drain()
        .collect()
}

fn test(app: &mut App) {
    app.world_mut().spawn((
        RigidBody::new(1.0e23, 1.0),
        Transform::from_translation(SHIFT),
        sphere_shape(1.0e6),
        CollisionLayersComponent::new(layers::CELESTIAL),
        StaticBody,
    ));
    app.world_mut().spawn((
        RigidBody::new(1.0e22, 1.0),
        Transform::from_translation(SHIFT + Vec3::Z * -1.5e6),
        sphere_shape(1.0e6),
        CollisionLayersComponent::new(layers::CELESTIAL),
        StaticBody,
    ));
    let events = run_detection_and_collect(app);
    check(
        events.len() == 1,
        &format!(
            "two overlapping moons must collide, got {} events",
            events.len()
        ),
    );
}

fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(&mut app, &[("test", test)]);
}
