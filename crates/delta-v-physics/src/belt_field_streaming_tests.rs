// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Unit tests for `belt_field_streaming` module.
//!
//! These tests create minimal in-memory entities to test the streaming logic
//! directly, without the full world loading pipeline or `ShipsPlugin`.

use crate::belt_field_generation::SizeDistributionEntry;
use crate::belt_field_spawn::AsteroidBelt;
use crate::belt_field_streaming::{
    LoadedSectors, belt_sector_streaming_system, calculate_sector_index,
};
use crate::celestial::OrbitalBody;
use crate::constants::FIXED_TIMESTEP_HZ;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::{Time, TimePlugin};
use delta_v_core::{AppState, PlayerShipEntity};
use delta_v_types::AsteroidBeltTemplate;
use delta_v_types::PlayerSettings;

/// Builds a minimal app with only the streaming system and required resources.
fn build_streaming_app() -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin)
        .add_plugins(AssetPlugin::default())
        .add_plugins(StatesPlugin)
        .insert_resource(Time::<Fixed>::from_hz(f64::from(FIXED_TIMESTEP_HZ)))
        .init_resource::<LoadedSectors>()
        .init_resource::<crate::belt_field_generation::AsteroidDeltaStore>()
        .insert_resource(PlayerSettings {
            language: "en".to_string(),
            detail_level: 1.0,
        })
        .init_state::<AppState>()
        .add_systems(FixedUpdate, belt_sector_streaming_system);
    // Set state to InGame so the streaming system runs
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::InGame);
    app
}

/// Runs the `FixedUpdate` schedule once by advancing time.
fn run_fixed_update(app: &mut App) {
    let fixed_duration = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(fixed_duration);
    app.update();
}

/// Creates a test belt entity with the given parameters.
fn spawn_test_belt(
    app: &mut App,
    id: &str,
    orbital_distance: f32,
    radial_extent: f32,
    thickness: f32,
    sector_size: f32,
) -> Entity {
    let belt = AsteroidBelt {
        template: AsteroidBeltTemplate {
            entity_type: "asteroid_belt".to_string(),
            meshes: vec!["asteroid_small".to_string()],
            density: 2800.0,
        },
        id: id.to_string(),
        radial_extent,
        thickness,
        mean_spacing: 23000.0,
        excluded_radii: vec![],
        seed: 20_260_930,
        sector_size,
        size_distribution: vec![SizeDistributionEntry {
            radius: 1000.0,
            weight: 1.0,
        }],
        density: 2.8e-29,
    };

    // Create a dummy parent entity for the orbital parent
    let sun_entity = app.world_mut().spawn_empty().id();

    let orbital_body = OrbitalBody {
        orbital_parent: sun_entity,
        orbital_distance,
        orbital_period: 140_000_000.0,
        orbital_eccentricity: 0.12,
        orbital_inclination: 10.0_f32.to_radians(),
        initial_orbital_angle: 0.0,
    };

    app.world_mut()
        .spawn((
            belt,
            orbital_body,
            crate::OrbitalParentId("sun".to_string()),
            GlobalTransform::default(),
            Transform::default(),
        ))
        .id()
}

/// Creates a test player ship entity at the given position.
fn spawn_test_player(app: &mut App, position: Vec3) -> Entity {
    let entity = app
        .world_mut()
        .spawn((
            GlobalTransform::from_translation(position),
            Transform::from_translation(position),
        ))
        .id();

    // Insert the PlayerShipEntity resource
    app.insert_resource(PlayerShipEntity(entity));

    entity
}

#[test]
fn test_calculate_sector_index() {
    let idx = calculate_sector_index("sun", 4.04e11, 5.0e7, Vec3::new(4.04e11, 0.0, 0.0));
    assert_eq!(idx, 0);

    let idx = calculate_sector_index("sun", 4.04e11, 5.0e7, Vec3::new(0.0, 4.04e11, 0.0));
    assert!(idx > 0);
}

/// Test that a sector loads when the player is within the belt's volume (in sector)
#[test]
fn test_belt_sector_loads_when_player_in_sector() {
    let mut app = build_streaming_app();

    // Belt at 404M km from sun, 44.9M km radial extent, 15M km thickness
    let orbital_distance = 4.04e11;
    let radial_extent = 4.49e10;
    let thickness = 1.5e10;
    let sector_size = 5.0e7;

    spawn_test_belt(
        &mut app,
        "test_belt",
        orbital_distance,
        radial_extent,
        thickness,
        sector_size,
    );

    // Player at 404M km (in the belt, on the ring)
    let player_pos = Vec3::new(4.04e11, 0.0, 0.0);
    spawn_test_player(&mut app, player_pos);

    // Run the streaming system (FixedUpdate)
    run_fixed_update(&mut app);

    // Check that a sector was loaded
    let loaded_sectors = app.world().resource::<LoadedSectors>();
    assert!(
        !loaded_sectors.belt_sectors.is_empty(),
        "belt sector should be loaded when player is in belt volume"
    );
    assert_eq!(loaded_sectors.belt_sectors.len(), 1);
    assert_eq!(
        loaded_sectors
            .belt_sectors
            .first()
            .map(|(id, _)| id.as_str()),
        Some("test_belt")
    );
}

