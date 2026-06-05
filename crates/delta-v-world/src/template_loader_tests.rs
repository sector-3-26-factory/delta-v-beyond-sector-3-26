// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for the template loader.
//!
//! See ADR-0021 (Testing strategy).

// Test code is allowed to use expect/unwrap/indexing per ADR-0023.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use crate::template_loader::load_template;

#[test]
fn test_load_ship_template() {
    // load_template expects a path relative to the assets directory.
    let result = load_template(
        "templates/ships/space-fighter-comrade1280/ship.json",
        "ship",
    );
    assert!(
        result.is_ok(),
        "failed to load fighter template: {result:?}"
    );
    let template = result.unwrap();
    assert_eq!(template["entity_type"], "ship");
    assert!(template.get("mass").is_some());
    assert!(template.get("propulsion").is_some());
    // Ship template should NOT have cameras.
    assert!(template.get("cameras").is_none());
}

#[test]
fn test_load_player_controlled_ship_template() {
    // load_template for player_controlled_ship loads the player_controlled_ship.json
    // which only has cameras. The merging with ship.json happens in lib.rs.
    let result = load_template(
        "templates/ships/debug-ship-sphere/player_controlled_ship.json",
        "player_controlled_ship",
    );
    assert!(
        result.is_ok(),
        "failed to load player_controlled_ship template: {result:?}"
    );
    let template = result.unwrap();
    // player_controlled_ship.json has entity_type player_controlled_ship.
    assert_eq!(template["entity_type"], "player_controlled_ship");
    // player_controlled_ship.json only has cameras, not mass/propulsion.
    // The merging with ship.json happens in the world loader.
    assert!(template.get("cameras").is_some());
}

#[test]
fn test_debug_ship_templates() {
    for (path, name) in [
        ("templates/ships/debug-ship-cube/ship.json", "cube"),
        ("templates/ships/debug-ship-sphere/ship.json", "sphere"),
        ("templates/ships/debug-ship-capsule/ship.json", "capsule"),
    ] {
        // load_template expects a path relative to the assets directory.
        let result = load_template(path, "ship");
        assert!(result.is_ok(), "failed to load {name} template: {result:?}");
        let template = result.unwrap();
        assert_eq!(template["entity_type"], "ship");
        // Debug ship templates should NOT have cameras.
        assert!(
            template.get("cameras").is_none(),
            "{name} template should not have cameras"
        );
    }
}
