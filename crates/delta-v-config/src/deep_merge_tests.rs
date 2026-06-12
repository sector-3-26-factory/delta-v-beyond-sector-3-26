// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for deep merge of user overrides.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use serde_json::json;

use crate::loader::merge_user_override;

// ---------------------------------------------------------------------------
// Nested object merge
// ---------------------------------------------------------------------------

#[test]
fn test_deep_merge_nested_objects() {
    let mut base = json!({
        "actions": {
            "thrust_forward": { "keyboard": ["KeyW"] },
            "thrust_backward": { "keyboard": ["KeyS"] }
        },
        "settings": {
            "sensitivity": 1.0,
            "invert_y": false
        }
    });

    let override_val = json!({
        "actions": {
            "thrust_forward": { "keyboard": ["KeyT"] }
        },
        "settings": {
            "sensitivity": 2.0
        }
    });

    merge_user_override(&mut base, override_val);

    // thrust_forward should be overridden
    assert_eq!(
        base["actions"]["thrust_forward"]["keyboard"],
        json!(["KeyT"])
    );
    // thrust_backward should be preserved
    assert_eq!(
        base["actions"]["thrust_backward"]["keyboard"],
        json!(["KeyS"])
    );
    // sensitivity should be overridden
    assert_eq!(base["settings"]["sensitivity"], json!(2.0));
    // invert_y should be preserved
    assert_eq!(base["settings"]["invert_y"], json!(false));
}

#[test]
fn test_deep_merge_array_replacement() {
    // Arrays should be replaced wholesale, not merged element-by-element
    let mut base = json!({
        "actions": {
            "thrust_forward": { "keyboard": ["KeyW", "ArrowUp"] }
        }
    });

    let override_val = json!({
        "actions": {
            "thrust_forward": { "keyboard": ["KeyT"] }
        }
    });

    merge_user_override(&mut base, override_val);

    // The entire array should be replaced
    assert_eq!(
        base["actions"]["thrust_forward"]["keyboard"],
        json!(["KeyT"]),
        "arrays should be replaced wholesale"
    );
}

#[test]
fn test_deep_merge_add_new_keys() {
    let mut base = json!({
        "actions": {
            "thrust_forward": { "keyboard": ["KeyW"] }
        }
    });

    let override_val = json!({
        "actions": {
            "pitch_up": { "keyboard": ["ArrowUp"] }
        }
    });

    merge_user_override(&mut base, override_val);

    // Original key preserved
    assert_eq!(
        base["actions"]["thrust_forward"]["keyboard"],
        json!(["KeyW"])
    );
    // New key added
    assert_eq!(base["actions"]["pitch_up"]["keyboard"], json!(["ArrowUp"]));
}

#[test]
fn test_deep_merge_scalar_override() {
    let mut base = json!({
        "settings": {
            "sensitivity": 1.0
        }
    });

    let override_val = json!({
        "settings": {
            "sensitivity": 3.5
        }
    });

    merge_user_override(&mut base, override_val);
    assert_eq!(base["settings"]["sensitivity"], json!(3.5));
}

#[test]
fn test_deep_merge_null_handling() {
    // When src is null, it should replace dst
    let mut base = json!({
        "actions": {
            "thrust_forward": { "keyboard": ["KeyW"] }
        }
    });

    let override_val = json!({
        "actions": {
            "thrust_forward": null
        }
    });

    merge_user_override(&mut base, override_val);
    assert!(
        base["actions"]["thrust_forward"].is_null(),
        "null should replace existing value"
    );
}

#[test]
fn test_deep_merge_deeply_nested() {
    let mut base = json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": "original"
                }
            }
        }
    });

    let override_val = json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": "overridden"
                }
            }
        }
    });

    merge_user_override(&mut base, override_val);
    assert_eq!(
        base["level1"]["level2"]["level3"]["value"],
        json!("overridden")
    );
}