/// Test that a sector does NOT load when the player is above the belt (outside thickness)
#[test]
fn test_belt_sector_not_load_when_player_above_belt() {
    let mut app = build_streaming_app();

    let orbital_distance = 4.04e11;
    let radial_extent = 4.49e10;
    let thickness = 1.5e10;
    let sector_size = 5.0e7;

    spawn_test_belt(
        &mut app,
        "test_belt",
        orbital_distance,
        radial_extent,
        thickness,
        sector_size,
    );

    // Player at z=20M km (above belt thickness of 15M km)
    let player_pos = Vec3::new(4.04e11, 0.0, 2.0e10);
    spawn_test_player(&mut app, player_pos);

    run_fixed_update(&mut app);

    let loaded_sectors = app.world().resource::<LoadedSectors>();
    assert!(
        loaded_sectors.belt_sectors.is_empty(),
        "belt sector should NOT load when player is above belt thickness"
    );
}

/// Test that a sector does NOT load when player is beneath the belt (outside thickness)
#[test]
fn test_belt_sector_not_load_when_player_beneath_belt() {
    let mut app = build_streaming_app();

    let orbital_distance = 4.04e11;
    let radial_extent = 4.49e10;
    let thickness = 1.5e10;
    let sector_size = 5.0e7;

    spawn_test_belt(
        &mut app,
        "test_belt",
        orbital_distance,
        radial_extent,
        thickness,
        sector_size,
    );

    // Player at z=-20M km (beneath belt thickness of 15M km)
    let player_pos = Vec3::new(4.04e11, 0.0, -2.0e10);
    spawn_test_player(&mut app, player_pos);

    run_fixed_update(&mut app);

    let loaded_sectors = app.world().resource::<LoadedSectors>();
    assert!(
        loaded_sectors.belt_sectors.is_empty(),
        "belt sector should NOT load when player is beneath belt thickness"
    );
}

/// Test that a sector does NOT load when player is before the belt (towards the sun)
#[test]
fn test_belt_sector_not_load_when_player_before_belt() {
    let mut app = build_streaming_app();

    let orbital_distance = 4.04e11;
    let radial_extent = 4.49e10;
    let thickness = 1.5e10;
    let sector_size = 5.0e7;

    spawn_test_belt(
        &mut app,
        "test_belt",
        orbital_distance,
        radial_extent,
        thickness,
        sector_size,
    );

    // Player at 359M km (before inner edge of belt at ~359.1M km)
    let player_pos = Vec3::new(3.59e11, 0.0, 0.0);
    spawn_test_player(&mut app, player_pos);

    run_fixed_update(&mut app);

    let loaded_sectors = app.world().resource::<LoadedSectors>();
    assert!(
        loaded_sectors.belt_sectors.is_empty(),
        "belt sector should NOT load when player is before the belt (towards sun)"
    );
}

/// Test that a sector does NOT load when player is behind the belt (away from sun)
#[test]
fn test_belt_sector_not_load_when_player_behind_belt() {
    let mut app = build_streaming_app();

    let orbital_distance = 4.04e11;
    let radial_extent = 4.49e10;
    let thickness = 1.5e10;
    let sector_size = 5.0e7;

    spawn_test_belt(
        &mut app,
        "test_belt",
        orbital_distance,
        radial_extent,
        thickness,
        sector_size,
    );

    // Player at 449M km (beyond outer edge of belt at ~448.9M km)
    let player_pos = Vec3::new(4.49e11, 0.0, 0.0);
    spawn_test_player(&mut app, player_pos);

    run_fixed_update(&mut app);

    let loaded_sectors = app.world().resource::<LoadedSectors>();
    assert!(
        loaded_sectors.belt_sectors.is_empty(),
        "belt sector should NOT load when player is behind the belt (away from sun)"
    );
}

/// Test that sector loads when player is within streaming distance but not exactly on ring
#[test]
fn test_belt_sector_loads_when_player_near_ring() {
    let mut app = build_streaming_app();

    let orbital_distance = 4.04e11;
    let radial_extent = 4.49e10;
    let thickness = 1.5e10;
    let sector_size = 5.0e7;

    spawn_test_belt(
        &mut app,
        "test_belt",
        orbital_distance,
        radial_extent,
        thickness,
        sector_size,
    );

    // Player clearly inside the ring (50,000 km inside, well within streaming distance of 100,000 km)
    // Ring is at 404M km, so 403.95M km is 50,000 km inside
    let player_pos = Vec3::new(4.0395e11, 0.0, 0.0);
    spawn_test_player(&mut app, player_pos);

    run_fixed_update(&mut app);

    let loaded_sectors = app.world().resource::<LoadedSectors>();
    assert!(
        !loaded_sectors.belt_sectors.is_empty(),
        "belt sector should load when player is within streaming distance of ring"
    );
}

/// Test that sector does NOT load when player is outside streaming distance
#[test]
fn test_belt_sector_not_load_when_player_far_from_ring() {
    let mut app = build_streaming_app();

    let orbital_distance = 4.04e11;
    let radial_extent = 4.49e10;
    let thickness = 1.5e10;
    let sector_size = 5.0e7;

    spawn_test_belt(
        &mut app,
        "test_belt",
        orbital_distance,
        radial_extent,
        thickness,
        sector_size,
    );

    // Player far from the ring (outside streaming distance)
    let player_pos = Vec3::new(3.0e11, 0.0, 0.0);
    spawn_test_player(&mut app, player_pos);

    run_fixed_update(&mut app);

    let loaded_sectors = app.world().resource::<LoadedSectors>();
    assert!(
        loaded_sectors.belt_sectors.is_empty(),
        "belt sector should NOT load when player is outside streaming distance"
    );
}
