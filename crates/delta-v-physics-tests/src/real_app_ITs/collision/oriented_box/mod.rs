// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Oriented box collision tests.

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
use delta_v_physics::systems::PhysicsSet;
use delta_v_test_framework::run_fixed_update;
use delta_v_types::collision::layers;

const SHIFT: Vec3 = Vec3::new(10.0, 0.0, 0.0);

const fn sphere_shape(radius: f32) -> CollisionShape {
    CollisionShape(CollisionShapeData::sphere(radius, Vec3::ZERO))
}

const fn box_shape(half_extents: Vec3) -> CollisionShape {
    CollisionShape(CollisionShapeData::box_shape(half_extents, Vec3::ZERO))
}

const SHIP_HALF_EXTENTS: Vec3 = Vec3::new(4.502, 1.781, 7.908);

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

fn test_pitched_ship_box_hits_a_body_straight_past_its_nose(_app: &mut App) {
    let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    let nose = rotation * Vec3::Z;
    let ship = box_shape(SHIP_HALF_EXTENTS);
    let moon = sphere_shape(0.5);
    let past_the_nose = delta_v_physics::check_collision(
        Vec3::ZERO,
        rotation,
        &ship,
        nose * 8.4,
        Quat::IDENTITY,
        &moon,
    );
    assert!(
        past_the_nose.is_some(),
        "a body past the pitched nose must collide with the oriented box"
    );
    let across_the_thin_axis = delta_v_physics::check_collision(
        Vec3::ZERO,
        rotation,
        &ship,
        (rotation * Vec3::Y) * 8.4,
        Quat::IDENTITY,
        &moon,
    );
    assert!(
        across_the_thin_axis.is_none(),
        "the ship is only {} m across its thin axis, so nothing collides 8.4 m out that way",
        SHIP_HALF_EXTENTS.y
    );
}

fn test_unrotated_reading_of_the_pitched_ship_misses_the_same_body(_app: &mut App) {
    let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    let nose = rotation * Vec3::Z;
    let ship = box_shape(SHIP_HALF_EXTENTS);
    let moon = sphere_shape(0.5);
    let world_aligned = delta_v_physics::check_collision(
        Vec3::ZERO,
        Quat::IDENTITY,
        &ship,
        nose * 8.4,
        Quat::IDENTITY,
        &moon,
    );
    assert!(
        world_aligned.is_none(),
        "ignoring the rotation must lose the collision - that is the bug being fixed"
    );
}

fn test_rotated_boxes_collide_where_axis_aligned_boxes_would_not(_app: &mut App) {
    let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
    let turned = box_shape(Vec3::splat(1.0));
    let small = box_shape(Vec3::splat(0.2));
    let other_pos = Vec3::new(0.0, 0.0, 1.3);
    let rotated = delta_v_physics::check_collision(
        Vec3::ZERO,
        rotation,
        &turned,
        other_pos,
        Quat::IDENTITY,
        &small,
    );
    assert!(
        rotated.is_some(),
        "the turned box reaches z = 1.414, past the near face at 1.1"
    );
    let unrotated = delta_v_physics::check_collision(
        Vec3::ZERO,
        Quat::IDENTITY,
        &turned,
        other_pos,
        Quat::IDENTITY,
        &small,
    );
    assert!(
        unrotated.is_none(),
        "an unturned box stops at z = 1.0 and leaves a gap"
    );
}

fn test_collision_normal_of_a_rotated_box_follows_the_rotated_face(_app: &mut App) {
    let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4);
    let face = rotation * Vec3::X;
    let turned = box_shape(Vec3::splat(1.0));
    let moon = sphere_shape(0.5);
    let result = delta_v_physics::check_collision(
        Vec3::ZERO,
        rotation,
        &turned,
        face * 1.4,
        Quat::IDENTITY,
        &moon,
    );
    assert!(
        result.is_some(),
        "a sphere just off the rotated face collides"
    );
    let (normal, penetration) = result.unwrap();
    assert!(
        normal.dot(face) > 0.99,
        "normal should follow the rotated face {face}, got {normal}"
    );
    assert!(
        (penetration - 0.1).abs() < 0.01,
        "penetration should be ~0.1, got {penetration}"
    );
}

fn test_collision_shape_offset_is_rotated_with_its_body(app: &mut App) {
    let rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    let local_offset = Vec3::Z * 3.0;
    let rotated_offset = rotation * local_offset;
    app.world_mut().spawn((
        RigidBody::new(10.0, 1.0),
        Transform::from_translation(SHIFT).with_rotation(rotation),
        CollisionShape(CollisionShapeData::box_shape(
            Vec3::splat(0.5),
            local_offset,
        )),
        CollisionLayersComponent::new(layers::SHIP),
    ));
    app.world_mut().spawn((
        RigidBody::new(10.0, 1.0),
        Transform::from_translation(SHIFT + rotated_offset),
        sphere_shape(0.2),
        CollisionLayersComponent::new(layers::ASTEROID),
    ));
    run_fixed_update(app);
    let events = app
        .world_mut()
        .resource_mut::<Messages<CollisionDetected>>()
        .drain()
        .collect::<Vec<_>>();
    assert!(
        events.len() == 1,
        "the collision shape offset must follow the body's rotation, got {} events",
        events.len()
    );
}

fn test_pitched_ship_is_stopped_by_a_body_past_its_nose(app: &mut App) {
    register_collision_systems(app);
    let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    let nose = rotation * Vec3::Z;
    let ship = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(SHIFT).with_rotation(rotation),
            box_shape(SHIP_HALF_EXTENTS),
            CollisionLayersComponent::new(layers::SHIP),
            DynamicBody,
        ))
        .id();
    app.world_mut().spawn((
        RigidBody::new(1.0e12, 1.0),
        Transform::from_translation(SHIFT + nose * 8.2),
        sphere_shape(0.5),
        CollisionLayersComponent::new(layers::CELESTIAL),
        StaticBody,
    ));
    run_fixed_update(app);
    let transform = app.world().get::<Transform>(ship).unwrap();
    let moved_along = transform.translation.dot(nose);
    assert!(
        moved_along < 0.0,
        "the ship should be pushed back off the body, but it sits {moved_along} m along its nose"
    );
}

pub fn init_registry() {
    do_register(
        "physics::pitched_ship_box_hits_a_body_straight_past_its_nose",
        test_pitched_ship_box_hits_a_body_straight_past_its_nose,
    );
    do_register(
        "physics::unrotated_reading_of_the_pitched_ship_misses_the_same_body",
        test_unrotated_reading_of_the_pitched_ship_misses_the_same_body,
    );
    do_register(
        "physics::rotated_boxes_collide_where_axis_aligned_boxes_would_not",
        test_rotated_boxes_collide_where_axis_aligned_boxes_would_not,
    );
    do_register(
        "physics::collision_normal_of_a_rotated_box_follows_the_rotated_face",
        test_collision_normal_of_a_rotated_box_follows_the_rotated_face,
    );
    do_register(
        "physics::collision_shape_offset_is_rotated_with_its_body",
        test_collision_shape_offset_is_rotated_with_its_body,
    );
    do_register(
        "physics::pitched_ship_is_stopped_by_a_body_past_its_nose",
        test_pitched_ship_is_stopped_by_a_body_past_its_nose,
    );
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
