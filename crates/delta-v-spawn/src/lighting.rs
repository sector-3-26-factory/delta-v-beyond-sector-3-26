// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Scene lighting setup utilities.

use bevy::prelude::{
    AmbientLight, Color, Commands, DirectionalLight, EulerRot, Quat, Transform, Visibility, default,
};

/// Sets up scene lighting (directional + ambient). Called once per world load.
///
/// Creates:
/// - A directional light simulating a distant sun, rotated to create interesting shadows.
/// - Ambient light for general scene fill.
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

    // Ambient light for general scene fill.
    // In Bevy 0.18, AmbientLight is a component, not a resource.
    commands.spawn(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
        affects_lightmapped_meshes: true,
    });
}
