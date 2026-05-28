// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Ship entity spawning from templates.
//!
//! See ADR-0038 (Entity template system) and ADR-0005 (Plugin architecture).

use bevy::prelude::*;
use delta_v_world::SpawnEntity;

use crate::components::PlayerShip;

/// Spawns ship entities in response to `SpawnEntity` events.
///
/// Listens for events with `entity_type` matching known ship types:
/// - `"local_player_ship"`: Player-controlled ship
/// - `"npc_ship"`: NPC-controlled ship (future)
///
/// Creates a `PbrBundle` with a primitive capsule mesh and inserts
/// the `PlayerShip` marker component for player ships.
pub fn spawn_ship_from_template(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
    mut events: EventReader<'_, '_, SpawnEntity>,
) {
    for event in events.read() {
        if event.entity_type == "local_player_ship" {
            spawn_player_ship(&mut commands, &mut meshes, &mut materials, event);
        }
        // Other ship types (e.g., "npc_ship") handled in future
    }
}

/// Spawns the player-controlled ship from a template event.
fn spawn_player_ship(
    commands: &mut Commands<'_, '_>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    materials: &mut ResMut<'_, Assets<StandardMaterial>>,
    event: &SpawnEntity,
) {
    // Create capsule mesh: 0.5m radius, 2m length.
    let mesh = meshes.add(Capsule3d::new(0.5, 2.0));

    // Create neutral blue-gray material.
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.7, 0.9),
        ..default()
    });

    // Build transform from event data.
    let transform = Transform {
        translation: event.position,
        rotation: event.rotation,
        scale: event.scale,
    };

    // Spawn the ship entity.
    commands.spawn((
        PbrBundle {
            mesh,
            material,
            transform,
            ..default()
        },
        PlayerShip,
    ));

    log::info!(
        "local player ship spawned at position ({:.1}, {:.1}, {:.1})",
        event.position.x,
        event.position.y,
        event.position.z
    );
}

/// Spawns lighting for the 3-D scene.
///
/// Runs once per world load. Creates:
/// - A directional light simulating a distant sun.
/// - Ambient light for general illumination.
pub fn setup_scene_lighting(mut commands: Commands<'_, '_>) {
    // Directional light (sun-like): rotated to create interesting shadows.
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
        ..default()
    });

    // Ambient light for general scene fill.
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
    });

    log::info!("scene lighting initialized");
}
