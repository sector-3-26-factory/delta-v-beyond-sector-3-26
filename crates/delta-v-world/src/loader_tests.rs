// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for the world definition loader.
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

    use crate::{error::WorldError, loader::load_world_from_paths};

    /// Returns the absolute workspace root path.
    fn workspace_root() -> std::path::PathBuf {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .parent()
            .expect("crates dir")
            .parent()
            .expect("workspace root")
            .to_owned()
    }

    /// Returns the path to the test fixtures directory.
    fn fixtures_path() -> std::path::PathBuf {
        workspace_root().join("crates/delta-v-world/tests/fixtures")
    }

    /// The shipped default world file must load without errors.
    #[test]
    fn test_loads_default_world_ok() {
        let root = workspace_root();
        let world = load_world_from_paths(
            &root.join("assets/worlds/default.world.json"),
            &root.join("assets/json/schema/world.schema.json"),
        )
        .expect("default world should load without error");
        assert_eq!(world.format_version, 1);
        assert!(!world.name.is_empty(), "world name must not be empty");
    }

    /// The default world must have at least one entity (per ADR-0038).
    #[test]
    fn test_entities_present() {
        let root = workspace_root();
        let world = load_world_from_paths(
            &root.join("assets/worlds/default.world.json"),
            &root.join("assets/json/schema/world.schema.json"),
        )
        .expect("load");
        assert!(
            !world.entities.is_empty(),
            "world must have at least one entity"
        );
        // The first entity should reference a valid ship template.
        let player_entity = &world.entities[0];
        assert!(
            player_entity.template.starts_with("ships/"),
            "entity template should reference a ship"
        );
    }

    /// Pointing the loader at a nonexistent path must produce
    /// [`WorldError::Io`].
    #[test]
    fn test_missing_file_errors() {
        let root = workspace_root();
        let result = load_world_from_paths(
            std::path::Path::new("/nonexistent/world.json"),
            &root.join("assets/json/schema/world.schema.json"),
        );
        assert!(
            matches!(result, Err(WorldError::Io { .. })),
            "expected WorldError::Io, got: {result:?}"
        );
    }

    /// A JSON object that violates `additionalProperties: false` must
    /// produce [`WorldError::Schema`].
    #[test]
    fn test_schema_violation_errors() {
        let root = workspace_root();
        let schema_path = root.join("assets/json/schema/world.schema.json");

        let bad_json = r#"{"unknown_key": true}"#;
        let mut tmp = NamedTempFile::new().expect("tempfile");
        write!(tmp, "{bad_json}").expect("write");

        let result = load_world_from_paths(tmp.path(), &schema_path);
        assert!(
            matches!(result, Err(WorldError::Schema { .. })),
            "expected WorldError::Schema, got: {result:?}"
        );
    }

    /// Test world loading with isolated test fixtures.
    #[test]
    fn test_loads_test_world_ok() {
        let fixtures = fixtures_path();
        let world = load_world_from_paths(
            &fixtures.join("test.world.json"),
            &fixtures.join("world.schema.json"),
        )
        .expect("test world should load without error");
        assert_eq!(world.format_version, 1);
        assert_eq!(world.name, "Test World");
        assert_eq!(world.entities.len(), 1);
        assert_eq!(world.entities[0].template, "ships/test-ship");
        assert!(world.entities[0].player_controlled);
    }
}
