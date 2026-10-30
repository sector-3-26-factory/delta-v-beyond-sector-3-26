// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! ADR-0057: a ship and an overlapping moon must collide.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_physics::collision::{
    CollisionDetected, CollisionLayersComponent, CollisionShape, CollisionShapeData, DynamicBody,
    StaticBody,
};
use delta_v_physics::rigid_body::RigidBody;
use delta_v_test_framework::run_fixed_update;
use delta_v_types::collision::layers;

const SHIFT: Vec3 = Vec3::new(10.0, 0.0, 0.0);

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
        RigidBody::new(10_000.0, 1.0),
        Transform::from_translation(SHIFT),
        sphere_shape(5.0),
        CollisionLayersComponent::new(layers::SHIP),
        DynamicBody,
    ));
    app.world_mut().spawn((
        RigidBody::new(1.5e23, 1.0),
        Transform::from_translation(SHIFT + Vec3::Z * -8.0),
        sphere_shape(6.0),
        CollisionLayersComponent::new(layers::CELESTIAL),
        StaticBody,
    ));
    let events = run_detection_and_collect(app);
    assert!(
        events.len() == 1,
        "a ship and an overlapping moon must collide, got {} events",
        events.len()
    );
}

pub fn init_registry() {
    do_register("physics::ship_collides_with_a_celestial_body", test);
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
