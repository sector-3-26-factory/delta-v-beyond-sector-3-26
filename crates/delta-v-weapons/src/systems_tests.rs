// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for weapon systems.

use delta_v_core::Health;

#[test]
fn health_new_starts_at_full() {
    let health = Health::new(100.0);
    assert!((health.current - 100.0).abs() < f32::EPSILON);
    assert!((health.max - 100.0).abs() < f32::EPSILON);
    assert!(!health.is_destroyed());
}

#[test]
fn health_apply_damage_reduces_current() {
    let mut health = Health::new(100.0);
    assert!(!health.apply_damage(30.0));
    assert!((health.current - 70.0).abs() < f32::EPSILON);
    assert!(!health.is_destroyed());
}

#[test]
fn health_apply_damage_clamps_to_zero() {
    let mut health = Health::new(100.0);
    assert!(health.apply_damage(150.0));
    assert!((health.current - 0.0).abs() < f32::EPSILON);
    assert!(health.is_destroyed());
}

#[test]
fn health_exact_damage_destroys() {
    let mut health = Health::new(100.0);
    assert!(health.apply_damage(100.0));
    assert!((health.current - 0.0).abs() < f32::EPSILON);
    assert!(health.is_destroyed());
}

#[test]
fn health_damage_already_destroyed() {
    let mut health = Health::new(100.0);
    health.apply_damage(100.0);
    assert!(health.apply_damage(50.0));
    assert!((health.current - 0.0).abs() < f32::EPSILON);
    assert!(health.is_destroyed());
}
