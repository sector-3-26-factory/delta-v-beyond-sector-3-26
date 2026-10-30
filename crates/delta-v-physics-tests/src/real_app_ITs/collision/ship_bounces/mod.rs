// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! ADR-0057: a ship bounces off a static moon.

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
    assert!(
        ship_velocity.z > 0.0,
        "the ship should rebound, its z velocity was {}",
        ship_velocity.z
    );
    assert!(
        moon_velocity == Vec3::ZERO,
        "the moon must not be moved, got {moon_velocity:?}"
    );
}

pub fn init_registry() {
    do_register("physics::ship_bounces_off_a_static_moon", test);
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
