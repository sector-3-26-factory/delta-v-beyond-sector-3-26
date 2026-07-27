// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Ship template types for deserializing ship configuration from JSON.
//!
//! These structs are used by `delta-v-ships`, `delta-v-ai`, and `delta-v-spawn`
//! to extract gameplay values from validated template JSON (ADR-0014, ADR-0039, ADR-0040).
//!
//! See ADR-0009 (Newtonian physics) and ADR-0010 (configuration system).

use serde::Deserialize;

use crate::ai::AiConfig;
use crate::ai::AiConfigJson;
use crate::collision::{CollisionShapeData, CollisionShapeJson};
use crate::physics::PhysicalQuantityJson;
use crate::propulsion::ShipPropulsionTemplate;
use crate::spatial::Vec3Json;
use crate::spatial::{BoundingBox, BoundingBoxJson, Vec3};

// ============================================================================
// Camera Types
// ============================================================================

/// Camera definition JSON type with position, target, and availability.
///
/// Used for deserializing camera definitions from ship templates.
/// Per ADR-0008, positions are in metres.
#[derive(Debug, Clone, Deserialize)]
pub struct CameraDefinitionJson {
    /// Camera position relative to ship center (metres).
    pub position: Vec3Json,
    /// Point the camera looks at, relative to ship center (metres).
    /// Direction = normalize(target - position).
    pub target: Vec3Json,
    /// If true, this camera is physically present and accessible.
    /// If false, the position/target are computed but not available to the player.
    pub available: bool,
}

/// Camera definition runtime type with SI units.
///
/// Converted from [`CameraDefinitionJson`] with positions as `Vec3` (Bevy type).
#[derive(Debug, Clone)]
pub struct CameraDefinition {
    /// Camera position relative to ship center (metres).
    pub position: Vec3,
    /// Point the camera looks at, relative to ship center (metres).
    pub target: Vec3,
    /// If true, this camera is physically present and accessible.
    pub available: bool,
}

impl From<CameraDefinitionJson> for CameraDefinition {
    fn from(json: CameraDefinitionJson) -> Self {
        Self {
            position: json.position.into(),
            target: json.target.into(),
            available: json.available,
        }
    }
}

/// Ship camera definitions from the template.
///
/// All 8 cameras are required in the schema. Each has a position, target, and availability flag.
#[derive(Debug, Clone, Deserialize)]
pub struct ShipCamerasTemplateJson {
    /// Cockpit camera (inside the cockpit, typically front-upper-center).
    pub cockpit: CameraDefinitionJson,
    /// Drone camera (behind and above the ship).
    pub drone: CameraDefinitionJson,
    /// Rear view camera (behind at cockpit height).
    pub rear: CameraDefinitionJson,
    /// Front/nose camera (forward view).
    pub front: CameraDefinitionJson,
    /// Left side view camera.
    pub left: CameraDefinitionJson,
    /// Right side view camera.
    pub right: CameraDefinitionJson,
    /// Top-down view camera.
    pub top: CameraDefinitionJson,
    /// Bottom-up view camera.
    pub bottom: CameraDefinitionJson,
}

/// Ship camera definitions runtime type.
///
/// Converted from [`ShipCamerasTemplateJson`] with positions as `Vec3` (Bevy type).
#[derive(Debug, Clone)]
pub struct ShipCamerasTemplate {
    /// Cockpit camera (inside the cockpit, typically front-upper-center).
    pub cockpit: CameraDefinition,
    /// Drone camera (behind and above the ship).
    pub drone: CameraDefinition,
    /// Rear view camera (behind at cockpit height).
    pub rear: CameraDefinition,
    /// Front/nose camera (forward view).
    pub front: CameraDefinition,
    /// Left side view camera.
    pub left: CameraDefinition,
    /// Right side view camera.
    pub right: CameraDefinition,
    /// Top-down view camera.
    pub top: CameraDefinition,
    /// Bottom-up view camera.
    pub bottom: CameraDefinition,
}

