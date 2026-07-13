// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Integration tests for i18n key completeness.
//!
//! See ADR-0021 (Testing strategy).

// Test code is allowed to use expect/unwrap/indexing per ADR-0023.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use std::collections::HashSet;
use std::path::PathBuf;

use serde_json::Value;

/// Returns the absolute path to the workspace root directory.
///
/// Derived from `CARGO_MANIFEST_DIR` (which points at the crate directory).
fn get_workspace_root() -> PathBuf {
    let manifest_dir = std::env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("CARGO_MANIFEST_DIR parent must exist")
        .parent()
        .expect("workspace root must exist")
        .to_path_buf()
}

/// Expected action keys in the i18n file.
const EXPECTED_ACTION_KEYS: &[&str] = &[
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
    "toggle_navigation_menu",
    "toggle_keybindings_menu",
    "cycle_target_next",
    "cycle_target_prev",
    "toggle_targeting_mode",
];

/// Expected key names in the i18n file.
const EXPECTED_KEY_NAMES: &[&str] = &[
    "KeyW",
    "KeyS",
    "KeyA",
    "KeyD",
    "KeyQ",
    "KeyE",
    "KeyR",
    "KeyF",
    "KeyC",
    "KeyN",
    "KeyT",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "Space",
    "F1",
    "F2",
    "F3",
    "ControlLeft",
    "ShiftLeft",
    "AltLeft",
];

/// Expected group keys in the i18n file.
const EXPECTED_GROUP_KEYS: &[&str] = &["flight", "combat", "systems"];

/// Expected camera names in the i18n file.
const EXPECTED_CAMERA_NAMES: &[&str] = &[
    "cockpit", "front", "rear", "left", "right", "top", "bottom", "drone",
];

/// Expected entity types in the i18n file.
const EXPECTED_ENTITY_TYPES: &[&str] = &["ship", "asteroid", "station"];

/// Loads and parses a JSON file from the assets directory.
fn load_json_file(path: &str) -> Value {
    let full_path = get_workspace_root().join(path);
    let content = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|_| panic!("failed to read i18n file: {full_path:?}"));
    serde_json::from_str(&content).expect("failed to parse i18n JSON")
}

/// Tests that all expected action keys are present in en.json.
#[test]
fn test_en_json_has_all_action_keys() {
    let en: Value = load_json_file("assets/i18n/en.json");
    let action_keys = get_action_keys(&en);

    for &key in EXPECTED_ACTION_KEYS {
        assert!(
            action_keys.contains(key),
            "en.json missing action key: {key}"
        );
    }
}

/// Tests that all expected key names are present in en.json.
#[test]
fn test_en_json_has_all_key_names() {
    let en: Value = load_json_file("assets/i18n/en.json");
    let key_names = get_key_names(&en);

    for &key in EXPECTED_KEY_NAMES {
        assert!(key_names.contains(key), "en.json missing key name: {key}");
    }
}

/// Tests that all expected group keys are present in en.json.
#[test]
fn test_en_json_has_all_group_keys() {
    let en: Value = load_json_file("assets/i18n/en.json");
    let group_keys = get_group_keys(&en);

    for &key in EXPECTED_GROUP_KEYS {
        assert!(group_keys.contains(key), "en.json missing group key: {key}");
    }
}

/// Tests that all expected camera names are present in en.json.
#[test]
fn test_en_json_has_all_camera_names() {
    let en: Value = load_json_file("assets/i18n/en.json");
    let camera_names = get_camera_names(&en);

    for &key in EXPECTED_CAMERA_NAMES {
        assert!(
            camera_names.contains(key),
            "en.json missing camera name: {key}"
        );
    }
}

/// Tests that all expected entity types are present in en.json.
#[test]
fn test_en_json_has_all_entity_types() {
    let en: Value = load_json_file("assets/i18n/en.json");
    let entity_types = get_entity_types(&en);

    for &key in EXPECTED_ENTITY_TYPES {
        assert!(
            entity_types.contains(key),
            "en.json missing entity type: {key}"
        );
    }
}

/// Tests that de.json has all the same action keys as en.json.
#[test]
fn test_de_json_has_all_action_keys() {
    let de: Value = load_json_file("assets/i18n/de.json");
    let action_keys = get_action_keys(&de);

    for &key in EXPECTED_ACTION_KEYS {
        assert!(
            action_keys.contains(key),
            "de.json missing action key: {key}"
        );
    }
}

/// Tests that de.json has all the same key names as en.json.
#[test]
fn test_de_json_has_all_key_names() {
    let de: Value = load_json_file("assets/i18n/de.json");
    let key_names = get_key_names(&de);

    for &key in EXPECTED_KEY_NAMES {
        assert!(key_names.contains(key), "de.json missing key name: {key}");
    }
}

/// Tests that de.json has all the same group keys as en.json.
#[test]
fn test_de_json_has_all_group_keys() {
    let de: Value = load_json_file("assets/i18n/de.json");
    let group_keys = get_group_keys(&de);

    for &key in EXPECTED_GROUP_KEYS {
        assert!(group_keys.contains(key), "de.json missing group key: {key}");
    }
}

/// Tests that de.json has all the same camera names as en.json.
#[test]
fn test_de_json_has_all_camera_names() {
    let de: Value = load_json_file("assets/i18n/de.json");
    let camera_names = get_camera_names(&de);

    for &key in EXPECTED_CAMERA_NAMES {
        assert!(
            camera_names.contains(key),
            "de.json missing camera name: {key}"
        );
    }
}

/// Tests that de.json has all the same entity types as en.json.
#[test]
fn test_de_json_has_all_entity_types() {
    let de: Value = load_json_file("assets/i18n/de.json");
    let entity_types = get_entity_types(&de);

    for &key in EXPECTED_ENTITY_TYPES {
        assert!(
            entity_types.contains(key),
            "de.json missing entity type: {key}"
        );
    }
}

