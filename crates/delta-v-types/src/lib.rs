// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Shared domain types for the delta-v game.
//!
//! This crate contains plain data types and serde deserialization structs used
//! across multiple crates. It MUST NOT contain systems, plugins, or Bevy
//! `Component` derives.
//!
//! # Module Structure
//!
//! - `spatial` — Spatial types: `BoundingBoxJson`, `Vec3Json`, `QuatJson`
//! - `physics` — Physics types: `PhysicalQuantityJson`
//! - `collision` — Collision types: `CollisionShapeJson`, `CollisionShapeData`, `CollisionShapeType`, `CollisionLayers`
//! - `ids` — Identifier types: `EntityId`, `TemplatePath`
//! - `weapons` — Weapon types: `WeaponTemplateJson`
//! - `ai` — AI types: `AiConfigJson`, `AiTaskJson`
//! - `logical_action` — Input actions: `LogicalAction`

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

pub mod ai;
pub mod collision;
pub mod i18n;
pub mod ids;
pub mod logical_action;
pub mod physics;
pub mod spatial;
pub mod weapons;

// Re-exports for convenience
pub use ai::{AiConfigJson, AiTaskJson};
pub use collision::layers::PROJECTILE;
pub use collision::{CollisionLayers, CollisionShapeData, CollisionShapeJson, CollisionShapeType};
pub use i18n::{I18n, KeybindingsMenuTranslations, MenuTranslations, UiTranslations};
pub use ids::{EntityId, TemplatePath};
pub use logical_action::LogicalAction;
pub use physics::PhysicalQuantityJson;
pub use spatial::{BoundingBoxJson, QuatJson, Vec3Json};
pub use weapons::WeaponTemplateJson;

#[cfg(test)]
#[path = "spatial_tests.rs"]
mod spatial_tests;

#[cfg(test)]
#[path = "collision_type_tests.rs"]
mod collision_type_tests;

#[cfg(test)]
#[path = "logical_action_tests.rs"]
mod logical_action_tests;
