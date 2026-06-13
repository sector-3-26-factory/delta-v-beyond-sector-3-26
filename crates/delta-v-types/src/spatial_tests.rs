// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for spatial types: [`Vec3Json`], [`QuatJson`], [`BoundingBoxJson`].

#![allow(clippy::float_cmp)]

use bevy::prelude::{Quat, Vec3};

use crate::spatial::{BoundingBoxJson, QuatJson, Vec3Json};

// ---------------------------------------------------------------------------
// Vec3Json
// ---------------------------------------------------------------------------

#[test]
fn test_vec3_json_to_vec3() {
    let v = Vec3Json {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let bevy_vec: Vec3 = v.into();
    assert_eq!(bevy_vec, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_vec3_json_negative() {
    let v = Vec3Json {
        x: -1.5,
        y: 0.0,
        z: 4.2,
    };
    let bevy_vec: Vec3 = v.into();
    assert_eq!(bevy_vec, Vec3::new(-1.5, 0.0, 4.2));
}

#[test]
fn test_vec3_json_zero() {
    let v = Vec3Json {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let bevy_vec: Vec3 = v.into();
    assert_eq!(bevy_vec, Vec3::ZERO);
}

// ---------------------------------------------------------------------------
// QuatJson
// ---------------------------------------------------------------------------

#[test]
fn test_quat_json_to_quat_identity() {
    let q = QuatJson {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };
    let bevy_quat: Quat = q.into();
    assert_eq!(bevy_quat, Quat::IDENTITY);
}

#[test]
fn test_quat_json_to_quat_90_deg_y() {
    // 90 degree rotation around Y axis
    let q = QuatJson {
        x: 0.0,
        y: std::f32::consts::FRAC_1_SQRT_2,
        z: 0.0,
        w: std::f32::consts::FRAC_1_SQRT_2,
    };
    let bevy_quat: Quat = q.into();
    let expected = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    assert!(
        (bevy_quat.x - expected.x).abs() < 0.001
            && (bevy_quat.y - expected.y).abs() < 0.001
            && (bevy_quat.z - expected.z).abs() < 0.001
            && (bevy_quat.w - expected.w).abs() < 0.001,
        "quaternion mismatch: got {bevy_quat:?}, expected {expected:?}"
    );
}

// ---------------------------------------------------------------------------
// BoundingBoxJson
// ---------------------------------------------------------------------------

#[test]
fn test_bounding_box_size_cube() {
    let bbox = BoundingBoxJson {
        min: Vec3Json {
            x: -1.0,
            y: -1.0,
            z: -1.0,
        },
        max: Vec3Json {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    };
    let size = bbox.size();
    assert_eq!(size, Vec3::new(2.0, 2.0, 2.0));
}

#[test]
fn test_bounding_box_size_rectangular() {
    let bbox = BoundingBoxJson {
        min: Vec3Json {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        max: Vec3Json {
            x: 10.0,
            y: 5.0,
            z: 2.0,
        },
    };
    let size = bbox.size();
    assert_eq!(size, Vec3::new(10.0, 5.0, 2.0));
}

#[test]
fn test_bounding_box_size_asymmetric() {
    let bbox = BoundingBoxJson {
        min: Vec3Json {
            x: -3.0,
            y: -2.0,
            z: -1.0,
        },
        max: Vec3Json {
            x: 7.0,
            y: 4.0,
            z: 5.0,
        },
    };
    let size = bbox.size();
    assert_eq!(size, Vec3::new(10.0, 6.0, 6.0));
}

#[test]
fn test_bounding_box_center_cube() {
    let bbox = BoundingBoxJson {
        min: Vec3Json {
            x: -1.0,
            y: -1.0,
            z: -1.0,
        },
        max: Vec3Json {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    };
    let center = bbox.center();
    assert_eq!(center, Vec3::ZERO);
}

#[test]
fn test_bounding_box_center_offset() {
    let bbox = BoundingBoxJson {
        min: Vec3Json {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        max: Vec3Json {
            x: 10.0,
            y: 10.0,
            z: 10.0,
        },
    };
    let center = bbox.center();
    assert_eq!(center, Vec3::new(5.0, 5.0, 5.0));
}

#[test]
fn test_bounding_box_center_asymmetric() {
    let bbox = BoundingBoxJson {
        min: Vec3Json {
            x: -5.0,
            y: 0.0,
            z: -3.0,
        },
        max: Vec3Json {
            x: 5.0,
            y: 4.0,
            z: 7.0,
        },
    };
    let center = bbox.center();
    assert_eq!(center, Vec3::new(0.0, 2.0, 2.0));
}
