// AGENTS: before modifying this file, read AGENTS.md at the repository root.

#![cfg(test)]

//! Tests for template loading and merging.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::redundant_clone,
    clippy::single_char_pattern
)]

use serde_json::Value;

// ---------------------------------------------------------------------------
// Template merging logic tests
// ---------------------------------------------------------------------------

/// Simulates the merge logic from `load_player_controlled_ship` to verify
/// that base ship properties are preserved and player-controlled properties
/// override correctly.
#[test]
fn test_template_merge_base_preserved() {
    let base_ship = serde_json::json!({
        "entity_type": "ship",
        "mass": { "value": 10_000.0, "unit": "kg" },
        "inertia_scale": 1.0,
        "propulsion": {
            "main_thrusters": [{
                "id": "main",
                "type": "chemical",
                "max_forward_thrust": { "value": 100_000.0, "unit": "N" },
                "max_backward_thrust": { "value": 40_000.0, "unit": "N" }
            }],
            "maneuvering_thruster": {
                "type": "rcs",
                "max_torque": { "value": 50_000.0, "unit": "N⋅m" },
                "max_strafe_thrust": { "value": 50_000.0, "unit": "N" },
                "rotation_ramp_ticks": 60
            }
        }
    });

    let player_template = serde_json::json!({
        "entity_type": "player_controlled_ship",
        "cameras": {
            "cockpit": {
                "position": { "x": 0.0, "y": 0.5, "z": -0.2 },
                "target": { "x": 0.0, "y": 0.5, "z": -10.0 },
                "available": true
            },
            "drone": {
                "position": { "x": 0.0, "y": 2.0, "z": 5.0 },
                "target": { "x": 0.0, "y": 0.0, "z": 0.0 },
                "available": true
            }
        },
        "bounding_box": {
            "min": { "x": -1.0, "y": -1.0, "z": -1.0 },
            "max": { "x": 1.0, "y": 1.0, "z": 1.0 }
        }
    });

    // Simulate the merge logic from load_player_controlled_ship
    let mut merged = base_ship.clone();
    if let (Some(merged_obj), Some(player_obj)) =
        (merged.as_object_mut(), player_template.as_object())
    {
        for (key, value) in player_obj {
            if key != "entity_type" {
                merged_obj.insert(key.clone(), value.clone());
            }
        }
        merged_obj.insert(
            "entity_type".to_string(),
            Value::String("player_controlled_ship".to_string()),
        );
    }

    let merged = merged.as_object().unwrap();

    // Base ship properties should be preserved
    assert_eq!(
        merged["mass"]["value"], 10_000.0,
        "mass should be preserved from base ship"
    );
    assert_eq!(
        merged["inertia_scale"], 1.0,
        "inertia_scale should be preserved from base ship"
    );
    assert_eq!(
        merged["propulsion"]["main_thrusters"][0]["max_forward_thrust"]["value"], 100_000.0,
        "propulsion should be preserved from base ship"
    );

    // Player-controlled properties should be overlaid
    assert!(
        merged.contains_key("cameras"),
        "cameras should be added from player template"
    );
    assert!(
        merged.contains_key("bounding_box"),
        "bounding_box should be added from player template"
    );

    // entity_type should be player_controlled_ship
    assert_eq!(
        merged["entity_type"], "player_controlled_ship",
        "entity_type should be overridden to player_controlled_ship"
    );
}

#[test]
fn test_template_merge_player_overrides_base() {
    let base_ship = serde_json::json!({
        "entity_type": "ship",
        "mass": { "value": 10_000.0, "unit": "kg" },
        "inertia_scale": 1.0
    });

    // Player template overrides mass
    let player_template = serde_json::json!({
        "entity_type": "player_controlled_ship",
        "mass": { "value": 15_000.0, "unit": "kg" }
    });

    let mut merged = base_ship;
    if let (Some(merged_obj), Some(player_obj)) =
        (merged.as_object_mut(), player_template.as_object())
    {
        for (key, value) in player_obj {
            if key != "entity_type" {
                merged_obj.insert(key.clone(), value.clone());
            }
        }
        merged_obj.insert(
            "entity_type".to_string(),
            Value::String("player_controlled_ship".to_string()),
        );
    }

    let merged = merged.as_object().unwrap();
    // Player template's mass should override base ship's mass
    assert_eq!(
        merged["mass"]["value"], 15_000.0,
        "player template should override base ship mass"
    );
}

