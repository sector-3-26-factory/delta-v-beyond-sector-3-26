// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Scene lighting setup utilities.

use bevy::prelude::{Commands, DirectionalLight};

/// Sets up scene lighting (directional + ambient). Called once per world load.
pub fn setup_scene_lighting(commands: &mut Commands<'_, '_>) {
    // Main directional light
    commands.spawn((DirectionalLight {
        illuminance: 15000.0,
        ..Default::default()
    },));
}
