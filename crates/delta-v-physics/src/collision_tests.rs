// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for collision detection and response.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::panic
)]

use bevy::prelude::*;
use bevy::time::TimePlugin;

use crate::collision::{
    CollisionDetected, CollisionLayersComponent, CollisionShape, CollisionShapeData,
    CollisionShapeType, DynamicBody, StaticBody,
};
use crate::rigid_body::RigidBody;
use crate::systems::PhysicsSet;
use delta_v_types::collision::layers;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds a minimal Bevy app with fixed timestep and physics sets configured.
fn build_collision_app() -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.add_message::<CollisionDetected>();
    app.configure_sets(
        FixedUpdate,
        (
            PhysicsSet::AccumulateForces,
            PhysicsSet::IntegrateVelocity,
            PhysicsSet::IntegratePosition,
            PhysicsSet::ClearAccumulators,
            CollisionResponseSet,
        )
            .chain(),
    );
    app
}

/// Runs one fixed update tick.
fn run_fixed_update(app: &mut App) {
    let fixed_duration = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(fixed_duration);
    app.update();
}

/// Creates a sphere `CollisionShape` component.
fn sphere_shape(radius: f32) -> CollisionShape {
    CollisionShape(CollisionShapeData::sphere(radius, Vec3::ZERO))
}

/// Creates a box `CollisionShape` component.
fn box_shape(half_extents: Vec3) -> CollisionShape {
    CollisionShape(CollisionShapeData::box_shape(half_extents, Vec3::ZERO))
}

// ---------------------------------------------------------------------------
// Collision detection: sphere-sphere
// ---------------------------------------------------------------------------

#[test]
fn test_sphere_sphere_overlap() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(2.0),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
            sphere_shape(2.0),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some(), "spheres should collide");
    let (normal, penetration) = result.unwrap();
    assert!(
        (penetration - 1.0).abs() < 0.01,
        "penetration should be ~1.0, got {penetration}"
    );
    assert!(
        (normal.x - 1.0).abs() < 0.01,
        "normal should point +X, got {normal}"
    );
}

#[test]
fn test_sphere_sphere_no_overlap() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(1.0),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
            sphere_shape(1.0),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_none(), "spheres should not collide");
}

#[test]
fn test_sphere_sphere_touching_exactly() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(1.0),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            sphere_shape(1.0),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(
        result.is_none(),
        "exactly touching spheres should not collide"
    );
}

#[test]
fn test_sphere_sphere_coincident_centers() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(2.0),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(3.0),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some(), "coincident spheres should collide");
    let (normal, penetration) = result.unwrap();
    assert!(
        (penetration - 5.0).abs() < 0.01,
        "penetration should be 5.0, got {penetration}"
    );
    assert!(
        (normal.z - 1.0).abs() < 0.01,
        "normal should be +Z for coincident, got {normal}"
    );
}

// ---------------------------------------------------------------------------
// Collision detection: box-box
// ---------------------------------------------------------------------------

#[test]
fn test_box_box_overlap() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            box_shape(Vec3::new(2.0, 2.0, 2.0)),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
            box_shape(Vec3::new(2.0, 2.0, 2.0)),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some(), "boxes should collide");
    let (normal, penetration) = result.unwrap();
    assert!(
        (penetration - 1.0).abs() < 0.01,
        "penetration should be ~1.0, got {penetration}"
    );
    assert!(
        (normal.x - 1.0).abs() < 0.01,
        "normal should point +X, got {normal}"
    );
}

#[test]
fn test_box_box_no_overlap() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            box_shape(Vec3::new(1.0, 1.0, 1.0)),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
            box_shape(Vec3::new(1.0, 1.0, 1.0)),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_none(), "boxes should not collide");
}

#[test]
fn test_box_box_minimum_penetration_axis() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            box_shape(Vec3::new(2.0, 2.0, 2.0)),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(3.0, 1.0, 0.0)),
            box_shape(Vec3::new(2.0, 2.0, 2.0)),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some());
    let (normal, penetration) = result.unwrap();
    assert!(
        (penetration - 1.0).abs() < 0.01,
        "min penetration should be 1.0 on X, got {penetration}"
    );
    assert!(
        (normal.x - 1.0).abs() < 0.01,
        "normal should be +X (min penetration axis), got {normal}"
    );
}

// ---------------------------------------------------------------------------
// Collision detection: sphere-box
// ---------------------------------------------------------------------------

#[test]
fn test_sphere_box_overlap() {
    let mut app = build_collision_app();

    // Sphere at origin with radius 3
    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(3.0),
        ))
        .id();

    // Box at (3,0,0) with half_extents (1,1,1). Closest box point to sphere center is (1,0,0).
    // Distance = 2 < radius 3 → overlap
    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
            box_shape(Vec3::new(1.0, 1.0, 1.0)),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some(), "sphere and box should collide");
    let (_normal, penetration) = result.unwrap();
    assert!(penetration > 0.0, "penetration should be positive");
}

#[test]
fn test_sphere_box_no_overlap() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(1.0),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
            box_shape(Vec3::new(1.0, 1.0, 1.0)),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_none(), "sphere and box should not collide");
}

