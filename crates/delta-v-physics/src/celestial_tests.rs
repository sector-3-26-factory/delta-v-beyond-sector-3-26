// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for celestial body components.

use super::*;

#[test]
fn sun_has_no_rotation_when_none() {
    let sun = Sun {
        rotation_period: None,
    };
    assert!(sun.rotation_period.is_none());
}

#[test]
fn planet_has_zero_axial_tilt() {
    let planet = Planet {
        rotation_period: None,
        axial_tilt: 0.0,
        animations_enabled: true,
    };
    assert!(
        (planet.axial_tilt - 0.0).abs() < f32::EPSILON,
        "expected zero axial tilt, got {}",
        planet.axial_tilt
    );
}

#[test]
fn planet_has_animations_enabled() {
    let planet = Planet {
        rotation_period: None,
        axial_tilt: 0.0,
        animations_enabled: true,
    };
    assert!(planet.animations_enabled);
}
