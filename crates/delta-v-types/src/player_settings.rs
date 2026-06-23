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

//! Player settings configuration.
//!
//! Player preferences that control game behavior. Loaded from
//! `assets/config/player_settings.json` with optional user overrides
//! from `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/player_settings.json`
//! per ADR-0010.
//!
//! Per ADR-0013 (no silent fallbacks), missing or invalid player settings
//! is a hard error.

use bevy::prelude::Resource;
use serde::Deserialize;

/// Player settings loaded from `player_settings.json`.
///
/// Controls player-configurable options like language selection.
/// Supports hot-reload in dev builds (ADR-0035).
#[derive(Resource, Debug, Clone, Deserialize)]
pub struct PlayerSettings {
    /// ISO 639-1 language code for UI translations (e.g. 'en', 'de').
    /// The game loads `assets/i18n/<language>.json` at startup.
    pub language: String,
}