#[test]
fn test_template_merge_entity_type_not_overridden_by_base() {
    // Even if base ship has entity_type: "ship", the merged result
    // should always be "player_controlled_ship"
    let base_ship = serde_json::json!({
        "entity_type": "ship"
    });

    let player_template = serde_json::json!({
        "entity_type": "player_controlled_ship"
    });

    let mut merged = base_ship;
    if let (Some(merged_obj), Some(player_obj)) =
        (merged.as_object_mut(), player_template.as_object())
    {
        for (key, value) in player_obj {
            if key != "entity_type" {
                merged_obj.insert(key.clone(), value.clone());
            }
        }
        merged_obj.insert(
            "entity_type".to_string(),
            Value::String("player_controlled_ship".to_string()),
        );
    }

    assert_eq!(
        merged["entity_type"], "player_controlled_ship",
        "entity_type must always be player_controlled_ship after merge"
    );
}

// ---------------------------------------------------------------------------
// resolve_template_path tests
// ---------------------------------------------------------------------------

#[test]
fn test_resolve_template_path() {
    let path = crate::paths::resolve_template_path("ships", "meshy-cargo-1");
    assert_eq!(path, "templates/ships/meshy-cargo-1/");
}

#[test]
fn test_resolve_template_path_asteroids() {
    let path = crate::paths::resolve_template_path("asteroids", "meshy-asteroid-2");
    assert_eq!(path, "templates/asteroids/meshy-asteroid-2/");
}

#[test]
fn test_resolve_template_path_from_struct() {
    let template = delta_v_types::TemplatePath::new("ships", "debug-ship-cube");
    let path = crate::paths::resolve_template_path_from(&template);
    assert_eq!(path, "templates/ships/debug-ship-cube/");
}

#[test]
fn test_resolve_template_path_stations() {
    let path = crate::paths::resolve_template_path("stations", "orbital-habitat");
    assert_eq!(path, "templates/stations/orbital-habitat/");
}

#[test]
fn test_resolve_template_path_with_hyphenated_name() {
    let path = crate::paths::resolve_template_path("ships", "debug-ship-cube");
    assert_eq!(path, "templates/ships/debug-ship-cube/");
}

#[test]
fn test_resolve_template_path_with_numeric_name() {
    let path = crate::paths::resolve_template_path("asteroids", "asteroid-42");
    assert_eq!(path, "templates/asteroids/asteroid-42/");
}

#[test]
fn test_resolve_template_path_from_struct_asteroids() {
    let template = delta_v_types::TemplatePath::new("asteroids", "meshy-asteroid-2");
    let path = crate::paths::resolve_template_path_from(&template);
    assert_eq!(path, "templates/asteroids/meshy-asteroid-2/");
}

#[test]
fn test_resolve_template_path_from_struct_stations() {
    let template = delta_v_types::TemplatePath::new("stations", "deep-space-relay");
    let path = crate::paths::resolve_template_path_from(&template);
    assert_eq!(path, "templates/stations/deep-space-relay/");
}

#[test]
fn test_resolve_template_path_trailing_slash() {
    // All paths should end with a trailing slash
    let path = crate::paths::resolve_template_path("ships", "test");
    assert!(
        path.ends_with("/"),
        "path should end with trailing slash, got: {path}"
    );
    assert!(
        path.starts_with("templates/"),
        "path should start with templates/, got: {path}"
    );
}

