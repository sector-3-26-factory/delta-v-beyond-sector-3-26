// AGENTS: before modifying this file, read AGENTS.md at the repository root.

#![cfg(test)]

//! Tests for collision detection and response.

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

/// Calls [`crate::check_collision`] with both bodies unrotated.
///
/// Every test below that predates oriented collision support spawns its bodies with
/// `Transform::default()`, whose rotation is the identity, so identity is exactly
/// the rotation those tests mean. The rotation tests further down pass a real
/// rotation to [`crate::check_collision`] instead of using this helper.
fn check_unrotated(
    pos_a: Vec3,
    shape_a: &CollisionShape,
    pos_b: Vec3,
    shape_b: &CollisionShape,
) -> Option<(Vec3, f32)> {
    crate::check_collision(
        pos_a,
        Quat::IDENTITY,
        shape_a,
        pos_b,
        Quat::IDENTITY,
        shape_b,
    )
}

/// Builds an app that runs only the collision detection system, so a test can
/// assert on the `CollisionDetected` messages it emits and nothing else.
fn build_detection_app() -> App {
    let mut app = App::new();
    app.add_message::<CollisionDetected>();
    app.add_systems(FixedUpdate, crate::collision_detection_system);
    app
}

/// Runs one tick of `FixedUpdate` and drains the `CollisionDetected` messages.
fn run_detection_and_collect(app: &mut App) -> Vec<CollisionDetected> {
    app.world_mut().run_schedule(FixedUpdate);
    app.world_mut()
        .resource_mut::<Messages<CollisionDetected>>()
        .drain()
        .collect()
}

/// Builds a `LazyLoadMesh` carrying an explicit relevance verdict.
///
/// `screen_radius_px` is written directly rather than derived from a camera: the value
/// is the only thing the filter reads, and it is a pure function of distance.
fn lazy_mesh(screen_radius_px: f32) -> crate::celestial::LazyLoadMesh {
    crate::celestial::LazyLoadMesh {
        min_screen_radius_px: 1.0,
        loaded: true,
        mesh_path: String::new(),
        current_screen_radius_px: screen_radius_px,
        collision_relevance_px: crate::constants::COLLISION_RELEVANCE_PX,
        mesh_child_entities: Vec::new(),
    }
}

/// Spawns an asteroid at `position` whose apparent radius is `screen_radius_px`.
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

// ---------------------------------------------------------------------------
// Collision detection: sphere-sphere
// ---------------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
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
#[allow(clippy::unwrap_used)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_none(), "spheres should not collide");
}

#[test]
#[allow(clippy::unwrap_used)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
    assert!(
        result.is_none(),
        "exactly touching spheres should not collide"
    );
}

#[test]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
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
#[allow(clippy::unwrap_used, clippy::float_cmp)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
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
#[allow(clippy::unwrap_used)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_none(), "boxes should not collide");
}

#[test]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
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
#[allow(clippy::unwrap_used)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some(), "sphere and box should collide");
    let (_normal, penetration) = result.unwrap();
    assert!(penetration > 0.0, "penetration should be positive");
}

#[test]
#[allow(clippy::unwrap_used)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_none(), "sphere and box should not collide");
}

#[test]
#[allow(clippy::unwrap_used)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some());
    let (normal, _penetration) = result.unwrap();
    assert!(
        normal.x > 0.9,
        "normal should point +X from sphere to box, got {normal}"
    );
}

#[test]
#[allow(clippy::unwrap_used)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some(), "sphere inside box should collide");
}

// ---------------------------------------------------------------------------
// Collision detection: normal direction
// ---------------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
    assert!(result.is_some());
    let (normal, _penetration) = result.unwrap();
    assert!(
        (normal.y - 1.0).abs() < 0.01,
        "normal should point +Y from A to B, got {normal}"
    );
}