/// Tests that en.json and de.json have the same set of action keys.
#[test]
fn test_en_and_de_have_same_action_keys() {
    let en: Value = load_json_file("assets/i18n/en.json");
    let de: Value = load_json_file("assets/i18n/de.json");

    let en_keys = get_action_keys(&en);
    let de_keys = get_action_keys(&de);

    let en_only: HashSet<_> = en_keys.difference(&de_keys).collect();
    let de_only: HashSet<_> = de_keys.difference(&en_keys).collect();

    assert!(
        en_only.is_empty(),
        "en.json has action keys not in de.json: {en_only:?}"
    );
    assert!(
        de_only.is_empty(),
        "de.json has action keys not in en.json: {de_only:?}"
    );
}

/// Tests that en.json and de.json have the same set of key names.
#[test]
fn test_en_and_de_have_same_key_names() {
    let en: Value = load_json_file("assets/i18n/en.json");
    let de: Value = load_json_file("assets/i18n/de.json");

    let en_keys = get_key_names(&en);
    let de_keys = get_key_names(&de);

    let en_only: HashSet<_> = en_keys.difference(&de_keys).collect();
    let de_only: HashSet<_> = de_keys.difference(&en_keys).collect();

    assert!(
        en_only.is_empty(),
        "en.json has key names not in de.json: {en_only:?}"
    );
    assert!(
        de_only.is_empty(),
        "de.json has key names not in en.json: {de_only:?}"
    );
}

/// Tests that en.json and de.json have the same set of group keys.
#[test]
fn test_en_and_de_have_same_group_keys() {
    let en: Value = load_json_file("assets/i18n/en.json");
    let de: Value = load_json_file("assets/i18n/de.json");

    let en_keys = get_group_keys(&en);
    let de_keys = get_group_keys(&de);

    let en_only: HashSet<_> = en_keys.difference(&de_keys).collect();
    let de_only: HashSet<_> = de_keys.difference(&en_keys).collect();

    assert!(
        en_only.is_empty(),
        "en.json has group keys not in de.json: {en_only:?}"
    );
    assert!(
        de_only.is_empty(),
        "de.json has group keys not in en.json: {de_only:?}"
    );
}

/// Tests that en.json and de.json have the same set of camera names.
#[test]
fn test_en_and_de_have_same_camera_names() {
    let en: Value = load_json_file("assets/i18n/en.json");
    let de: Value = load_json_file("assets/i18n/de.json");

    let en_keys = get_camera_names(&en);
    let de_keys = get_camera_names(&de);

    let en_only: HashSet<_> = en_keys.difference(&de_keys).collect();
    let de_only: HashSet<_> = de_keys.difference(&en_keys).collect();

    assert!(
        en_only.is_empty(),
        "en.json has camera names not in de.json: {en_only:?}"
    );
    assert!(
        de_only.is_empty(),
        "de.json has camera names not in en.json: {de_only:?}"
    );
}

/// Tests that en.json and de.json have the same set of entity types.
#[test]
fn test_en_and_de_have_same_entity_types() {
    let en: Value = load_json_file("assets/i18n/en.json");
    let de: Value = load_json_file("assets/i18n/de.json");

    let en_keys = get_entity_types(&en);
    let de_keys = get_entity_types(&de);

    let en_only: HashSet<_> = en_keys.difference(&de_keys).collect();
    let de_only: HashSet<_> = de_keys.difference(&en_keys).collect();

    assert!(
        en_only.is_empty(),
        "en.json has entity types not in de.json: {en_only:?}"
    );
    assert!(
        de_only.is_empty(),
        "de.json has entity types not in en.json: {de_only:?}"
    );
}

/// Extracts action keys from the i18n JSON.
fn get_action_keys(json: &Value) -> HashSet<String> {
    json.get("ui")
        .and_then(|ui| ui.get("menu"))
        .and_then(|menu| menu.get("keybindings"))
        .and_then(|kb| kb.get("action"))
        .and_then(|action| action.as_object())
        .map(|obj| obj.keys().map(String::from).collect())
        .unwrap_or_default()
}

/// Extracts key names from the i18n JSON.
fn get_key_names(json: &Value) -> HashSet<String> {
    json.get("ui")
        .and_then(|ui| ui.get("menu"))
        .and_then(|menu| menu.get("keybindings"))
        .and_then(|kb| kb.get("key"))
        .and_then(|key| key.as_object())
        .map(|obj| obj.keys().map(String::from).collect())
        .unwrap_or_default()
}

/// Extracts group keys from the i18n JSON.
fn get_group_keys(json: &Value) -> HashSet<String> {
    json.get("ui")
        .and_then(|ui| ui.get("menu"))
        .and_then(|menu| menu.get("keybindings"))
        .and_then(|kb| kb.get("group"))
        .and_then(|group| group.as_object())
        .map(|obj| obj.keys().map(String::from).collect())
        .unwrap_or_default()
}

/// Extracts camera names from the i18n JSON.
fn get_camera_names(json: &Value) -> HashSet<String> {
    json.get("ui")
        .and_then(|ui| ui.get("notification"))
        .and_then(|notification| notification.get("camera_name"))
        .and_then(|camera_name| camera_name.as_object())
        .map(|obj| obj.keys().map(String::from).collect())
        .unwrap_or_default()
}

/// Extracts entity types from the i18n JSON.
fn get_entity_types(json: &Value) -> HashSet<String> {
    json.get("entity_types")
        .and_then(|entity_types| entity_types.as_object())
        .map(|obj| obj.keys().map(String::from).collect())
        .unwrap_or_default()
}
