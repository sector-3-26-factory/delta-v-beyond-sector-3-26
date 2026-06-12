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

//! Shared JSON loading, schema validation and default-fill utilities.
//!
//! This crate provides the single implementation of the JSON pipeline
//! used by all JSON-backed crates in the workspace:
//!
//! ```text
//! read_json -> validate -> fill_defaults -> serde_json::from_value
//! ```
//!
//! See ADR-0012 (JSON schema validation), ADR-0013 (No silent
//! fallbacks), and ADR-0038 (Shared JSON utilities crate).

#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![warn(clippy::all, clippy::pedantic)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

pub mod error;
pub mod loader;

pub use error::JsonError;
pub use loader::{
    fill_defaults, load_validated, load_validated_with_units, read_json, validate,
    validate_with_registry,
};

#[cfg(test)]
#[path = "loader_tests.rs"]
mod loader_tests;

#[cfg(test)]
#[path = "fill_defaults_array_tests.rs"]
mod fill_defaults_array_tests;