#[test]
fn test_sphere_box_normal_direction() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(3.0),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
            box_shape(Vec3::new(1.0, 1.0, 1.0)),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some());
    let (normal, _penetration) = result.unwrap();
    assert!(
        normal.x > 0.9,
        "normal should point +X from sphere to box, got {normal}"
    );
}

#[test]
fn test_sphere_inside_box() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(0.5),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            box_shape(Vec3::new(2.0, 2.0, 2.0)),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some(), "sphere inside box should collide");
}

// ---------------------------------------------------------------------------
// Collision detection: normal direction
// ---------------------------------------------------------------------------

#[test]
fn test_collision_normal_points_from_a_to_b() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(2.0),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(0.0, 3.0, 0.0)),
            sphere_shape(2.0),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some());
    let (normal, _penetration) = result.unwrap();
    assert!(
        (normal.y - 1.0).abs() < 0.01,
        "normal should point +Y from A to B, got {normal}"
    );
}

#[test]
fn test_collision_normal_negative_direction() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(2.0),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
            sphere_shape(2.0),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some());
    let (normal, _penetration) = result.unwrap();
    assert!(
        (normal.x + 1.0).abs() < 0.01,
        "normal should point -X from A to B, got {normal}"
    );
}

// ---------------------------------------------------------------------------
// Collision detection: penetration depth
// ---------------------------------------------------------------------------

#[test]
fn test_penetration_depth_sphere_sphere() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(3.0),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)),
            sphere_shape(3.0),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some());
    let (_normal, penetration) = result.unwrap();
    assert!(
        (penetration - 2.0).abs() < 0.01,
        "penetration should be 2.0, got {penetration}"
    );
}

// ---------------------------------------------------------------------------
// Collision detection: system-level test via events
// ---------------------------------------------------------------------------

#[test]
fn test_collision_response_dynamic_static() {
    let mut app = build_collision_app();

    // Register detection and response systems in the same set.
    // They will run in parallel within the set, but events from detection
    // are available to response in the same frame.
    app.add_systems(
        FixedUpdate,
        (
            crate::collision_detection_system.in_set(PhysicsSet::AccumulateForces),
            crate::collision_response_system.in_set(CollisionResponseSet),
        ),
    );

    let dynamic = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::SHIP),
            DynamicBody,
        ))
        .id();

    app.world_mut().spawn((
        RigidBody::new(1000.0, 1.0),
        Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
        sphere_shape(2.0),
        CollisionLayersComponent::new(layers::ASTEROID),
        StaticBody,
    ));

    {
        let mut body = app.world_mut().get_mut::<RigidBody>(dynamic).unwrap();
        body.velocity = Vec3::new(5.0, 0.0, 0.0);
    }

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let body = app.world().get::<RigidBody>(dynamic).unwrap();
    assert!(
        body.velocity.x < 5.0,
        "dynamic body should lose velocity after collision, got {}",
        body.velocity.x
    );
}

#[test]
fn test_collision_response_dynamic_dynamic() {
    let mut app = build_collision_app();

    app.add_systems(
        FixedUpdate,
        (
            crate::collision_detection_system.in_set(PhysicsSet::AccumulateForces),
            crate::collision_response_system.in_set(CollisionResponseSet),
        ),
    );

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::PROJECTILE),
            DynamicBody,
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::PROJECTILE),
            DynamicBody,
        ))
        .id();

    {
        let mut body = app.world_mut().get_mut::<RigidBody>(a).unwrap();
        body.velocity = Vec3::new(5.0, 0.0, 0.0);
    }

    {
        for _ in 0..2 {
            run_fixed_update(&mut app);
            run_fixed_update(&mut app);
        }
    };

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

#[test]
fn test_collision_response_separating_velocities_no_impulse() {
    let mut app = build_collision_app();

    app.add_systems(
        FixedUpdate,
        (
            crate::collision_detection_system.in_set(PhysicsSet::AccumulateForces),
            crate::collision_response_system.in_set(CollisionResponseSet),
        ),
    );

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::PROJECTILE),
            DynamicBody,
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
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

    run_fixed_update(&mut app);

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

#[test]
fn test_collision_response_position_correction() {
    let mut app = build_collision_app();

    app.add_systems(
        FixedUpdate,
        (
            crate::collision_detection_system.in_set(PhysicsSet::AccumulateForces),
            crate::collision_response_system.in_set(CollisionResponseSet),
        ),
    );

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::PROJECTILE),
            DynamicBody,
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
            sphere_shape(2.0),
            CollisionLayersComponent::new(layers::PROJECTILE),
            DynamicBody,
        ))
        .id();

    run_fixed_update(&mut app);

    let transform_a = app.world().get::<Transform>(a).unwrap();
    let transform_b = app.world().get::<Transform>(b).unwrap();
    let distance = (transform_b.translation - transform_a.translation).length();
    assert!(
        distance > 2.5,
        "bodies should be pushed apart by position correction, distance={distance}"
    );
}

