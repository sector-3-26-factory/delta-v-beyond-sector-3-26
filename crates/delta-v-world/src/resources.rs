// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Bevy resources produced by the world definition loader.

use bevy::prelude::*;

use crate::world_def::WorldDef;

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
