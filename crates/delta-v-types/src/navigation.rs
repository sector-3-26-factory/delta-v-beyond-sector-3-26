// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Shared navigation types for entity identification in targeting/navigation lists.
//!
//! These types are used by multiple crates (delta-v-core, delta-v-ships, delta-v-world, delta-v-ui)
//! and must live in delta-v-types per ADR-0046.
//!
//! Per ADR-0046, this crate MUST NOT contain Component derives — only plain types.
//! Components using these types are defined in the domain crates that need them.

/// Plain type for entity type identification in navigation lists.
///
/// This is a plain struct used for JSON deserialization and data transfer.
/// Components using this type are defined in domain crates.
#[derive(Debug, Clone)]
pub struct EntityType(pub String);

/// Plain type for entity ID from world definitions.
///
/// This is a plain struct used for JSON deserialization and data transfer.
/// Components using this type are defined in domain crates.
/// Note: This is the string ID from the world definition (e.g., "ship-1", "asteroid-5"),
/// not to be confused with `delta_v_types::ids::EntityId` which wraps Bevy's Entity.
#[derive(Debug, Clone)]
pub struct WorldEntityId(pub String);
