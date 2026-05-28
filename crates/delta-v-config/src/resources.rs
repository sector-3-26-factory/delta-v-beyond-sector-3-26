// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Bevy resources produced by the configuration loader.
//!
//! Resources defined here are inserted during [`AppState::LoadingDefaults`]
//! and remain available for the lifetime of the application.

use bevy::prelude::*;

use crate::keybindings::Keybindings;

/// Loaded and validated keybindings.
///
/// Inserted as a Bevy resource by [`crate::ConfigPlugin`] during
/// [`delta_v_core::AppState::LoadingDefaults`]. All gameplay systems that
/// need to query which keys are bound to which actions read this resource.
///
/// The inner [`Keybindings`] value is guaranteed to have passed schema
/// validation (ADR-0012) and to contain no silent fallbacks (ADR-0013).
#[derive(Resource)]
pub struct KeybindingsResource(pub Keybindings);
