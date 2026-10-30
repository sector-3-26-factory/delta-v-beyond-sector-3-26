// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Collision response tests.

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
    CollisionLayersComponent, CollisionShape, CollisionShapeData, DynamicBody, StaticBody,
};
use delta_v_physics::rigid_body::RigidBody;
use delta_v_physics::systems::PhysicsSet;
use delta_v_test_framework::run_fixed_update;
use delta_v_types::collision::layers;

const SHIFT: Vec3 = Vec3::new(10.0, 0.0, 0.0);

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

fn test_collision_response_dynamic_static(app: &mut App) {
    register_collision_systems(app);
    let dynamic = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(SHIFT),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::SHIP),
            DynamicBody,
        ))
        .id();
    app.world_mut().spawn((
        RigidBody::new(1000.0, 1.0),
        Transform::from_translation(SHIFT + Vec3::new(3.0, 0.0, 0.0)),
        sphere_shape(2.0),
        CollisionLayersComponent::new(layers::ASTEROID),
        StaticBody,
    ));
    {
        let mut body = app.world_mut().get_mut::<RigidBody>(dynamic).unwrap();
        body.velocity = Vec3::new(5.0, 0.0, 0.0);
    }
    run_fixed_update(app);
    run_fixed_update(app);
    run_fixed_update(app);
    let body = app.world().get::<RigidBody>(dynamic).unwrap();
    assert!(
        body.velocity.x < 5.0,
        "dynamic body should lose velocity after collision, got {}",
        body.velocity.x
    );
}

fn test_collision_response_dynamic_dynamic(app: &mut App) {
    register_collision_systems(app);
    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(SHIFT),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::PROJECTILE),
            DynamicBody,
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(SHIFT + Vec3::new(3.0, 0.0, 0.0)),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::PROJECTILE),
            DynamicBody,
        ))
        .id();
    {
        let mut body = app.world_mut().get_mut::<RigidBody>(a).unwrap();
        body.velocity = Vec3::new(5.0, 0.0, 0.0);
    }
    for _ in 0..2 {
        run_fixed_update(app);
        run_fixed_update(app);
    }
    let body_a = app.world().get::<RigidBody>(a).unwrap();
    let body_b = app.world().get::<RigidBody>(b).unwrap();
    assert!(
        body_a.velocity.x < 5.0,
        "body A should lose velocity after collision, got {}",
        body_a.velocity.x
    );
    assert!(
        body_b.velocity.x > 0.0,
        "body B should gain velocity in +X direction, got {}",
        body_b.velocity.x
    );
}

fn test_collision_response_separating_velocities_no_impulse(app: &mut App) {
    register_collision_systems(app);
    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(SHIFT),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::PROJECTILE),
            DynamicBody,
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(SHIFT + Vec3::new(3.0, 0.0, 0.0)),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::PROJECTILE),
            DynamicBody,
        ))
        .id();
    {
        let mut body = app.world_mut().get_mut::<RigidBody>(a).unwrap();
        body.velocity = Vec3::new(-5.0, 0.0, 0.0);
    }
    {
        let mut body = app.world_mut().get_mut::<RigidBody>(b).unwrap();
        body.velocity = Vec3::new(5.0, 0.0, 0.0);
    }
    run_fixed_update(app);
    let body_a = app.world().get::<RigidBody>(a).unwrap();
    let body_b = app.world().get::<RigidBody>(b).unwrap();
    assert!(
        (body_a.velocity.x - (-5.0)).abs() < 0.01,
        "separating body A should keep its velocity, got {}",
        body_a.velocity.x
    );
    assert!(
        (body_b.velocity.x - 5.0).abs() < 0.01,
        "separating body B should keep its velocity, got {}",
        body_b.velocity.x
    );
}

fn test_collision_response_position_correction(app: &mut App) {
    register_collision_systems(app);
    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(SHIFT),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::PROJECTILE),
            DynamicBody,
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(SHIFT + Vec3::new(3.0, 0.0, 0.0)),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::PROJECTILE),
            DynamicBody,
        ))
        .id();
    run_fixed_update(app);
    let transform_a = app.world().get::<Transform>(a).unwrap();
    let transform_b = app.world().get::<Transform>(b).unwrap();
    let distance = (transform_b.translation - transform_a.translation).length();
    assert!(
        distance > 2.5,
        "bodies should be pushed apart by position correction, distance={distance}"
    );
}

fn test_collision_response_static_static_skipped(app: &mut App) {
    register_collision_systems(app);
    app.world_mut().spawn((
        RigidBody::new(100.0, 1.0),
        Transform::from_translation(SHIFT),
        sphere_shape(2.0),
        CollisionLayersComponent::new(layers::ASTEROID),
        StaticBody,
    ));
    app.world_mut().spawn((
        RigidBody::new(100.0, 1.0),
        Transform::from_translation(SHIFT + Vec3::new(3.0, 0.0, 0.0)),
        sphere_shape(2.0),
        CollisionLayersComponent::new(layers::ASTEROID),
        StaticBody,
    ));
    run_fixed_update(app);
}

pub fn init_registry() {
    do_register(
        "physics::collision_response_dynamic_static",
        test_collision_response_dynamic_static,
    );
    do_register(
        "physics::collision_response_dynamic_dynamic",
        test_collision_response_dynamic_dynamic,
    );
    do_register(
        "physics::collision_response_separating_velocities_no_impulse",
        test_collision_response_separating_velocities_no_impulse,
    );
    do_register(
        "physics::collision_response_position_correction",
        test_collision_response_position_correction,
    );
    do_register(
        "physics::collision_response_static_static_skipped",
        test_collision_response_static_static_skipped,
    );
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
