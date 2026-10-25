// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Scene lighting setup utilities.

use bevy::prelude::{Commands, DirectionalLight, EulerRot, Quat, Transform, Visibility, default};

/// Sets up scene lighting (directional + ambient). Called once per world load.
///
/// Creates a directional light simulating a distant sun, rotated to create
/// interesting shadows. Ambient fill is provided via the
/// [`GlobalAmbientLight`](bevy::light::GlobalAmbientLight) resource, which
/// `LightPlugin` already initialises — no standalone `AmbientLight` entity is
/// spawned.
///
/// Spawning a standalone `AmbientLight` component is forbidden here: in Bevy
/// 0.19 `AmbientLight` carries `#[require(Camera)]`, so it would silently add a
/// bare `Camera` with no render graph configured, producing a warning and a
/// useless render target. The `GlobalAmbientLight` resource lights the whole
/// scene without needing a camera entity.
pub fn setup_scene_lighting(commands: &mut Commands<'_, '_>) {
    // Directional light (sun-like): rotated to create interesting shadows.
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
        Visibility::default(),
    ));
}