#[test]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
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
#[allow(clippy::unwrap_used, clippy::float_cmp)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
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
#[allow(clippy::unwrap_used)]
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
            CollisionLayersComponent::new(delta_v_types::collision::layers::SHIP),
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
#[allow(clippy::unwrap_used)]
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
#[allow(clippy::unwrap_used, clippy::float_cmp)]
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
#[allow(clippy::unwrap_used)]
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
#[allow(clippy::unwrap_used)]
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
#[allow(clippy::float_cmp, clippy::panic)]
fn test_collision_shape_sphere_constructor() {
    let shape = CollisionShape::sphere(5.0, Vec3::new(1.0, 2.0, 3.0));
    match shape.shape_type {
        CollisionShapeType::Sphere { radius } => assert!((radius - 5.0).abs() < 0.01),
        _ => panic!("expected sphere shape"),
    }
    assert_eq!(shape.offset, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
#[allow(clippy::panic)]
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
#[allow(clippy::unwrap_used)]
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
    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
    assert!(
        result.is_none(),
        "spheres with offset should be exactly touching, not overlapping"
    );
}

#[test]
#[allow(clippy::unwrap_used, clippy::float_cmp)]
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
    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
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
#[allow(clippy::unwrap_used)]
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

    let result = check_unrotated(pos_a, shape_a, pos_b, shape_b);
    assert!(
        result.is_some(),
        "convex hull fallback should detect collision via sphere approx"
    );
}

// ---------------------------------------------------------------------------
// Distance to surface calculation
// ---------------------------------------------------------------------------

