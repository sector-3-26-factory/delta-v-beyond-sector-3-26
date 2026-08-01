// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for celestial body components.

use super::*;
use bevy::prelude::Entity;

#[test]
fn sun_has_no_rotation_when_none() {
    let sun = Sun {
        rotation_period: None,
    };
    assert!(sun.rotation_period.is_none());
}

#[test]
fn planet_has_zero_inclination() {
    let planet = Planet {
        orbital_parent: Entity::PLACEHOLDER,
        orbital_distance: 0.0,
        orbital_period: 0.0,
        orbital_eccentricity: 0.0,
        orbital_inclination: 0.0,
        initial_orbital_angle: 0.0,
        rotation_period: None,
        axial_tilt: 0.0,
    };
    assert!(
        (planet.orbital_inclination - 0.0).abs() < f32::EPSILON,
        "expected zero inclination, got {}",
        planet.orbital_inclination
    );
}

#[test]
fn planet_has_circular_orbit() {
    let planet = Planet {
        orbital_parent: Entity::PLACEHOLDER,
        orbital_distance: 0.0,
        orbital_period: 0.0,
        orbital_eccentricity: 0.0,
        orbital_inclination: 0.0,
        initial_orbital_angle: 0.0,
        rotation_period: None,
        axial_tilt: 0.0,
    };
    assert!(
        (planet.orbital_eccentricity - 0.0).abs() < f32::EPSILON,
        "expected zero eccentricity, got {}",
        planet.orbital_eccentricity
    );
}
