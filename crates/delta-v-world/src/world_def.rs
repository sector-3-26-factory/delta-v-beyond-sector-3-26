// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Deserialisation types for the world definition JSON format.
//!
//! These types mirror the shape of `assets/json/schema/world.schema.json`.
//! They have no `Default` impl: every instance must come from a validated
//! JSON load (ADR-0013, ADR-0014).
//!
//! See ADR-0019 (Asset pipeline) and ADR-0020 (Save and load format).

use serde::Deserialize;

/// Top-level world definition loaded from `*.world.json`.
#[derive(Debug, Deserialize)]
pub struct WorldDef {
    /// Schema version. Always 1 at this milestone.
    pub format_version: u32,
    /// Human-readable sector name.
    pub name: String,
    /// Spawn parameters for the player ship.
    pub player_ship: ShipSpawn,
}

/// Spawn parameters for the player-controlled ship.
#[derive(Debug, Deserialize)]
pub struct ShipSpawn {
    /// Initial world position in metres.
    pub position: Vec3Json,
    /// Initial facing direction (unit vector).
    ///
    /// No `#[serde(default)]` here: the `delta-v-json` fill-defaults pass
    /// inserts `{x:0,y:0,z:-1}` from the schema before deserialisation.
    /// If that pass is skipped or broken, deserialisation fails — the
    /// correct hard failure (ADR-0013).
    pub facing: FacingJson,
}

/// A 3-component position in metres.
#[derive(Debug, Deserialize)]
pub struct Vec3Json {
    /// X coordinate in metres.
    pub x: f32,
    /// Y coordinate in metres.
    pub y: f32,
    /// Z coordinate in metres.
    pub z: f32,
}

/// A 3-component direction (unit vector).
#[derive(Debug, Deserialize)]
pub struct FacingJson {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
}
