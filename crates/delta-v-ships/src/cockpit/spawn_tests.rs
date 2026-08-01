// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for cockpit overlay types and schema validation.
//!
//! See ADR-0021 (Testing strategy).

// Test code is allowed to use expect/unwrap/indexing per ADR-0023.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use delta_v_assets::template::load_player_controlled_ship;

/// Tests that a valid player-controlled ship template loads and validates correctly,
/// including the cockpit definition.
#[test]
fn test_load_player_controlled_ship_with_cockpit() {
    let result = load_player_controlled_ship("space-fighter-comrade1280");
    assert!(
        result.is_ok(),
        "valid player-controlled ship should load: {:?}",
        result.err()
    );

    let (template_path, template) = result.unwrap();
    assert!(template_path.contains("space-fighter-comrade1280"));
    assert!(template_path.contains("player_controlled_ship.json"));

    // Verify the template is a PlayerShip variant
    match template {
        delta_v_types::EntityTemplate::PlayerShip(player_ship) => {
            // Verify cockpit is present and has stations
            assert!(
                !player_ship.cockpit.stations.is_empty(),
                "cockpit should have at least one station"
            );
            assert_eq!(player_ship.cockpit.stations[0].id, "default");
            assert_eq!(
                player_ship.cockpit.stations[0].texture,
                "cockpit/default.png"
            );
        }
        _ => panic!("expected PlayerShip template variant"),
    }
}

/// Tests that the cockpit definition in the player-controlled ship has the expected structure.
#[test]
fn test_cockpit_structure_in_ship_template() {
    let result = load_player_controlled_ship("space-fighter-comrade1280");
    assert!(result.is_ok());

    let (_, template) = result.unwrap();
    match template {
        delta_v_types::EntityTemplate::PlayerShip(player_ship) => {
            let cockpit = &player_ship.cockpit;
            // At least one station
            assert!(!cockpit.stations.is_empty());

            // First station should have required fields
            let station = &cockpit.stations[0];
            assert!(!station.id.is_empty());
            assert!(!station.texture.is_empty());
            // Slots may be empty (default from schema)
            // but if present, they should have shape and default_gauge
            for slot in &station.slots {
                assert!(!slot.default_gauge.is_empty());
                match &slot.shape {
                    delta_v_types::GaugeShape::Rectangle { x1, y1, x2, y2 } => {
                        assert!(*x2 > *x1);
                        assert!(*y2 > *y1);
                    }
                    delta_v_types::GaugeShape::Circle { r, .. } => {
                        assert!(*r > 0.0);
                    }
                }
            }
        }
        _ => panic!("expected PlayerShip template variant"),
    }
}
