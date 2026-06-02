// See AGENTS.md

//! Tests for [`RigidBody`].

#![allow(clippy::float_cmp)]

use super::*;

#[test]
fn test_rigid_body_creation() {
    let body = RigidBody::new(10.0, 1.0);
    assert_eq!(body.mass, 10.0);
    assert_eq!(body.velocity, Vec3::ZERO);
    assert_eq!(body.angular_velocity, Vec3::ZERO);
    assert_eq!(body.force_accumulator, Vec3::ZERO);
    assert_eq!(body.torque_accumulator, Vec3::ZERO);
}

#[test]
fn test_inertia_tensor_diagonal() {
    let body = RigidBody::new(5.0, 1.0);
    // Sphere inertia: (2/5) * 5.0 = 2.0
    let expected_inertia = 2.0;
    assert!((body.inertia_tensor[0] - expected_inertia).abs() < 0.01);
    assert!((body.inertia_tensor[4] - expected_inertia).abs() < 0.01);
    assert!((body.inertia_tensor[8] - expected_inertia).abs() < 0.01);
}

#[test]
fn test_inertia_scale_applied() {
    let body = RigidBody::new(10.0, 2.0);
    // Base sphere inertia: (2/5) * 10.0 = 4.0
    // With scale 2.0: 4.0 * 2.0 = 8.0
    let expected_inertia = 8.0;
    assert!((body.inertia_tensor[0] - expected_inertia).abs() < 0.01);
}

#[test]
fn test_apply_force() {
    let mut body = RigidBody::new(10.0, 1.0);
    body.apply_force(Vec3::new(100.0, 0.0, 0.0));
    assert_eq!(body.force_accumulator, Vec3::new(100.0, 0.0, 0.0));
    body.apply_force(Vec3::new(50.0, 0.0, 0.0));
    assert_eq!(body.force_accumulator, Vec3::new(150.0, 0.0, 0.0));
}

#[test]
fn test_apply_torque() {
    let mut body = RigidBody::new(10.0, 1.0);
    body.apply_torque(Vec3::new(0.0, 10.0, 0.0));
    assert_eq!(body.torque_accumulator, Vec3::new(0.0, 10.0, 0.0));
}

#[test]
fn test_clear_accumulators() {
    let mut body = RigidBody::new(10.0, 1.0);
    body.apply_force(Vec3::new(100.0, 0.0, 0.0));
    body.apply_torque(Vec3::new(0.0, 10.0, 0.0));
    body.clear_accumulators();
    assert_eq!(body.force_accumulator, Vec3::ZERO);
    assert_eq!(body.torque_accumulator, Vec3::ZERO);
}

#[test]
fn test_integrate_velocity() {
    let mut body = RigidBody::new(10.0, 1.0);
    body.apply_force(Vec3::new(100.0, 0.0, 0.0));
    body.integrate_velocity(1.0 / 60.0); // 60 Hz timestep
                                         // a = F/m = 100/10 = 10 m/s²
                                         // v = a * dt = 10 * (1/60) ≈ 0.1667
    assert!((body.velocity.x - 0.1667).abs() < 0.001);
}

#[test]
fn test_integrate_angular_velocity() {
    let mut body = RigidBody::new(5.0, 1.0);
    // Sphere inertia: (2/5) * 5.0 = 2.0
    body.apply_torque(Vec3::new(0.0, 4.0, 0.0));
    body.integrate_angular_velocity(1.0 / 60.0);
    // alpha = tau / I = 4.0 / 2.0 = 2.0 rad/s²
    // omega = alpha * dt = 2.0 * (1/60) ≈ 0.0333
    assert!((body.angular_velocity.y - 0.0333).abs() < 0.001);
}

#[test]
fn test_new_with_unit_mass() {
    let body = RigidBody::new(1.0, 1.0);
    assert_eq!(body.mass, 1.0);
    assert_eq!(body.velocity, Vec3::ZERO);
}