#[test]
#[allow(clippy::float_cmp)]
fn test_distance_to_surface_sphere_outside() {
    // Point at (10, 0, 0), sphere at origin with radius 3
    // Distance to surface = 10 - 3 = 7
    let point = Vec3::new(10.0, 0.0, 0.0);
    let entity_pos = Vec3::ZERO;
    let shape = sphere_shape(3.0);

    let distance = crate::distance_to_surface(point, entity_pos, &shape);
    assert!(
        (distance - 7.0).abs() < 0.01,
        "distance to sphere surface should be 7, got {distance}"
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn test_distance_to_surface_sphere_inside() {
    // Point at (1, 0, 0), sphere at origin with radius 3
    // Point is inside sphere, distance to surface = 0
    let point = Vec3::new(1.0, 0.0, 0.0);
    let entity_pos = Vec3::ZERO;
    let shape = sphere_shape(3.0);

    let distance = crate::distance_to_surface(point, entity_pos, &shape);
    assert!(
        (distance - 0.0).abs() < 0.01,
        "distance to sphere surface when inside should be 0, got {distance}"
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn test_distance_to_surface_sphere_on_surface() {
    // Point at (3, 0, 0), sphere at origin with radius 3
    // Point is on surface, distance to surface = 0
    let point = Vec3::new(3.0, 0.0, 0.0);
    let entity_pos = Vec3::ZERO;
    let shape = sphere_shape(3.0);

    let distance = crate::distance_to_surface(point, entity_pos, &shape);
    assert!(
        (distance - 0.0).abs() < 0.01,
        "distance to sphere surface when on surface should be 0, got {distance}"
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn test_distance_to_surface_box_outside() {
    // Point at (5, 0, 0), box at origin with half_extents (2, 2, 2)
    // Closest point on box is (2, 0, 0), distance = 3
    let point = Vec3::new(5.0, 0.0, 0.0);
    let entity_pos = Vec3::ZERO;
    let shape = box_shape(Vec3::new(2.0, 2.0, 2.0));

    let distance = crate::distance_to_surface(point, entity_pos, &shape);
    assert!(
        (distance - 3.0).abs() < 0.01,
        "distance to box surface should be 3, got {distance}"
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn test_distance_to_surface_box_inside() {
    // Point at (1, 1, 1), box at origin with half_extents (2, 2, 2)
    // Point is inside box, distance to surface = 0
    let point = Vec3::new(1.0, 1.0, 1.0);
    let entity_pos = Vec3::ZERO;
    let shape = box_shape(Vec3::new(2.0, 2.0, 2.0));

    let distance = crate::distance_to_surface(point, entity_pos, &shape);
    assert!(
        (distance - 0.0).abs() < 0.01,
        "distance to box surface when inside should be 0, got {distance}"
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn test_distance_to_surface_box_corner() {
    // Point at (5, 5, 5), box at origin with half_extents (2, 2, 2)
    // Closest point on box is (2, 2, 2), distance = sqrt(3^2 + 3^2 + 3^2) = sqrt(27) ≈ 5.196
    let point = Vec3::new(5.0, 5.0, 5.0);
    let entity_pos = Vec3::ZERO;
    let shape = box_shape(Vec3::new(2.0, 2.0, 2.0));

    let distance = crate::distance_to_surface(point, entity_pos, &shape);
    let expected = 3.0 * 3.0_f32.sqrt();
    assert!(
        (distance - expected).abs() < 0.01,
        "distance to box corner should be ~{expected}, got {distance}"
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn test_distance_to_surface_box_with_offset() {
    // Point at (10, 0, 0), box at (5, 0, 0) with half_extents (2, 2, 2) and offset (0, 0, 0)
    // Box center is at (5, 0, 0), closest point is (7, 0, 0), distance = 3
    let point = Vec3::new(10.0, 0.0, 0.0);
    let entity_pos = Vec3::new(5.0, 0.0, 0.0);
    let shape = box_shape(Vec3::new(2.0, 2.0, 2.0));

    let distance = crate::distance_to_surface(point, entity_pos, &shape);
    assert!(
        (distance - 3.0).abs() < 0.01,
        "distance to box surface with offset should be 3, got {distance}"
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn test_distance_to_surface_convex_hull_placeholder() {
    // Convex hull returns center distance as placeholder
    // Point at (10, 0, 0), entity at origin
    // Distance = 10 (center distance)
    let point = Vec3::new(10.0, 0.0, 0.0);
    let entity_pos = Vec3::ZERO;
    let shape = CollisionShape(CollisionShapeData {
        shape_type: CollisionShapeType::ConvexHull,
        offset: Vec3::ZERO,
    });

    let distance = crate::distance_to_surface(point, entity_pos, &shape);
    assert!(
        (distance - 10.0).abs() < 0.01,
        "distance to convex hull should be center distance (10), got {distance}"
    );
}

// ---------------------------------------------------------------------------
// ADR-0057: collision relevance filter
// ---------------------------------------------------------------------------

/// A collision between two bodies that are both above the threshold is still detected.
#[test]
fn test_collision_relevant_asteroid_pair_still_collides() {
    let mut app = build_detection_app();

    spawn_relevance_asteroid(&mut app, Vec3::ZERO, 20.0);
    spawn_relevance_asteroid(&mut app, Vec3::new(3.0, 0.0, 0.0), 20.0);

    let events = run_detection_and_collect(&mut app);
    assert_eq!(
        events.len(),
        1,
        "two relevant asteroids overlapping must still be detected"
    );
}

/// Two asteroids both below the threshold are never tested, so no collision is reported.
#[test]
fn test_collision_irrelevant_asteroid_pair_is_skipped() {
    let mut app = build_detection_app();

    spawn_relevance_asteroid(&mut app, Vec3::ZERO, 0.5);
    spawn_relevance_asteroid(&mut app, Vec3::new(3.0, 0.0, 0.0), 0.5);

    let events = run_detection_and_collect(&mut app);
    assert!(
        events.is_empty(),
        "two irrelevant asteroids must not be tested, got {} events",
        events.len()
    );
}

/// Asteroid/asteroid pairs require BOTH sides relevant, so one relevant and one
/// irrelevant asteroid does not produce a collision.
#[test]
fn test_collision_asteroid_pair_requires_both_relevant() {
    let mut app = build_detection_app();

    spawn_relevance_asteroid(&mut app, Vec3::ZERO, 20.0);
    spawn_relevance_asteroid(&mut app, Vec3::new(3.0, 0.0, 0.0), 0.5);

    let events = run_detection_and_collect(&mut app);
    assert!(
        events.is_empty(),
        "asteroid pairs need both sides relevant, got {} events",
        events.len()
    );
}

/// A body with no `LazyLoadMesh` has no relevance value and is always tested.
/// A ship must therefore still hit an asteroid that is below the threshold.
#[test]
fn test_collision_body_without_lazy_mesh_is_always_tested() {
    let mut app = build_detection_app();

    // No LazyLoadMesh on the ship, and the asteroid is far below the threshold.
    app.world_mut().spawn((
        RigidBody::new(10.0, 1.0),
        Transform::from_translation(Vec3::ZERO),
        sphere_shape(2.0),
        CollisionLayersComponent::new(delta_v_types::collision::layers::SHIP),
        DynamicBody,
    ));
    spawn_relevance_asteroid(&mut app, Vec3::new(3.0, 0.0, 0.0), 0.1);

    let events = run_detection_and_collect(&mut app);
    assert_eq!(
        events.len(),
        1,
        "a body without a relevance value must still be tested against everything"
    );
}

/// The threshold itself is inclusive: a body exactly at the threshold is relevant.
#[test]
#[allow(clippy::unwrap_used)]
fn test_collision_relevance_threshold_is_inclusive() {
    let mut app = build_detection_app();
    let threshold = crate::constants::COLLISION_RELEVANCE_PX;

    spawn_relevance_asteroid(&mut app, Vec3::ZERO, threshold);
    spawn_relevance_asteroid(&mut app, Vec3::new(3.0, 0.0, 0.0), threshold);

    let events = run_detection_and_collect(&mut app);
    assert_eq!(
        events.len(),
        1,
        "a body exactly at the threshold must count as relevant"
    );
}

/// A mixed pair needs only one relevant side, so an asteroid that is relevant still
/// collides with a ship that carries no relevance value at all.
/// A ship and a moon must be tested against each other.
///
/// A moon once carried a `CollisionShape` but no `CollisionLayersComponent`.
/// The detection query requires both, so the moon was in no bucket, nothing
/// was ever compared against it, and a ship flew straight through.
#[test]
#[allow(clippy::unwrap_used)]
fn test_ship_collides_with_a_celestial_body() {
    let mut app = build_detection_app();
    app.world_mut().spawn((
        RigidBody::new(10_000.0, 1.0),
        Transform::from_translation(Vec3::ZERO),
        sphere_shape(5.0),
        CollisionLayersComponent::new(delta_v_types::collision::layers::SHIP),
        DynamicBody,
    ));
    app.world_mut().spawn((
        RigidBody::new(1.5e23, 1.0),
        Transform::from_translation(Vec3::Z * -8.0),
        sphere_shape(6.0),
        CollisionLayersComponent::new(layers::CELESTIAL),
        StaticBody,
    ));

    let events = run_detection_and_collect(&mut app);
    assert_eq!(
        events.len(),
        1,
        "a ship and an overlapping moon must collide"
    );
}

/// Two moons that touch must collide, like any other pair.
///
/// Nothing is excluded from collision. A pair of bodies is only skipped when
/// their shapes do not overlap.
#[test]
#[allow(clippy::unwrap_used)]
fn test_two_celestial_bodies_collide() {
    let mut app = build_detection_app();
    app.world_mut().spawn((
        RigidBody::new(1.0e23, 1.0),
        Transform::from_translation(Vec3::ZERO),
        sphere_shape(1.0e6),
        CollisionLayersComponent::new(layers::CELESTIAL),
        StaticBody,
    ));
    app.world_mut().spawn((
        RigidBody::new(1.0e22, 1.0),
        Transform::from_translation(Vec3::Z * -1.5e6),
        sphere_shape(1.0e6),
        CollisionLayersComponent::new(layers::CELESTIAL),
        StaticBody,
    ));

    assert_eq!(
        run_detection_and_collect(&mut app).len(),
        1,
        "two overlapping moons must collide"
    );
}

/// Every body collides with every other body.
#[test]
#[allow(clippy::unwrap_used)]
fn test_every_layer_collides_with_every_other() {
    let all = [
        ("ship", layers::SHIP),
        ("asteroid", layers::ASTEROID),
        ("projectile", layers::PROJECTILE),
        ("celestial", layers::CELESTIAL),
    ];
    for (name, body) in all {
        assert_eq!(
            body.mask,
            layers::ALL_LAYERS,
            "{name} excludes something from collision"
        );
        for (other_name, other) in all {
            assert_ne!(
                body.mask & other.layers,
                0,
                "{name} cannot collide with a {other_name}"
            );
        }
    }
}

/// A moon must not be moved by the impact, and the ship must rebound.
///
/// Without `StaticBody` the response treats the body as dynamic and divides
/// the impulse by 1.5e23 kg, which is no impulse at all.
#[test]
#[allow(clippy::unwrap_used, clippy::expect_used)]
fn test_ship_bounces_off_a_static_moon() {
    let mut app = build_collision_app();
    app.add_systems(
        FixedUpdate,
        (
            crate::collision_detection_system.in_set(PhysicsSet::AccumulateForces),
            crate::collision_response_system.in_set(CollisionResponseSet),
        ),
    );
    let ship = app
        .world_mut()
        .spawn((
            RigidBody::new(10_000.0, 1.0),
            Transform::from_translation(Vec3::ZERO),
            sphere_shape(5.0),
            CollisionLayersComponent::new(delta_v_types::collision::layers::SHIP),
            DynamicBody,
        ))
        .id();
    let moon = app
        .world_mut()
        .spawn((
            RigidBody::new(1.5e23, 1.0),
            Transform::from_translation(Vec3::Z * -8.0),
            sphere_shape(6.0),
            CollisionLayersComponent::new(layers::CELESTIAL),
            StaticBody,
        ))
        .id();

    app.world_mut()
        .get_mut::<RigidBody>(ship)
        .expect("ship has a body")
        .velocity = Vec3::Z * -60.0;

    run_fixed_update(&mut app);

    let ship_velocity = app.world().get::<RigidBody>(ship).expect("ship").velocity;
    let moon_velocity = app.world().get::<RigidBody>(moon).expect("moon").velocity;
    assert!(
        ship_velocity.z > 0.0,
        "the ship should rebound, its z velocity was {}",
        ship_velocity.z
    );
    assert_eq!(moon_velocity, Vec3::ZERO, "the moon must not be moved");
}

// ---------------------------------------------------------------------------
// Celestial bodies
// ---------------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_collision_mixed_pair_needs_only_one_relevant_side() {
    let mut app = build_detection_app();

    spawn_relevance_asteroid(&mut app, Vec3::ZERO, 20.0);
    app.world_mut().spawn((
        RigidBody::new(10.0, 1.0),
        Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
        sphere_shape(2.0),
        CollisionLayersComponent::new(delta_v_types::collision::layers::SHIP),
        DynamicBody,
    ));

    let events = run_detection_and_collect(&mut app);
    assert_eq!(
        events.len(),
        1,
        "a mixed pair must be tested when either side is relevant"
    );
}

// ---------------------------------------------------------------------------
// Collision detection: oriented boxes
// ---------------------------------------------------------------------------

/// A ship-shaped box: 9.0 m wide, 3.6 m tall, 15.8 m long — the collision half
/// extents from `assets/templates/ships/space-fighter-comrade1280/ship.json`.
const SHIP_HALF_EXTENTS: Vec3 = Vec3::new(4.502, 1.781, 7.908);

#[test]
fn test_pitched_ship_box_hits_a_body_straight_past_its_nose() {
    // The ship is pitched a quarter turn about X, so its long axis (local Z, the
    // nose) points straight down and its thin axis (local Y) is still world Y.
    let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    let nose = rotation * Vec3::Z;

    let ship = box_shape(SHIP_HALF_EXTENTS);
    let moon = sphere_shape(0.5);

    // 8.4 m out along the nose: inside the ship as drawn, and 8.4 - 7.908 = 0.49 m
    // short of the nose tip so the 0.5 m sphere overlaps it.
    let past_the_nose = crate::check_collision(
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

    // The same distance along the thin axis is nowhere near the hull. An unrotated
    // test would also miss this one, so it does not distinguish the two readings;
    // it is here to pin down that the long axis is what gained reach.
    let across_the_thin_axis = crate::check_collision(
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

#[test]
#[allow(clippy::unwrap_used)]
fn test_unrotated_reading_of_the_pitched_ship_misses_the_same_body() {
    // The body the test above collides with is far outside the world-axis-aligned
    // box the rotation-blind test used, which only reaches 1.781 m up. This is the
    // regression that let a ship fly through bodies its drawn collision shape
    // visibly overlapped.
    let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    let nose = rotation * Vec3::Z;

    let ship = box_shape(SHIP_HALF_EXTENTS);
    let moon = sphere_shape(0.5);

    let world_aligned = crate::check_collision(
        Vec3::ZERO,
        Quat::IDENTITY,
        &ship,
        nose * 8.4,
        Quat::IDENTITY,
        &moon,
    );
    assert!(
        world_aligned.is_none(),
        "ignoring the rotation must lose the collision — that is the bug being fixed"
    );
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_rotated_boxes_collide_where_axis_aligned_boxes_would_not() {
    // A unit box turned 45 degrees reaches sqrt(2) ≈ 1.414 along Z. A small box
    // whose near face sits at z = 1.1 is inside that reach but 0.1 m clear of the
    // unturned box, which stops at z = 1.0.
    let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
    let turned = box_shape(Vec3::splat(1.0));
    let small = box_shape(Vec3::splat(0.2));
    let other_pos = Vec3::new(0.0, 0.0, 1.3);

    let rotated = crate::check_collision(
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

    let unrotated = crate::check_collision(
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

#[test]
#[allow(clippy::expect_used, clippy::float_cmp)]
fn test_collision_normal_of_a_rotated_box_follows_the_rotated_face() {
    // The sphere sits just off the box's local +X face, so the minimum-translation
    // normal is that face's world direction, not the world X axis.
    let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4);
    let face = rotation * Vec3::X;

    let turned = box_shape(Vec3::splat(1.0));
    let moon = sphere_shape(0.5);

    let (normal, penetration) = crate::check_collision(
        Vec3::ZERO,
        rotation,
        &turned,
        face * 1.4,
        Quat::IDENTITY,
        &moon,
    )
    .expect("a sphere just off the rotated face collides");

    assert!(
        normal.dot(face) > 0.99,
        "normal should follow the rotated face {face}, got {normal}"
    );
    assert!(
        (penetration - 0.1).abs() < 0.01,
        "penetration should be ~0.1, got {penetration}"
    );
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_collision_shape_offset_is_rotated_with_its_body() {
    // The shape's offset is expressed in the body's local frame. Turned a quarter
    // turn about X it points down instead of forward, so a body placed at the
    // rotated offset is the one that collides.
    let mut app = build_detection_app();

    let rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    let local_offset = Vec3::Z * 3.0;
    let rotated_offset = rotation * local_offset;

    app.world_mut().spawn((
        RigidBody::new(10.0, 1.0),
        Transform::from_translation(Vec3::ZERO).with_rotation(rotation),
        CollisionShape(CollisionShapeData::box_shape(
            Vec3::splat(0.5),
            local_offset,
        )),
        CollisionLayersComponent::new(layers::SHIP),
    ));
    app.world_mut().spawn((
        RigidBody::new(10.0, 1.0),
        Transform::from_translation(rotated_offset),
        sphere_shape(0.2),
        CollisionLayersComponent::new(layers::ASTEROID),
    ));

    let events = run_detection_and_collect(&mut app);
    assert_eq!(
        events.len(),
        1,
        "the collision shape offset must follow the body's rotation"
    );
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_pitched_ship_is_stopped_by_a_body_past_its_nose() {
    // End to end through the detection and response systems: a pitched ship whose
    // nose is inside a static body is pushed back out and keeps its velocity along
    // the rotated axis.
    let mut app = build_collision_app();

    app.add_systems(
        FixedUpdate,
        (
            crate::collision_detection_system.in_set(PhysicsSet::AccumulateForces),
            crate::collision_response_system.in_set(CollisionResponseSet),
        ),
    );

    let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    let nose = rotation * Vec3::Z;

    let ship = app
        .world_mut()
        .spawn((
            RigidBody::new(1000.0, 1.0),
            Transform::from_translation(Vec3::ZERO).with_rotation(rotation),
            box_shape(SHIP_HALF_EXTENTS),
            CollisionLayersComponent::new(layers::SHIP),
            DynamicBody,
        ))
        .id();

    // 8.2 m out along the nose: 7.908 + 0.5 - 8.2 = 0.208 m of penetration, well
    // past the response system's 0.01 m slop.
    app.world_mut().spawn((
        RigidBody::new(1.0e12, 1.0),
        Transform::from_translation(nose * 8.2),
        sphere_shape(0.5),
        CollisionLayersComponent::new(layers::CELESTIAL),
        StaticBody,
    ));

    run_fixed_update(&mut app);

    let transform = app.world().get::<Transform>(ship).unwrap();
    let moved_along = transform.translation.dot(nose);
    assert!(
        moved_along < 0.0,
        "the ship should be pushed back off the body, but it sits {moved_along} m along its nose"
    );
}

/// System set for collision response, runs after collision detection.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct CollisionResponseSet;