#[test]
fn test_resolve_template_path_from_struct_trailing_slash() {
    let template = delta_v_types::TemplatePath::new("category", "name");
    let path = crate::paths::resolve_template_path_from(&template);
    assert!(
        path.ends_with("/"),
        "path should end with trailing slash, got: {path}"
    );
    assert!(
        path.starts_with("templates/"),
        "path should start with templates/, got: {path}"
    );
}

// ---------------------------------------------------------------------------
// Planet template validation tests
// ---------------------------------------------------------------------------

#[test]
fn test_load_planet_mercury() {
    // Validate that the mercury planet template is valid against the schema
    let result = crate::template::load_planet("mercury");
    assert!(
        result.is_ok(),
        "mercury planet template should be valid: {:?}",
        result.unwrap_err()
    );
}

#[test]
fn test_load_planet_pluto() {
    // Validate that the pluto planet template is valid against the schema
    let result = crate::template::load_planet("pluto");
    assert!(
        result.is_ok(),
        "pluto planet template should be valid: {:?}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// World definition loading tests
// ---------------------------------------------------------------------------

/// Returns the path to the test fixtures directory.
fn fixtures_path() -> std::path::PathBuf {
    crate::paths::get_workspace_root().join("crates/delta-v-world/tests/fixtures")
}

/// The shipped default world file must load without errors.
#[test]
fn test_loads_default_world_ok() {
    let root = crate::paths::get_workspace_root();
    let world = crate::template::load_world(&root.join("assets/worlds/default/world.json"))
        .expect("default world should load without error");
    assert_eq!(world.format_version, 1);
    assert!(!world.name.is_empty(), "world name must not be empty");
}

/// The default world must have at least one entity (per ADR-0038).
#[test]
fn test_entities_present() {
    let root = crate::paths::get_workspace_root();
    let world =
        crate::template::load_world(&root.join("assets/worlds/default/world.json")).expect("load");
    assert!(
        !world.entities.is_empty(),
        "world must have at least one entity"
    );
    // The first entity should reference a valid ship template.
    // Check template_short since template is loaded later by build_spawn_event.
    let player_entity = &world.entities[0];
    assert!(
        player_entity.template_short.starts_with("ships/"),
        "entity template should reference a ship"
    );
}

/// Pointing the loader at a nonexistent path must produce [`AssetError::Io`].
#[test]
fn test_missing_file_errors() {
    let _root = crate::paths::get_workspace_root();
    let result = crate::template::load_world(std::path::Path::new("/nonexistent/world.json"));
    assert!(
        matches!(result, Err(crate::error::AssetError::Io { .. })),
        "expected AssetError::Io, got: {result:?}"
    );
}

/// A JSON object that violates `additionalProperties: false` must
/// produce [`AssetError::Validation`].
#[test]
fn test_schema_violation_errors() {
    let bad_json = r#"{"unknown_key": true}"#;
    let mut tmp = tempfile::NamedTempFile::new().expect("tempfile");
    std::io::Write::write_all(&mut tmp, bad_json.as_bytes()).expect("write");

    let result = crate::template::load_world(tmp.path());
    assert!(
        matches!(result, Err(crate::error::AssetError::Validation(_))),
        "expected AssetError::Validation, got: {result:?}"
    );
}

/// Test world loading with isolated test fixtures.
#[test]
fn test_loads_test_world_ok() {
    let fixtures = fixtures_path();
    let world = crate::template::load_world(&fixtures.join("test.world.json"))
        .expect("test world should load without error");
    assert_eq!(world.format_version, 1);
    assert_eq!(world.name, "Test World");
    assert_eq!(world.entities.len(), 1);
    // The template field is loaded later by build_spawn_event, so check template_short
    assert_eq!(world.entities[0].template_short, "ships/debug-ship-cube");
    assert!(world.entities[0].player_controlled);
}

// ---------------------------------------------------------------------------
// ADR-0058: templates describe geometry only
// ---------------------------------------------------------------------------

/// No shipped template may carry a `mass`. Mass belongs to the world entity,
/// because one mesh is reused at many sizes and the bodies are not one object
/// scaled up. This walks the templates on disk so a future template cannot
/// reintroduce the field.
#[test]
fn test_no_template_carries_a_mass() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/templates");
    let mut checked = 0usize;

    for category in std::fs::read_dir(&root).expect("assets/templates must exist") {
        let category = category.expect("dir entry").path();
        if !category.is_dir() {
            continue;
        }
        for template in std::fs::read_dir(&category).expect("category must be readable") {
            let template = template.expect("dir entry").path();
            if !template.is_dir() {
                continue;
            }
            for entry in std::fs::read_dir(&template).expect("template must be readable") {
                let path = entry.expect("dir entry").path();
                if path.extension().and_then(std::ffi::OsStr::to_str) != Some("json") {
                    continue;
                }
                let raw = std::fs::read_to_string(&path).expect("template must be readable");
                let value: serde_json::Value =
                    serde_json::from_str(&raw).expect("template must be valid JSON");
                assert!(
                    value.get("mass").is_none(),
                    "{} carries a mass; per ADR-0058 mass belongs to the world entity",
                    path.display()
                );
                checked += 1;
            }
        }
    }

    assert!(checked > 0, "no template files were inspected");
}

/// Template `entity_type` values whose world entities are bodies, i.e. the
/// types that reach `require_mass` at spawn time. A belt or a field is a
/// definition rather than a body and is deliberately absent (ADR-0058).
const BODY_ENTITY_TYPES: &[&str] = &[
    "sun",
    "planet",
    "moon",
    "asteroid",
    "ship",
    "player_controlled_ship",
    "ai_controlled_ship",
];

/// Returns `true` when the referenced template describes a body rather than a
/// region definition.
///
/// Per ADR-0038 a template file is named after its `entity_type`, so the file
/// name answers this without reading the template. A player-controlled ship
/// directory holds both `ship.json` and `player_controlled_ship.json`; both
/// name a body, so any match is enough.
fn template_produces_a_body(template_short: &str) -> bool {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/templates")
        .join(template_short);
    std::fs::read_dir(&directory)
        .expect("template directory must be readable")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().and_then(std::ffi::OsStr::to_str) == Some("json"))
        .filter_map(|path| {
            path.file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .map(str::to_string)
        })
        .any(|stem| BODY_ENTITY_TYPES.contains(&stem.as_str()))
}