impl From<ShipCamerasTemplateJson> for ShipCamerasTemplate {
    fn from(json: ShipCamerasTemplateJson) -> Self {
        Self {
            cockpit: json.cockpit.into(),
            drone: json.drone.into(),
            rear: json.rear.into(),
            front: json.front.into(),
            left: json.left.into(),
            right: json.right.into(),
            top: json.top.into(),
            bottom: json.bottom.into(),
        }
    }
}

// ============================================================================
// Cockpit Types
// ============================================================================

/// Cockpit overlay definition with stations and gauge slots.
#[derive(Debug, Clone, Deserialize)]
pub struct CockpitDefinition {
    /// List of cockpit stations.
    pub stations: Vec<CockpitStation>,
}

/// A cockpit station with its texture and gauge slots.
///
/// The `slots` field defaults to an empty array via the schema (cockpit.schema.json).
/// Per ADR-0039, defaults are defined in schema only — no `#[serde(default)]`.
#[derive(Debug, Clone, Deserialize)]
pub struct CockpitStation {
    /// Station identifier.
    pub id: String,
    /// PNG file path relative to the template directory.
    pub texture: String,
    /// Gauge slot definitions. Defaults to `[]` via schema.
    pub slots: Vec<GaugeSlot>,
}

/// A gauge slot defining position/shape and the default gauge type.
#[derive(Debug, Clone, Deserialize)]
pub struct GaugeSlot {
    /// Shape defining the slot position and size.
    pub shape: GaugeShape,
    /// Gauge type name (e.g., "altitude", "velocity").
    pub default_gauge: String,
}

/// Shape for a gauge slot: either rectangle or circle.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum GaugeShape {
    /// Rectangle shape with pixel coordinates.
    Rectangle {
        /// Left edge (pixels).
        x1: f32,
        /// Top edge (pixels).
        y1: f32,
        /// Right edge (pixels).
        x2: f32,
        /// Bottom edge (pixels).
        y2: f32,
    },
    /// Circle shape with center and radius.
    Circle {
        /// Center X (pixels).
        cx: f32,
        /// Center Y (pixels).
        cy: f32,
        /// Radius (pixels).
        r: f32,
    },
}

// ============================================================================
// Ship Template Types
// ============================================================================

/// Base deserialized ship template JSON.
///
/// Contains common properties for all ship types (mass, inertia, propulsion).
/// This struct is produced by deserializing the validated + default-filled
/// `serde_json::Value` from the template file. Per ADR-0040, the schema is
/// the only source of defaults — no `#[serde(default)]` or `impl Default`.
#[derive(Debug, Deserialize)]
pub struct ShipTemplateJson {
    /// Ship mass in kilograms.
    pub mass: PhysicalQuantityJson,
    /// Dimensionless inertia multiplier (default 1.0 from schema).
    pub inertia_scale: f32,
    /// Propulsion system configuration.
    pub propulsion: ShipPropulsionTemplate,
    /// Axis-aligned bounding box in ship-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBoxJson,
    /// Collision shape for the ship.
    /// Used for collision detection with asteroids.
    pub collision_shape: CollisionShapeJson,
    /// Weapon names. Optional; ships may have no weapons (default [] from schema).
    pub weapons: Vec<String>,
    /// Ship health in hit points (default 100.0 from schema).
    /// Used for damage model (M4).
    pub health: PhysicalQuantityJson,
    /// Maximum number of weapons this ship can carry (default 2 from schema).
    pub max_weapons_count: usize,
    /// Maximum number of main thrusters that can be selected (default 2 from schema).
    pub max_propulsions_count: usize,
}

