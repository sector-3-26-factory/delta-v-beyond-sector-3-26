// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Deserialisation types for the world definition JSON format.
//!
//! These types mirror the shape of `assets/json/schema/world.schema.json`.
//! They have no `Default` impl: every instance must come from a validated
//! JSON load (ADR-0013, ADR-0014).
//!
//! See ADR-0019 (Asset pipeline) and ADR-0020 (Save and load format).

use serde::Deserialize;

use crate::{
    AiTask, AiTaskJson, EntityTemplate, physics::PhysicalQuantityJson, spatial::Quat,
    spatial::QuatJson, spatial::Vec3, spatial::Vec3Json,
};

/// Top-level world definition loaded from `*.world.json` (JSON deserialization type).
///
/// Per ADR-0038 (entity template system), the world definition contains
/// an array of entities to spawn. Each entity references a template file
/// and specifies instance data (position, rotation, scale).
#[derive(Debug, Deserialize)]
pub struct WorldDefJson {
    /// Schema version. Always 1 at this milestone.
    pub format_version: u32,
    /// Human-readable sector name.
    pub name: String,
    /// Array of entities to spawn (per ADR-0038).
    /// Each entry specifies a template and instance data.
    /// The schema provides `"default": []`, so this field is always present
    /// after the delta-v-json load pipeline (ADR-0039, ADR-0040).
    pub entities: Vec<EntitySpawnJson>,
}

/// Template-based entity spawn descriptor (JSON deserialization type).
///
/// Per ADR-0038, each entity in the world is described by:
/// - A reference to a template file (e.g., `templates/ships/player_ship/template.json`)
/// - Instance data (position, rotation, scale)
/// - A unique identifier for entity referencing (UI panels, save/load, etc.)
///
/// The entity type is derived from the template's `entity_type` field at load time.
#[derive(Debug, Deserialize)]
pub struct EntitySpawnJson {
    /// Short path to the template (e.g., `ships/debug-ship-cube`). Resolved internally to `templates/<path>/<entity_type>.json`.
    #[serde(alias = "template")]
    pub template_short: String,
    /// Unique identifier for this entity instance.
    /// Used to reference the entity throughout the game (UI panels, save/load, networking, etc.).
    /// Required; missing `id` is a hard error (ADR-0013).
    pub id: String,
    /// Spawn position in world coordinates (metres).
    pub position: Vec3Json,
    /// Rotation as a unit quaternion (x, y, z, w).
    /// Filled by schema defaults if not provided; never absent after loading.
    pub rotation: QuatJson,
    /// Scale factor (x, y, z).
    /// Filled by schema defaults if not provided; never absent after loading.
    pub scale: Vec3Json,
    /// If true, this entity is player-controlled. The loader will load
    /// `player_controlled_ship.json` and merge with the co-located `ship.json`.
    /// Only valid for ship templates. Default: false (from schema).
    /// Filled by schema defaults; never absent after loading (ADR-0039, ADR-0040).
    pub player_controlled: bool,
    /// Optional AI task assignment. If present, the entity is AI-driven
    /// and the loader will load `ai_controlled_ship.json` and merge with
    /// the co-located `ship.json`. The task determines the AI mission
    /// (e.g., "patrol"). Absent means static ship. Optionality is defined
    /// by the JSON schema (no default in schema = optional).
    pub ai_task: Option<AiTaskJson>,
    /// Optional mass override. If present, this value overrides the template's mass.
    /// Mass is NOT scaled with the scale factor - it is used as-is or overridden.
    /// Per ADR-0008, uses value+unit format. Units: `kg`, `t`, `M_earth`, `M_sun`.
    /// Converted to kilograms at load time.
    pub mass: Option<PhysicalQuantityJson>,
}

/// Runtime world definition with SI units.
///
/// This is the converted version of [`WorldDefJson`] with all
/// physical quantities converted to SI base units.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct WorldDef {
    /// Schema version. Always 1 at this milestone.
    pub format_version: u32,
    /// Human-readable sector name.
    pub name: String,
    /// Array of entities to spawn (per ADR-0038).
    /// Each entry specifies a template and instance data.
    pub entities: Vec<EntitySpawn>,
}

/// Runtime template-based entity spawn descriptor with SI units.
///
/// This is the converted version of [`EntitySpawnJson`] with all
/// physical quantities converted to SI base units.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct EntitySpawn {
    /// Short path to the template (e.g., `ships/debug-ship-cube`). Resolved internally to `templates/<path>/<entity_type>.json`.
    pub template_short: String,
    /// Unique identifier for this entity instance.
    /// Used to reference the entity throughout the game (UI panels, save/load, networking, etc.).
    pub id: String,
    /// Spawn position in world coordinates (metres).
    pub position: Vec3,
    /// Rotation as a unit quaternion (x, y, z, w).
    pub rotation: Quat,
    /// Scale factor (x, y, z).
    pub scale: Vec3,
    /// If true, this entity is player-controlled. The loader will load
    /// `player_controlled_ship.json` and merge with the co-located `ship.json`.
    /// Only valid for ship templates. Default: false (from schema).
    pub player_controlled: bool,
    /// Optional AI task assignment. If present, the entity is AI-driven
    /// and the loader will load `ai_controlled_ship.json` and merge with
    /// the co-located `ship.json`. The task determines the AI mission
    /// (e.g., "patrol"). Absent means static ship.
    pub ai_task: Option<AiTask>,
    /// Optional mass override. If present, this value overrides the template's mass.
    /// Mass is NOT scaled with the scale factor - it is used as-is or overridden.
    /// Units: kilograms (SI base unit).
    pub mass: Option<f32>,
    /// The loaded template (populated by loader).
    pub template: Option<EntityTemplate>,
}

impl From<WorldDefJson> for WorldDef {
    fn from(json: WorldDefJson) -> Self {
        Self {
            format_version: json.format_version,
            name: json.name,
            entities: json.entities.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<EntitySpawnJson> for EntitySpawn {
    fn from(json: EntitySpawnJson) -> Self {
        Self {
            template_short: json.template_short,
            id: json.id,
            position: json.position.into(),
            rotation: json.rotation.into(),
            scale: json.scale.into(),
            player_controlled: json.player_controlled,
            ai_task: json.ai_task.map(Into::into),
            mass: json.mass.map(|m| m.to_kilograms()),
            template: None, // Populated by loader
        }
    }
}
