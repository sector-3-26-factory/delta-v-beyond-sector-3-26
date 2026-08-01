// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Spawning utilities for the delta-v game.
//!
//! This crate provides shared infrastructure for spawning entities from templates:
//! - Template field extraction
//! - Collision shape conversion
//! - Mesh attachment
//! - Scene lighting setup
//!
//! See ADR-0047 for the centralized spawning architecture.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(unreachable_pub)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::dbg_macro)]

pub mod collision;
pub mod lighting;
pub mod mesh_attachment;
pub mod ship_spawn;
pub mod template_extraction;

pub use collision::scale_collision_shape;
pub use mesh_attachment::{PendingMesh, attach_meshes};
pub use ship_spawn::{PendingShipMesh, build_physical_ship};
pub use template_extraction::{
    compute_debug_axis_length, extract_bounding_box, extract_collision_shape, extract_mass,
    resolve_mass, scale_bounding_box,
};

#[cfg(test)]
#[path = "collision_tests.rs"]
mod collision_tests;

#[cfg(test)]
#[path = "extraction_tests.rs"]
mod extraction_tests;