/// Runtime ship template with SI units.
///
/// This is the converted version of [`ShipTemplateJson`] with all
/// physical quantities converted to SI base units (kilograms, hit points).
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct ShipTemplate {
    /// Ship mass in kilograms (SI base unit).
    pub mass: f32,
    /// Dimensionless inertia multiplier.
    pub inertia_scale: f32,
    /// Propulsion system configuration.
    pub propulsion: ShipPropulsionTemplate,
    /// Axis-aligned bounding box in ship-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBox,
    /// Collision shape data for the ship.
    /// Used for collision detection with asteroids.
    pub collision_shape: CollisionShapeData,
    /// Weapon names.
    pub weapons: Vec<String>,
    /// Ship health in hit points (SI base unit).
    pub health: f32,
    /// Maximum number of weapons this ship can carry.
    pub max_weapons_count: usize,
    /// Maximum number of main thrusters that can be selected.
    pub max_propulsions_count: usize,
}

impl From<ShipTemplateJson> for ShipTemplate {
    fn from(json: ShipTemplateJson) -> Self {
        Self {
            mass: json.mass.to_kilograms(),
            inertia_scale: json.inertia_scale,
            propulsion: json.propulsion,
            bounding_box: json.bounding_box.into(),
            collision_shape: json.collision_shape.into(),
            weapons: json.weapons,
            health: json.health.to_hit_points(),
            max_weapons_count: json.max_weapons_count,
            max_propulsions_count: json.max_propulsions_count,
        }
    }
}

/// Deserialized player-controlled ship template JSON.
///
/// Extends [`ShipTemplateJson`] with camera definitions.
/// Used only for `entity_type` `player_controlled_ship`.
/// The template JSON is merged from the `player_controlled_ship` template
/// and the referenced ship template at load time.
#[derive(Debug, Deserialize)]
pub struct PlayerShipTemplateJson {
    /// Ship mass in kilograms (from merged ship template).
    pub mass: PhysicalQuantityJson,
    /// Dimensionless inertia multiplier (from merged ship template).
    pub inertia_scale: f32,
    /// Propulsion system configuration (from merged ship template).
    pub propulsion: ShipPropulsionTemplate,
    /// Camera definitions (cockpit required, others optional).
    pub cameras: ShipCamerasTemplateJson,
    /// Axis-aligned bounding box in ship-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBoxJson,
    /// Collision shape for the ship.
    /// Used for collision detection with asteroids.
    pub collision_shape: CollisionShapeJson,
    /// Weapon names. Optional; ships may have no weapons (default [] from schema).
    pub weapons: Vec<String>,
    /// Ship health in hit points (default 100.0 from schema).
    /// Used for damage model (M4).
    pub health: PhysicalQuantityJson,
    /// Cockpit overlay definition with stations and gauge slots.
    pub cockpit: CockpitDefinition,
    /// Maximum number of weapons this ship can carry (default 2 from schema).
    pub max_weapons_count: usize,
    /// Maximum number of main thrusters that can be selected (default 2 from schema).
    pub max_propulsions_count: usize,
}

/// Runtime player-controlled ship template with SI units.
///
/// This is the converted version of [`PlayerShipTemplateJson`] with all
/// physical quantities converted to SI base units.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct PlayerShipTemplate {
    /// Ship mass in kilograms (SI base unit).
    pub mass: f32,
    /// Dimensionless inertia multiplier.
    pub inertia_scale: f32,
    /// Propulsion system configuration.
    pub propulsion: ShipPropulsionTemplate,
    /// Camera definitions.
    pub cameras: ShipCamerasTemplate,
    /// Axis-aligned bounding box in ship-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBox,
    /// Collision shape data for the ship.
    /// Used for collision detection with asteroids.
    pub collision_shape: CollisionShapeData,
    /// Weapon names.
    pub weapons: Vec<String>,
    /// Ship health in hit points (SI base unit).
    pub health: f32,
    /// Cockpit overlay definition with stations and gauge slots.
    pub cockpit: CockpitDefinition,
    /// Maximum number of weapons this ship can carry.
    pub max_weapons_count: usize,
    /// Maximum number of main thrusters that can be selected.
    pub max_propulsions_count: usize,
}

