// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! ADR-0057: a ship bounces off a static moon.
//!
//!   xvfb-run -a cargo test --test `ship_bounces_off_a_static_moon` -p delta-v-physics

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
    CollisionLayersComponent, CollisionShape, CollisionShapeData, DynamicBody, StaticBody,
};
use delta_v_physics::rigid_body::RigidBody;
use delta_v_physics::systems::PhysicsSet;
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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct CollisionResponseSet;

fn register_collision_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            delta_v_physics::collision_detection_system.in_set(PhysicsSet::AccumulateForces),
            delta_v_physics::collision_response_system.in_set(CollisionResponseSet),
        ),
    );
}

fn test(app: &mut App) {
    register_collision_systems(app);
    let ship = app
        .world_mut()
        .spawn((
            RigidBody::new(10_000.0, 1.0),
            Transform::from_translation(SHIFT),
            sphere_shape(5.0),
            CollisionLayersComponent::new(layers::SHIP),
            DynamicBody,
        ))
        .id();
    let moon = app
        .world_mut()
        .spawn((
            RigidBody::new(1.5e23, 1.0),
            Transform::from_translation(SHIFT + Vec3::Z * -8.0),
            sphere_shape(6.0),
            CollisionLayersComponent::new(layers::CELESTIAL),
            StaticBody,
        ))
        .id();
    app.world_mut().get_mut::<RigidBody>(ship).unwrap().velocity = Vec3::Z * -60.0;
    run_fixed_update(app);
    let ship_velocity = app.world().get::<RigidBody>(ship).unwrap().velocity;
    let moon_velocity = app.world().get::<RigidBody>(moon).unwrap().velocity;
    check(
        ship_velocity.z > 0.0,
        &format!(
            "the ship should rebound, its z velocity was {}",
            ship_velocity.z
        ),
    );
    check(
        moon_velocity == Vec3::ZERO,
        &format!("the moon must not be moved, got {moon_velocity:?}"),
    );
}

fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(&mut app, &[("test", test)]);
}
