// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Integration tests for ships plugin.
//!
//! See ADR-0021 (Testing strategy).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use bevy::app::App;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use delta_v_core::AppState;
use delta_v_ships::cockpit::components::ActiveCockpitStation;

/// Integration test: verify `SelectedTarget` resource can be set and read.
#[test]
fn test_selected_target_resource_update() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: "assets".to_string(),
        ..default()
    });
    app.add_plugins(StatesPlugin);
    app.init_state::<AppState>();

    // Add required resources
    app.insert_resource(delta_v_core::navigation::SelectedTarget(None));
    app.insert_resource(delta_v_core::navigation::TargetingMode::default());
    app.insert_resource(delta_v_core::NavigationListData::default());

    // Create a targetable entity
    let target_entity = app
        .world_mut()
        .spawn((
            Name::new("TestTarget"),
            delta_v_core::health::Targetable,
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        ))
        .id();

    // Set the selected target
    app.world_mut()
        .resource_mut::<delta_v_core::navigation::SelectedTarget>()
        .0 = Some(target_entity);

    // Verify SelectedTarget resource was updated
    let selected = app
        .world()
        .get_resource::<delta_v_core::navigation::SelectedTarget>()
        .unwrap();
    assert!(
        selected.0.is_some(),
        "SelectedTarget should have a value after selection"
    );
    assert_eq!(
        selected.0.unwrap(),
        target_entity,
        "SelectedTarget should reference the correct entity"
    );
}

/// Integration test: verify `ActiveCockpitStation` resource is set correctly.
#[test]
fn test_active_cockpit_station_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: "assets".to_string(),
        ..default()
    });
    app.add_plugins(StatesPlugin);
    app.init_state::<AppState>();

    // Insert the ActiveCockpitStation resource
    app.insert_resource(ActiveCockpitStation {
        station_id: "default".to_string(),
    });

    // Verify the resource exists
    let active_station = app.world().get_resource::<ActiveCockpitStation>();
    assert!(
        active_station.is_some(),
        "ActiveCockpitStation resource should be set"
    );
    assert_eq!(
        active_station.unwrap().station_id,
        "default",
        "station_id should match"
    );
}

/// Integration test: verify `ShipsPlugin` can be added to an app without errors.
#[test]
fn test_ships_plugin_builds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: "assets".to_string(),
        ..default()
    });
    app.add_plugins(StatesPlugin);
    app.init_state::<AppState>();

    // Add the ships plugin - this should not panic
    app.add_plugins(delta_v_ships::ShipsPlugin);

    // Verify the plugin registered its public resources
    assert!(
        app.world()
            .contains_resource::<delta_v_ships::ThrustCommand>()
    );
    assert!(
        app.world()
            .contains_resource::<delta_v_ships::TorqueCommand>()
    );
    // Note: PreviousActions and RotationRampState are internal (not pub)
    // so we don't test for them here
}
