// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for debug axes behavior.
//!
//! Tests verify that:
//! 1. `DebugAxesEligible` marker can be created and cloned
//! 2. `DebugAxes` component preserves entity_id and axis_length
//! 3. `DebugConfig::should_show_axes_for()` correctly filters based on configuration
//!
//! Per ADR-0021 (testing strategy), unit tests are in sibling `_tests.rs` files.
//! System behavior (mark_debug_axes, spawn_debug_axes) is tested indirectly via
//! DebugConfig filtering logic, which is the decision point for visibility.

#![cfg(test)]

use crate::debug_axes::{DebugAxes, DebugAxesEligible};
use crate::debug_config::DebugConfig;

#[test]
fn test_debug_axes_eligible_creation() {
    let eligible = DebugAxesEligible::new("ship_1".to_string(), 2.5);
    assert_eq!(eligible.entity_id, "ship_1");
    assert_eq!(eligible.axis_length, 2.5);
}

#[test]
fn test_debug_axes_creation() {
    let axes = DebugAxes::new("station_main".to_string(), 3.0);
    assert_eq!(axes.entity_id, "station_main");
    assert_eq!(axes.axis_length, 3.0);
}

#[test]
fn test_debug_axes_eligible_clone() {
    let eligible = DebugAxesEligible::new("entity_1".to_string(), 1.5);
    let cloned = eligible.clone();
    assert_eq!(cloned.entity_id, eligible.entity_id);
    assert_eq!(cloned.axis_length, eligible.axis_length);
}

#[test]
fn test_debug_axes_clone() {
    let axes = DebugAxes::new("entity_1".to_string(), 1.5);
    let cloned = axes.clone();
    assert_eq!(cloned.entity_id, axes.entity_id);
    assert_eq!(cloned.axis_length, axes.axis_length);
}

#[test]
fn test_mark_debug_axes_decision_show_false_blocks_all() {
    // When show_axis_indicators is false, should_show_axes_for() always returns false.
    // This is the condition checked by mark_debug_axes() before adding DebugAxes.
    let config = DebugConfig {
        show_axis_indicators: false,
        axis_indicator_entities: vec!["ship_1".to_string()],
    };

    assert!(!config.should_show_axes_for("ship_1"));
    assert!(!config.should_show_axes_for("ship_2"));
}

#[test]
fn test_mark_debug_axes_decision_show_true_empty_list() {
    // When show_axis_indicators is true and axis_indicator_entities is empty,
    // should_show_axes_for() returns true for all entities.
    // This is the condition checked by mark_debug_axes() for the "debug all" case.
    let config = DebugConfig {
        show_axis_indicators: true,
        axis_indicator_entities: vec![],
    };

    assert!(config.should_show_axes_for("ship_1"));
    assert!(config.should_show_axes_for("any_entity"));
}

#[test]
fn test_mark_debug_axes_decision_show_true_selective() {
    // When show_axis_indicators is true and axis_indicator_entities has entries,
    // should_show_axes_for() returns true only for entities in the list.
    // This is the condition checked by mark_debug_axes() for selective debugging.
    let config = DebugConfig {
        show_axis_indicators: true,
        axis_indicator_entities: vec!["ship_1".to_string(), "station_main".to_string()],
    };

    assert!(config.should_show_axes_for("ship_1"));
    assert!(config.should_show_axes_for("station_main"));
    assert!(!config.should_show_axes_for("ship_2"));
    assert!(!config.should_show_axes_for("asteroid_1"));
}

#[test]
fn test_spawn_debug_axes_uses_config_decision() {
    // spawn_debug_axes() calls config.should_show_axes_for() to decide
    // whether to render axes. This test verifies the decision logic.
    let config_disabled = DebugConfig {
        show_axis_indicators: false,
        axis_indicator_entities: vec![],
    };

    let config_enabled_all = DebugConfig {
        show_axis_indicators: true,
        axis_indicator_entities: vec![],
    };

    let config_enabled_selective = DebugConfig {
        show_axis_indicators: true,
        axis_indicator_entities: vec!["ship_1".to_string()],
    };

    // Test entity ID: "ship_1"
    let entity_id = "ship_1";

    // Case 1: Master switch off
    assert!(
        !config_disabled.should_show_axes_for(entity_id),
        "spawn_debug_axes should not render when show_axis_indicators is false"
    );

    // Case 2: Master switch on, empty list (render all)
    assert!(
        config_enabled_all.should_show_axes_for(entity_id),
        "spawn_debug_axes should render when show_axis_indicators is true and list is empty"
    );

    // Case 3: Master switch on, selective list (entity in list)
    assert!(
        config_enabled_selective.should_show_axes_for(entity_id),
        "spawn_debug_axes should render when entity ID is in axis_indicator_entities"
    );

    // Case 4: Master switch on, selective list (entity NOT in list)
    assert!(
        !config_enabled_selective.should_show_axes_for("ship_2"),
        "spawn_debug_axes should not render when entity ID is not in axis_indicator_entities"
    );
}
