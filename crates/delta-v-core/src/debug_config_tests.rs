// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for debug configuration.
//!
//! Tests verify that `DebugConfig::should_show_axes_for()` correctly respects
//! the `show_axis_indicators` master switch and `axis_indicator_entities` filtering.
//! Per ADR-0021 (testing strategy), unit tests live in sibling `_tests.rs` files.

use crate::debug_config::DebugConfig;

#[test]
fn test_show_axis_indicators_false_blocks_all() {
    // When show_axis_indicators is false, no entity should show axes,
    // regardless of axis_indicator_entities. Per ADR-0022, debug axes are
    // controlled by a master switch.
    let config = DebugConfig {
        show_axis_indicators: false,
        axis_indicator_entities: vec!["ship_1".to_string()],
        show_collision_shapes: false,
    };

    assert!(!config.should_show_axes_for("ship_1"));
    assert!(!config.should_show_axes_for("ship_2"));
    assert!(!config.should_show_axes_for("any_entity"));
}

#[test]
fn test_show_axis_indicators_true_empty_list_shows_all() {
    // When show_axis_indicators is true and axis_indicator_entities is empty,
    // all entities should show axes. This is the "debug everything" mode.
    let config = DebugConfig {
        show_axis_indicators: true,
        axis_indicator_entities: vec![],
        show_collision_shapes: false,
    };

    assert!(config.should_show_axes_for("ship_1"));
    assert!(config.should_show_axes_for("ship_2"));
    assert!(config.should_show_axes_for("any_entity"));
}

#[test]
fn test_show_axis_indicators_true_selective_list() {
    // When show_axis_indicators is true and axis_indicator_entities has entries,
    // only entities with IDs in the list should show axes. This is selective
    // debugging per ADR-0022.
    let config = DebugConfig {
        show_axis_indicators: true,
        axis_indicator_entities: vec!["ship_1".to_string(), "station_main".to_string()],
        show_collision_shapes: false,
    };

    // Entities in the list should show axes
    assert!(config.should_show_axes_for("ship_1"));
    assert!(config.should_show_axes_for("station_main"));

    // Entities NOT in the list should not show axes
    assert!(!config.should_show_axes_for("ship_2"));
    assert!(!config.should_show_axes_for("asteroid_1"));
    assert!(!config.should_show_axes_for("any_other_entity"));
}

#[test]
fn test_axis_indicator_entity_matching_is_exact() {
    // Entity ID matching must be exact. "ship_1" should not match "ship_10" or "my_ship_1".
    let config = DebugConfig {
        show_axis_indicators: true,
        axis_indicator_entities: vec!["ship_1".to_string()],
        show_collision_shapes: false,
    };

    assert!(config.should_show_axes_for("ship_1"));
    assert!(!config.should_show_axes_for("ship_10"));
    assert!(!config.should_show_axes_for("ship_1_variant"));
    assert!(!config.should_show_axes_for("my_ship_1"));
}

#[test]
fn test_axis_indicator_entities_case_sensitive() {
    // Entity ID matching is case-sensitive. "Ship_1" ≠ "ship_1".
    let config = DebugConfig {
        show_axis_indicators: true,
        axis_indicator_entities: vec!["ship_1".to_string()],
        show_collision_shapes: false,
    };

    assert!(config.should_show_axes_for("ship_1"));
    assert!(!config.should_show_axes_for("Ship_1"));
    assert!(!config.should_show_axes_for("SHIP_1"));
}