impl From<PlayerShipTemplateJson> for PlayerShipTemplate {
    fn from(json: PlayerShipTemplateJson) -> Self {
        Self {
            mass: json.mass.to_kilograms(),
            inertia_scale: json.inertia_scale,
            propulsion: json.propulsion,
            cameras: json.cameras.into(),
            bounding_box: json.bounding_box.into(),
            collision_shape: json.collision_shape.into(),
            weapons: json.weapons,
            health: json.health.to_hit_points(),
            cockpit: json.cockpit,
            max_weapons_count: json.max_weapons_count,
            max_propulsions_count: json.max_propulsions_count,
        }
    }
}

/// Deserialized AI-controlled ship template JSON.
///
/// Contains all fields needed to spawn an AI-controlled NPC ship: physical
/// properties (mass, inertia, health), collision shape, bounding box, weapons,
/// propulsion, and AI behavioral configuration. This struct is produced by
/// deserializing the validated + default-filled `serde_json::Value` from the
/// template file. Per ADR-0040, the schema is the only source of defaults —
/// no `#[serde(default)]` or `impl Default`.
#[derive(Debug, Deserialize)]
pub struct AiShipTemplateJson {
    /// Ship mass in kilograms.
    pub mass: PhysicalQuantityJson,
    /// Dimensionless inertia multiplier (default 1.0 from schema).
    pub inertia_scale: f32,
    /// Ship health in hit points (default 100.0 from schema).
    pub health: PhysicalQuantityJson,
    /// Collision shape for the ship.
    pub collision_shape: CollisionShapeJson,
    /// Axis-aligned bounding box in ship-local coordinates (metres).
    pub bounding_box: BoundingBoxJson,
    /// Weapon names. Defaults to `[]` via schema.
    pub weapons: Vec<String>,
    /// Propulsion system configuration.
    pub propulsion: ShipPropulsionTemplate,
    /// AI behavioral parameters (aggro, attack, leash, patrol ranges).
    pub ai: AiConfigJson,
    /// Maximum number of weapons this ship can carry (default 2 from schema).
    pub max_weapons_count: usize,
    /// Maximum number of main thrusters that can be selected (default 2 from schema).
    pub max_propulsions_count: usize,
}

/// Runtime AI-controlled ship template with SI units.
///
/// This is the converted version of [`AiShipTemplateJson`] with all
/// physical quantities converted to SI base units.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct AiShipTemplate {
    /// Ship mass in kilograms (SI base unit).
    pub mass: f32,
    /// Dimensionless inertia multiplier.
    pub inertia_scale: f32,
    /// Ship health in hit points (SI base unit).
    pub health: f32,
    /// Collision shape data for the ship.
    /// Used for collision detection with asteroids.
    pub collision_shape: CollisionShapeData,
    /// Axis-aligned bounding box in ship-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBox,
    /// Weapon names.
    pub weapons: Vec<String>,
    /// Propulsion system configuration.
    pub propulsion: ShipPropulsionTemplate,
    /// AI behavioral parameters (aggro, attack, leash, patrol ranges in metres).
    pub ai: AiConfig,
    /// Maximum number of weapons this ship can carry.
    pub max_weapons_count: usize,
    /// Maximum number of main thrusters that can be selected.
    pub max_propulsions_count: usize,
}

impl From<AiShipTemplateJson> for AiShipTemplate {
    fn from(json: AiShipTemplateJson) -> Self {
        Self {
            mass: json.mass.to_kilograms(),
            inertia_scale: json.inertia_scale,
            health: json.health.to_hit_points(),
            collision_shape: json.collision_shape.into(),
            bounding_box: json.bounding_box.into(),
            weapons: json.weapons,
            propulsion: json.propulsion,
            ai: json.ai.into(),
            max_weapons_count: json.max_weapons_count,
            max_propulsions_count: json.max_propulsions_count,
        }
    }
}