#[test]
fn test_collision_response_static_static_skipped() {
    let mut app = build_collision_app();

    app.add_systems(
        FixedUpdate,
        (
            crate::collision_detection_system.in_set(PhysicsSet::AccumulateForces),
            crate::collision_response_system.in_set(CollisionResponseSet),
        ),
    );

    app.world_mut().spawn((
        RigidBody::new(100.0, 1.0),
        Transform::from_translation(Vec3::ZERO),
        sphere_shape(2.0),
        CollisionLayersComponent::new(layers::ASTEROID),
        StaticBody,
    ));

    app.world_mut().spawn((
        RigidBody::new(100.0, 1.0),
        Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
        sphere_shape(2.0),
        CollisionLayersComponent::new(layers::ASTEROID),
        StaticBody,
    ));

    run_fixed_update(&mut app);
}

// ---------------------------------------------------------------------------
// Collision shape helpers
// ---------------------------------------------------------------------------

#[test]
fn test_collision_shape_sphere_constructor() {
    let shape = CollisionShape::sphere(5.0, Vec3::new(1.0, 2.0, 3.0));
    match shape.shape_type {
        CollisionShapeType::Sphere { radius } => assert!((radius - 5.0).abs() < 0.01),
        _ => panic!("expected sphere shape"),
    }
    assert_eq!(shape.offset, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_collision_shape_box_constructor() {
    let he = Vec3::new(1.0, 2.0, 3.0);
    let offset = Vec3::new(0.5, 0.5, 0.5);
    let shape = CollisionShape::box_shape(he, offset);
    match shape.shape_type {
        CollisionShapeType::Box { half_extents } => assert_eq!(half_extents, he),
        _ => panic!("expected box shape"),
    }
    assert_eq!(shape.offset, offset);
}

// ---------------------------------------------------------------------------
// Collision detection: offset handling
// Note: check_collision does NOT add offsets — the caller must add them.
// These tests manually add offsets to match what collision_detection_system does.
// ---------------------------------------------------------------------------

#[test]
fn test_collision_with_offset() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            CollisionShape(CollisionShapeData::sphere(2.0, Vec3::new(2.0, 0.0, 0.0))),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            CollisionShape(CollisionShapeData::sphere(2.0, Vec3::new(-2.0, 0.0, 0.0))),
        ))
        .id();

    // Manually add offsets, just like collision_detection_system does
    let transform_a = app.world().get::<Transform>(a).unwrap();
    let transform_b = app.world().get::<Transform>(b).unwrap();
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let pos_a = transform_a.translation + shape_a.offset;
    let pos_b = transform_b.translation + shape_b.offset;

    // Effective positions: A at (2,0,0), B at (-2,0,0). Distance = 4, sum radii = 4 → touching
    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(
        result.is_none(),
        "spheres with offset should be exactly touching, not overlapping"
    );
}

#[test]
fn test_collision_with_offset_overlap() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            CollisionShape(CollisionShapeData::sphere(2.0, Vec3::new(1.5, 0.0, 0.0))),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            CollisionShape(CollisionShapeData::sphere(2.0, Vec3::new(-1.5, 0.0, 0.0))),
        ))
        .id();

    // Manually add offsets, just like collision_detection_system does
    let transform_a = app.world().get::<Transform>(a).unwrap();
    let transform_b = app.world().get::<Transform>(b).unwrap();
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let pos_a = transform_a.translation + shape_a.offset;
    let pos_b = transform_b.translation + shape_b.offset;

    // Effective positions: A at (1.5,0,0), B at (-1.5,0,0). Distance = 3, sum radii = 4 → overlap = 1
    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some(), "spheres with offset should overlap");
    let (_normal, penetration) = result.unwrap();
    assert!(
        (penetration - 1.0).abs() < 0.01,
        "penetration should be ~1.0, got {penetration}"
    );
}

// ---------------------------------------------------------------------------
// Collision detection: convex hull fallback
// ---------------------------------------------------------------------------

#[test]
fn test_convex_hull_fallback_uses_sphere_approximation() {
    let mut app = build_collision_app();

    let a = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            CollisionShape(CollisionShapeData {
                shape_type: CollisionShapeType::ConvexHull,
                offset: Vec3::ZERO,
            }),
        ))
        .id();

    let b = app
        .world_mut()
        .spawn((
            RigidBody::new(10.0, 1.0),
            Transform::from_translation(Vec3::new(1.5, 0.0, 0.0)),
            CollisionShape(CollisionShapeData {
                shape_type: CollisionShapeType::ConvexHull,
                offset: Vec3::ZERO,
            }),
        ))
        .id();

    let pos_a = app.world().get::<Transform>(a).unwrap().translation;
    let pos_b = app.world().get::<Transform>(b).unwrap().translation;
    let shape_a = app.world().get::<CollisionShape>(a).unwrap();
    let shape_b = app.world().get::<CollisionShape>(b).unwrap();

    let result = crate::check_collision(pos_a, shape_a, pos_b, shape_b);
    assert!(
        result.is_some(),
        "convex hull fallback should detect collision via sphere approx"
    );
}

/// System set for collision response, runs after collision detection.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct CollisionResponseSet;
