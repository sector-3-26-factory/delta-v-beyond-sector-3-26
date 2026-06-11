// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Shared domain types for the delta-v game.
//!
//! This crate contains plain data types and serde deserialization structs used
//! across multiple crates. It MUST NOT contain systems, plugins, or Bevy
//! `Component` derives.
//!
//! # Module Structure
//!
//! - `spatial` — Spatial types: `BoundingBox`, `Vec3Json`, `QuatJson`
//! - `physics` — Physics types: `PhysicalQuantity`
//! - `collision` — Collision types: `CollisionShapeJson`
//! - `ids` — Identifier types: `EntityId`, `TemplatePath`

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(unreachable_pub)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::dbg_macro)]

pub mod collision;
pub mod ids;
pub mod physics;
pub mod spatial;

// Re-exports for convenience
pub use collision::{CollisionLayers, CollisionShapeJson};
pub use ids::{EntityId, TemplatePath};
pub use physics::PhysicalQuantity;
pub use spatial::{BoundingBox, QuatJson, Vec3Json};
