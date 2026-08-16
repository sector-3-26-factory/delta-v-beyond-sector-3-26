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
//! - `celestial` — Celestial body types: `SunTemplateJson`, `PlanetTemplateJson`
//! - `ai` — AI types: `AiConfigJson`, `AiTaskJson`
//! - `logical_action` — Input actions: `LogicalAction`
//! - `navigation` — Navigation types: `EntityType`, `EntityId`
//! - `main_thruster` — Main thruster types: `MainThrusterDefinitionJson`
//! - `maneuvering_thruster` — Maneuvering thruster types: `ManeuveringThrusterDefinitionJson`
//! - `propulsion` — Propulsion types: `ShipPropulsionTemplate`
//! - `entity_template` — Entity template enum: `EntityTemplate`

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
pub mod celestial;
pub mod collision;
pub mod config_types;
pub mod entity_template;
pub mod i18n;
pub mod ids;
pub mod logical_action;
pub mod main_thruster;
pub mod maneuvering_thruster;
pub mod navigation;
pub mod physics;
pub mod player_settings;
pub mod propulsion;
pub mod ship_templates;
pub mod spatial;
pub mod weapons;
pub mod world_def;

pub use ai::{AiConfig, AiConfigJson, AiTask, AiTaskJson};
pub use celestial::{
    AsteroidTemplate, AsteroidTemplateJson, LightColorJson, MoonTemplate, MoonTemplateJson,
    PlanetTemplate, PlanetTemplateJson, SunTemplate, SunTemplateJson,
};
pub use collision::layers::PROJECTILE;
pub use collision::{
    CollisionLayers, CollisionShapeData, CollisionShapeJson, CollisionShapeType,
    scale_collision_shape,
};
pub use config_types::{DebugConfigJson, DiagnosticsConfigJson, FlightAssistConfigJson};
pub use entity_template::EntityTemplate;
pub use i18n::{I18n, KeybindingsMenuTranslations, MenuTranslations, UiTranslations};
pub use ids::{EntityId, TemplatePath};
pub use logical_action::LogicalAction;
pub use main_thruster::{MainThrusterDefinition, MainThrusterDefinitionJson};
pub use maneuvering_thruster::{ManeuveringThrusterDefinition, ManeuveringThrusterDefinitionJson};
pub use navigation::{EntityType, WorldEntityId};
pub use physics::{PhysicalQuantityJson, RigidBodyData, resolve_mass};
pub use player_settings::PlayerSettings;
pub use propulsion::{PropulsionConfig, ShipPropulsionTemplate};
pub use ship_templates::{
    AiShipTemplate, AiShipTemplateJson, CameraDefinition, CameraDefinitionJson, CockpitDefinition,
    CockpitStation, GaugeShape, GaugeSlot, PlayerShipTemplate, PlayerShipTemplateJson,
    ShipCamerasTemplate, ShipCamerasTemplateJson, ShipTemplate, ShipTemplateBase, ShipTemplateJson,
    StaticShipTemplate, StaticShipTemplateJson,
};
pub use spatial::{
    BoundingBox, BoundingBoxJson, QuatJson, Vec3, Vec3Json, compute_debug_axis_length,
    scale_bounding_box,
};
pub use weapons::{
    ProjectileDefinition, ProjectileDefinitionJson, WeaponTemplate, WeaponTemplateJson,
};
pub use world_def::{EntitySpawn, EntitySpawnJson, WorldDef, WorldDefJson};

#[cfg(test)]
#[path = "spatial_tests.rs"]
mod spatial_tests;

#[cfg(test)]
#[path = "collision_type_tests.rs"]
mod collision_type_tests;

#[cfg(test)]
#[path = "physics_tests.rs"]
mod physics_tests;