/// Deserialized static ship template JSON.
///
/// Contains all fields needed to spawn a static NPC ship: physical
/// properties (mass, inertia, health), collision shape, bounding box, weapons,
/// propulsion. This struct is produced by deserializing the validated +
/// default-filled `serde_json::Value` from the template file.
/// Per ADR-0040, the schema is the only source of defaults —
/// no `#[serde(default)]` or `impl Default`.
#[derive(Debug, Deserialize)]
pub struct StaticShipTemplateJson {
    /// Ship mass in kilograms.
    pub mass: PhysicalQuantityJson,
    /// Dimensionless inertia multiplier (default 1.0 from schema).
    pub inertia_scale: f32,
    /// Ship health in hit points (default 100.0 from schema).
    pub health: PhysicalQuantityJson,
    /// Collision shape for the ship.
    pub collision_shape: CollisionShapeJson,
    /// Axis-aligned bounding box in ship-local coordinates (metres).
    pub bounding_box: BoundingBoxJson,
    /// Weapon names. Defaults to `[]` via schema.
    pub weapons: Vec<String>,
    /// Propulsion system configuration.
    pub propulsion: ShipPropulsionTemplate,
    /// Maximum number of weapons this ship can carry (default 2 from schema).
    pub max_weapons_count: usize,
    /// Maximum number of main thrusters that can be selected (default 2 from schema).
    pub max_propulsions_count: usize,
}

/// Runtime static ship template with SI units.
///
/// This is the converted version of [`StaticShipTemplateJson`] with all
/// physical quantities converted to SI base units.
/// Per ADR-0008, conversion happens at load time.
#[derive(Debug, Clone)]
pub struct StaticShipTemplate {
    /// Ship mass in kilograms (SI base unit).
    pub mass: f32,
    /// Dimensionless inertia multiplier.
    pub inertia_scale: f32,
    /// Ship health in hit points (SI base unit).
    pub health: f32,
    /// Collision shape data for the ship.
    /// Used for collision detection with asteroids.
    pub collision_shape: CollisionShapeData,
    /// Axis-aligned bounding box in ship-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBox,
    /// Weapon names.
    pub weapons: Vec<String>,
    /// Propulsion system configuration.
    pub propulsion: ShipPropulsionTemplate,
    /// Maximum number of weapons this ship can carry.
    pub max_weapons_count: usize,
    /// Maximum number of main thrusters that can be selected.
    pub max_propulsions_count: usize,
}

impl From<StaticShipTemplateJson> for StaticShipTemplate {
    fn from(json: StaticShipTemplateJson) -> Self {
        Self {
            mass: json.mass.to_kilograms(),
            inertia_scale: json.inertia_scale,
            health: json.health.to_hit_points(),
            collision_shape: json.collision_shape.into(),
            bounding_box: json.bounding_box.into(),
            weapons: json.weapons,
            propulsion: json.propulsion,
            max_weapons_count: json.max_weapons_count,
            max_propulsions_count: json.max_propulsions_count,
        }
    }
}

// ============================================================================
// ShipTemplateBase Trait
// ============================================================================

/// Trait for accessing common ship template fields.
///
/// Implemented for all ship template types to provide a unified interface
/// for accessing physical properties, collision shapes, and weapon configurations.
/// Per ADR-0047, this trait is defined in `delta-v-types` where the runtime
/// types live, avoiding orphan rule issues.
pub trait ShipTemplateBase {
    /// Returns the ship's mass in kilograms (SI base unit).
    fn mass(&self) -> f32;
    /// Returns the dimensionless inertia multiplier.
    fn inertia_scale(&self) -> f32;
    /// Returns the axis-aligned bounding box in ship-local coordinates (metres).
    fn bounding_box(&self) -> &BoundingBox;
    /// Returns the collision shape data for the ship.
    fn collision_shape(&self) -> &CollisionShapeData;
    /// Returns the ship's health in hit points (SI base unit).
    fn health(&self) -> f32;
    /// Returns the weapon names.
    fn weapons(&self) -> &[String];
    /// Returns the entity type string (e.g., "`player_controlled_ship`", "`ship`", "`ai_controlled_ship`").
    fn entity_type(&self) -> &'static str;
    /// Returns the array of main thruster names.
    fn main_thruster_names(&self) -> &[String];
    /// Returns the name of the maneuvering thruster.
    fn maneuvering_thruster_name(&self) -> &str;
    /// Returns the maximum number of weapons this ship can carry.
    fn max_weapons_count(&self) -> usize;
    /// Returns the maximum number of main thrusters that can be selected.
    fn max_propulsions_count(&self) -> usize;
}

