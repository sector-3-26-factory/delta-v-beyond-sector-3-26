// AGENTS: before modifying this file, read AGENTS.md at the repository root.
// allow-default: Bevy Resource trait requires Default for state-machine initialization.

//! Bevy resources produced by the world definition loader.

use std::ops::Deref;
use std::path::PathBuf;

use bevy::prelude::*;

use delta_v_types::WorldDef;

/// Path to the world definition file.
///
/// This resource is set by the CLI before the world is loaded.
/// It allows selecting different worlds via command-line arguments.
#[derive(Resource, Debug, Clone)]
pub struct WorldPath(pub PathBuf);

impl Default for WorldPath {
    fn default() -> Self {
        Self(PathBuf::from("assets/worlds/default.world.json"))
    }
}

impl Deref for WorldPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Loaded and validated world definition.
///
/// Inserted by [`crate::WorldPlugin`] during
/// [`delta_v_core::AppState::LoadingWorld`]. Downstream systems read
/// this resource to determine initial entity positions and spawn
/// parameters.
///
/// The inner [`WorldDef`] is guaranteed to have passed schema validation
/// (ADR-0012) and to contain no silent fallbacks (ADR-0013).
#[derive(Resource)]
pub struct WorldDefResource(pub WorldDef);
