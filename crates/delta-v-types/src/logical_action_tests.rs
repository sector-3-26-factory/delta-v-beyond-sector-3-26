// AGENTS: before modifying this file, read AGENTS.md at the repository root.

use crate::logical_action::LogicalAction;

#[test]
fn test_logical_action_str_keys_are_unique() {
    let all = LogicalAction::all();
    let mut seen = std::collections::BTreeSet::new();
    for action in all {
        let key = action.as_str();
        assert!(
            seen.insert(key),
            "duplicate as_str() key: '{key}' (check LogicalAction::as_str)"
        );
    }
}

#[test]
fn test_logical_action_all_count() {
    assert_eq!(
        LogicalAction::all().len(),
        22,
        "expected 22 LogicalAction variants"
    );
}

#[test]
fn test_logical_action_str_keys_match_expected() {
    let expected_keys = [
        "thrust_forward",
        "thrust_backward",
        "pitch_up",
        "pitch_down",
        "yaw_left",
        "yaw_right",
        "roll_left",
        "roll_right",
        "strafe_left",
        "strafe_right",
        "strafe_up",
        "strafe_down",
        "toggle_flight_assist",
        "fire_primary",
        "cockpit_cycle_next",
        "cockpit_cycle_prev",
        "camera_switch_next",
        "camera_switch_prev",
        "toggle_targeting_menu",
        "cycle_target_next",
        "cycle_target_prev",
        "toggle_targeting_mode",
    ];
    for key in &expected_keys {
        let found = LogicalAction::all().iter().any(|a| a.as_str() == *key);
        assert!(found, "missing LogicalAction for key '{key}'");
    }
}
