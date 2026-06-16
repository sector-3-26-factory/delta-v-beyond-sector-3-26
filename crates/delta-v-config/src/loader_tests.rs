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

    use crate::error::ConfigError;
    use crate::loader::load_and_validate_from_paths;
    use delta_v_assets::get_workspace_root;

    /// The shipped default keybindings file must load without errors.
    #[test]
    fn test_loads_default_keybindings_ok() {
        let root = get_workspace_root();
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
        assert!(
            kb.actions.contains_key("toggle_flight_assist"),
            "expected toggle_flight_assist action (M2 flight assist feature)"
        );
    }

    /// The shipped default flight-assist file must load without errors.
    #[test]
    fn test_loads_default_flight_assist_ok() {
        let root = get_workspace_root();
        let fa = load_and_validate_from_paths(
            &root.join("assets/config/flight-assist.json"),
            &root.join("assets/json/schema/flight-assist.schema.json"),
        )
        .expect("default flight-assist config should load without error");
        let fa: delta_v_core::FlightAssistConfig = serde_json::from_value(fa).expect("deserialise");
        assert!(
            fa.enabled_by_default,
            "expected enabled_by_default to be true"
        );
        assert!(
            (fa.damping_coefficient - 0.05).abs() < f32::EPSILON,
            "expected damping_coefficient 0.05"
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
        let schema_path = get_workspace_root().join("assets/json/schema/keybindings.schema.json");

        let bad_json = r#"{"unknown_key": true}"#;
        let mut tmp = NamedTempFile::new().expect("tempfile");
        write!(tmp, "{bad_json}").expect("write");

        let result = load_and_validate_from_paths(tmp.path(), &schema_path);
        assert!(
            matches!(result, Err(ConfigError::Schema { .. })),
            "expected ConfigError::Schema for unknown field, got: {result:?}"
        );
    }
}
