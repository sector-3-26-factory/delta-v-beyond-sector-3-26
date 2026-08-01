// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for ship spawning.
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

/// Tests that a valid player-controlled ship template loads and validates correctly.
#[test]
fn test_load_player_controlled_ship() {
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
            // Verify core ship properties
            assert!(player_ship.mass > 0.0, "mass should be positive");
            assert!(
                player_ship.inertia_scale > 0.0,
                "inertia_scale should be positive"
            );
            assert!(player_ship.health > 0.0, "health should be positive");
            assert!(
                player_ship.max_weapons_count > 0,
                "max_weapons_count should be positive"
            );
            assert!(
                player_ship.max_propulsions_count > 0,
                "max_propulsions_count should be positive"
            );

            // Verify propulsion
            assert!(
                !player_ship.propulsion.main_thruster_names.is_empty(),
                "should have at least one main thruster"
            );
            assert!(
                !player_ship.propulsion.maneuvering_thruster.is_empty(),
                "should have maneuvering thruster"
            );

            // Verify cameras
            assert!(
                player_ship.cameras.cockpit.available,
                "cockpit camera should be available"
            );
            assert!(
                player_ship.cameras.drone.available,
                "drone camera should be available"
            );

            // Verify bounding box
            assert!(player_ship.bounding_box.max.x > player_ship.bounding_box.min.x);
            assert!(player_ship.bounding_box.max.y > player_ship.bounding_box.min.y);
            assert!(player_ship.bounding_box.max.z > player_ship.bounding_box.min.z);

            // Verify collision shape
            match &player_ship.collision_shape.shape_type {
                delta_v_types::CollisionShapeType::Box { half_extents } => {
                    assert!(half_extents.x > 0.0);
                    assert!(half_extents.y > 0.0);
                    assert!(half_extents.z > 0.0);
                }
                _ => panic!("expected Box collision shape"),
            }

            // Verify cockpit
            assert!(
                !player_ship.cockpit.stations.is_empty(),
                "cockpit should have at least one station"
            );
        }
        _ => panic!("expected PlayerShip template variant"),
    }
}

/// Tests that the player-controlled ship has all required cameras.
#[test]
fn test_player_ship_cameras() {
    let result = load_player_controlled_ship("space-fighter-comrade1280");
    assert!(result.is_ok());

    let (_, template) = result.unwrap();
    match template {
        delta_v_types::EntityTemplate::PlayerShip(player_ship) => {
            let cameras = &player_ship.cameras;
            // All 8 cameras should be present
            assert!(cameras.cockpit.available);
            assert!(cameras.drone.available);
            assert!(cameras.rear.available);
            assert!(cameras.front.available);
            assert!(cameras.left.available);
            assert!(cameras.right.available);
            assert!(cameras.top.available);
            assert!(cameras.bottom.available);

            // Each camera should have position and target
            for cam in [
                &cameras.cockpit,
                &cameras.drone,
                &cameras.rear,
                &cameras.front,
                &cameras.left,
                &cameras.right,
                &cameras.top,
                &cameras.bottom,
            ] {
                // Position and target are Vec3Json, just verify they exist
                let _ = cam.position;
                let _ = cam.target;
            }
        }
        _ => panic!("expected PlayerShip template variant"),
    }
}

/// Tests that the player-controlled ship has valid propulsion configuration.
#[test]
fn test_player_ship_propulsion() {
    let result = load_player_controlled_ship("space-fighter-comrade1280");
    assert!(result.is_ok());

    let (_, template) = result.unwrap();
    match template {
        delta_v_types::EntityTemplate::PlayerShip(player_ship) => {
            let propulsion = &player_ship.propulsion;
            assert!(!propulsion.main_thruster_names.is_empty());
            assert!(!propulsion.maneuvering_thruster.is_empty());
        }
        _ => panic!("expected PlayerShip template variant"),
    }
}

/// Tests that the player-controlled ship has valid weapons configuration.
#[test]
fn test_player_ship_weapons() {
    let result = load_player_controlled_ship("space-fighter-comrade1280");
    assert!(result.is_ok());

    let (_, template) = result.unwrap();
    match template {
        delta_v_types::EntityTemplate::PlayerShip(player_ship) => {
            // Weapons may be empty, but the field should exist
            let _ = &player_ship.weapons;
            assert!(player_ship.max_weapons_count > 0);
        }
        _ => panic!("expected PlayerShip template variant"),
    }
}
