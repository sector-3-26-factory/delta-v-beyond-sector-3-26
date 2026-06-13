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

//! Networking infrastructure (placeholder).
//!
//! Concrete networking decisions are deferred to ADR-0030 (Authoritative model),
//! ADR-0031 (Network library choice), and ADR-0032 (Snapshot and delta encoding).

use bevy::prelude::*;

/// Networking plugin (stub for M0.5).
pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, _app: &mut App) {
        log::info!("NetPlugin initialized");
    }
}