/// Every body in a shipped world declares a mass on its world entity.
///
/// This is the JSON-level half of the ADR-0058 pair: it checks the content, so
/// a mistake is reported against the world file that carries it. The
/// code-level half lives in `delta-v-world` (`lib_tests.rs`), which spawns the
/// same entities and reads the mass off the spawned body.
///
/// Belt and field entities are definitions rather than bodies and carry no
/// mass; their generated bodies derive a mass from the region density, so they
/// are skipped here.
#[test]
fn test_every_shipped_world_body_declares_a_mass() {
    let worlds = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/worlds");
    let mut checked = 0usize;

    for dir_entry in std::fs::read_dir(&worlds).expect("assets/worlds must exist") {
        let dir = dir_entry.expect("dir entry").path();
        if !dir.is_dir() {
            continue;
        }
        let path = dir.join("world.json");
        if !path.exists() {
            continue;
        }
        // Loaded through delta-v-json like the game does (ADR-0040), so the
        // file is schema-validated on the way in and `mass` is absent exactly
        // when the world file omits it.
        let world = crate::template::load_world(&path).expect("world file must load");
        for entity in &world.entities {
            if !template_produces_a_body(&entity.template_short) {
                continue;
            }
            assert!(
                entity.mass.is_some(),
                "{} entity '{}' declares no mass; the spawn system requires one (ADR-0058)",
                path.display(),
                entity.id
            );
            checked += 1;
        }
    }

    assert!(checked > 0, "no world entities were inspected");
}
