// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Panic tests for [`RigidBody`] invalid mass values.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::rigid_body::RigidBody;

#[test]
#[should_panic(expected = "RigidBodyData mass must be positive and finite")]
fn test_rigid_body_zero_mass_panics() {
    RigidBody::new(0.0, 1.0);
}

#[test]
#[should_panic(expected = "RigidBodyData mass must be positive and finite")]
fn test_rigid_body_negative_mass_panics() {
    RigidBody::new(-10.0, 1.0);
}

#[test]
#[should_panic(expected = "RigidBodyData mass must be positive and finite")]
fn test_rigid_body_nan_mass_panics() {
    RigidBody::new(f32::NAN, 1.0);
}

#[test]
#[should_panic(expected = "RigidBodyData mass must be positive and finite")]
fn test_rigid_body_infinite_mass_panics() {
    RigidBody::new(f32::INFINITY, 1.0);
}

#[test]
#[should_panic(expected = "RigidBodyData mass must be positive and finite")]
fn test_rigid_body_neg_infinity_mass_panics() {
    RigidBody::new(f32::NEG_INFINITY, 1.0);
}