impl ShipTemplateBase for ShipTemplate {
    fn mass(&self) -> f32 {
        self.mass
    }
    fn inertia_scale(&self) -> f32 {
        self.inertia_scale
    }
    fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }
    fn collision_shape(&self) -> &CollisionShapeData {
        &self.collision_shape
    }
    fn health(&self) -> f32 {
        self.health
    }
    fn weapons(&self) -> &[String] {
        &self.weapons
    }
    fn entity_type(&self) -> &'static str {
        "ship"
    }
    fn main_thruster_names(&self) -> &[String] {
        &self.propulsion.main_thruster_names
    }
    fn maneuvering_thruster_name(&self) -> &str {
        &self.propulsion.maneuvering_thruster
    }
    fn max_weapons_count(&self) -> usize {
        self.max_weapons_count
    }
    fn max_propulsions_count(&self) -> usize {
        self.max_propulsions_count
    }
}

impl ShipTemplateBase for PlayerShipTemplate {
    fn mass(&self) -> f32 {
        self.mass
    }
    fn inertia_scale(&self) -> f32 {
        self.inertia_scale
    }
    fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }
    fn collision_shape(&self) -> &CollisionShapeData {
        &self.collision_shape
    }
    fn health(&self) -> f32 {
        self.health
    }
    fn weapons(&self) -> &[String] {
        &self.weapons
    }
    fn entity_type(&self) -> &'static str {
        "player_controlled_ship"
    }
    fn main_thruster_names(&self) -> &[String] {
        &self.propulsion.main_thruster_names
    }
    fn maneuvering_thruster_name(&self) -> &str {
        &self.propulsion.maneuvering_thruster
    }
    fn max_weapons_count(&self) -> usize {
        self.max_weapons_count
    }
    fn max_propulsions_count(&self) -> usize {
        self.max_propulsions_count
    }
}

impl ShipTemplateBase for AiShipTemplate {
    fn mass(&self) -> f32 {
        self.mass
    }
    fn inertia_scale(&self) -> f32 {
        self.inertia_scale
    }
    fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }
    fn collision_shape(&self) -> &CollisionShapeData {
        &self.collision_shape
    }
    fn health(&self) -> f32 {
        self.health
    }
    fn weapons(&self) -> &[String] {
        &self.weapons
    }
    fn entity_type(&self) -> &'static str {
        "ai_controlled_ship"
    }
    fn main_thruster_names(&self) -> &[String] {
        &self.propulsion.main_thruster_names
    }
    fn maneuvering_thruster_name(&self) -> &str {
        &self.propulsion.maneuvering_thruster
    }
    fn max_weapons_count(&self) -> usize {
        self.max_weapons_count
    }
    fn max_propulsions_count(&self) -> usize {
        self.max_propulsions_count
    }
}

impl ShipTemplateBase for StaticShipTemplate {
    fn mass(&self) -> f32 {
        self.mass
    }
    fn inertia_scale(&self) -> f32 {
        self.inertia_scale
    }
    fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }
    fn collision_shape(&self) -> &CollisionShapeData {
        &self.collision_shape
    }
    fn health(&self) -> f32 {
        self.health
    }
    fn weapons(&self) -> &[String] {
        &self.weapons
    }
    fn entity_type(&self) -> &'static str {
        "ship"
    }
    fn main_thruster_names(&self) -> &[String] {
        &self.propulsion.main_thruster_names
    }
    fn maneuvering_thruster_name(&self) -> &str {
        &self.propulsion.maneuvering_thruster
    }
    fn max_weapons_count(&self) -> usize {
        self.max_weapons_count
    }
    fn max_propulsions_count(&self) -> usize {
        self.max_propulsions_count
    }
}
