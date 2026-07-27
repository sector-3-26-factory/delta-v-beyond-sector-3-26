// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for physical quantity conversion methods.

use super::PhysicalQuantityJson;

#[test]
fn test_to_meters_from_meters() {
    let q = PhysicalQuantityJson {
        value: 10.0,
        unit: "m".to_string(),
    };
    assert!((q.to_meters() - 10.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_meters_from_kilometers() {
    let q = PhysicalQuantityJson {
        value: 1.0,
        unit: "km".to_string(),
    };
    assert!((q.to_meters() - 1000.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_meters_from_au() {
    let q = PhysicalQuantityJson {
        value: 1.0,
        unit: "AU".to_string(),
    };
    // 1 AU = 1.4959786e11 m (IAU defined value)
    assert!((q.to_meters() - 1.495_978_6e11).abs() < 1e6);
}

#[test]
fn test_to_kilograms_from_kilograms() {
    let q = PhysicalQuantityJson {
        value: 1000.0,
        unit: "kg".to_string(),
    };
    assert!((q.to_kilograms() - 1000.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_kilograms_from_t() {
    let q = PhysicalQuantityJson {
        value: 1.0,
        unit: "t".to_string(),
    };
    assert!((q.to_kilograms() - 1000.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_seconds_from_seconds() {
    let q = PhysicalQuantityJson {
        value: 60.0,
        unit: "s".to_string(),
    };
    assert!((q.to_seconds() - 60.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_seconds_from_minutes() {
    let q = PhysicalQuantityJson {
        value: 1.0,
        unit: "min".to_string(),
    };
    assert!((q.to_seconds() - 60.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_seconds_from_hours() {
    let q = PhysicalQuantityJson {
        value: 1.0,
        unit: "h".to_string(),
    };
    assert!((q.to_seconds() - 3600.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_radians_from_radians() {
    let q = PhysicalQuantityJson {
        value: std::f32::consts::PI,
        unit: "rad".to_string(),
    };
    assert!((q.to_radians() - std::f32::consts::PI).abs() < f32::EPSILON);
}

#[test]
fn test_to_radians_from_degrees() {
    let q = PhysicalQuantityJson {
        value: 180.0,
        unit: "deg".to_string(),
    };
    let expected = std::f32::consts::PI;
    assert!((q.to_radians() - expected).abs() < 0.001);
}

#[test]
fn test_to_newtons_from_newtons() {
    let q = PhysicalQuantityJson {
        value: 1000.0,
        unit: "N".to_string(),
    };
    assert!((q.to_newtons() - 1000.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_newtons_from_kilonewtons() {
    let q = PhysicalQuantityJson {
        value: 1.0,
        unit: "kN".to_string(),
    };
    assert!((q.to_newtons() - 1000.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_newton_meters_from_newton_meters() {
    let q = PhysicalQuantityJson {
        value: 100.0,
        unit: "N⋅m".to_string(),
    };
    assert!((q.to_newton_meters() - 100.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_meters_per_second_from_meters_per_second() {
    let q = PhysicalQuantityJson {
        value: 100.0,
        unit: "m/s".to_string(),
    };
    assert!((q.to_meters_per_second() - 100.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_meters_per_second_from_kilometers_per_second() {
    let q = PhysicalQuantityJson {
        value: 1.0,
        unit: "km/s".to_string(),
    };
    assert!((q.to_meters_per_second() - 1000.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_meters_per_second_squared_from_meters_per_second_squared() {
    let q = PhysicalQuantityJson {
        value: 9.81,
        unit: "m/s²".to_string(),
    };
    assert!((q.to_meters_per_second_squared() - 9.81).abs() < f32::EPSILON);
}

#[test]
fn test_to_hertz_from_hertz() {
    let q = PhysicalQuantityJson {
        value: 10.0,
        unit: "Hz".to_string(),
    };
    assert!((q.to_hertz() - 10.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_hit_points_from_hit_points() {
    let q = PhysicalQuantityJson {
        value: 100.0,
        unit: "hp".to_string(),
    };
    assert!((q.to_hit_points() - 100.0).abs() < f32::EPSILON);
}

#[test]
fn test_to_dimensionless_from_dimensionless() {
    let q = PhysicalQuantityJson {
        value: 42.0,
        unit: "dimensionless".to_string(),
    };
    assert!((q.to_dimensionless() - 42.0).abs() < f32::EPSILON);
}
