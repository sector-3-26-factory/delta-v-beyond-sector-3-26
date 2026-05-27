// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for the configuration loader.
//!
//! See ADR-0021 (Testing strategy).

#[cfg(test)]
mod tests {
    // Test code is allowed to use expect/unwrap/indexing per ADR-0023.
    #![allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use std::io::Write;

    use tempfile::NamedTempFile;

    use crate::{
        error::ConfigError,
        loader::{load_and_validate_from_paths, merge_user_override},
    };

    /// Returns the absolute path to the workspace root, derived from
    /// `CARGO_MANIFEST_DIR` (which points at the crate directory).
    fn workspace_root() -> std::path::PathBuf {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .parent()
            .expect("crates dir")
            .parent()
            .expect("workspace root")
            .to_owned()
    }

    /// The shipped default keybindings file must load without errors.
    #[test]
    fn test_loads_default_keybindings_ok() {
        let root = workspace_root();
        let kb = load_and_validate_from_paths(
            &root.join("assets/config/keybindings.json"),
            &root.join("assets/json/schema/keybindings.schema.json"),
        )
        .expect("default keybindings should load without error");
        let kb: crate::keybindings::Keybindings = serde_json::from_value(kb).expect("deserialise");
        assert!(
            !kb.actions.is_empty(),
            "expected at least one action in default keybindings"
        );
        assert!(
            kb.actions.contains_key("thrust_forward"),
            "expected thrust_forward action"
        );
    }

    /// Pointing the loader at a nonexistent path must produce a
    /// [`ConfigError::Io`] variant.
    #[test]
    fn test_missing_defaults_file_errors() {
        let result = load_and_validate_from_paths(
            std::path::Path::new("/nonexistent/keybindings.json"),
            std::path::Path::new("assets/json/schema/keybindings.schema.json"),
        );
        assert!(
            matches!(result, Err(ConfigError::Io { .. })),
            "expected ConfigError::Io for missing file, got: {result:?}"
        );
    }

    /// A JSON object with an unknown top-level key must fail schema
    /// validation with [`ConfigError::Schema`].
    #[test]
    fn test_schema_violation_errors() {
        let schema_path = workspace_root().join("assets/json/schema/keybindings.schema.json");

        let bad_json = r#"{"unknown_key": true}"#;
        let mut tmp = NamedTempFile::new().expect("tempfile");
        write!(tmp, "{bad_json}").expect("write");

        let result = load_and_validate_from_paths(tmp.path(), &schema_path);
        assert!(
            matches!(result, Err(ConfigError::Schema { .. })),
            "expected ConfigError::Schema for unknown field, got: {result:?}"
        );
    }

    /// A user override that changes one action's keyboard binding must be
    /// reflected in the merged result; unchanged actions must survive.
    #[test]
    fn test_user_override_merges() {
        use serde_json::json;

        let mut base = json!({
            "actions": {
                "thrust_forward":  { "keyboard": ["KeyW"] },
                "thrust_backward": { "keyboard": ["KeyS"] }
            }
        });
        let override_value = json!({
            "actions": {
                "thrust_forward": { "keyboard": ["KeyT"] }
            }
        });

        merge_user_override(&mut base, override_value);

        let fwd = &base["actions"]["thrust_forward"]["keyboard"];
        assert_eq!(
            fwd,
            &json!(["KeyT"]),
            "override should replace thrust_forward binding"
        );

        let bwd = &base["actions"]["thrust_backward"]["keyboard"];
        assert_eq!(bwd, &json!(["KeyS"]), "thrust_backward should be unchanged");
    }
}
