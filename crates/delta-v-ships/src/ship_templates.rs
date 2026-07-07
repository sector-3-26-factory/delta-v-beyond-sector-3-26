// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Ship template types for deserializing ship configuration from JSON.
//!
//! These structs are used by `delta-v-ships` to extract gameplay values from
//! validated template JSON (ADR-0014, ADR-0039, ADR-0040).
//!
//! See ADR-0009 (Newtonian physics) and ADR-0010 (configuration system).

use bevy::prelude::*;
use serde::Deserialize;

use delta_v_core::camera::ShipCamerasTemplate;
use delta_v_types::{BoundingBoxJson, PhysicalQuantityJson};

/// Base deserialized ship template JSON.
///
/// Contains common properties for all ship types (mass, inertia, propulsion).
/// This struct is produced by deserializing the validated + default-filled
/// `serde_json::Value` from the template file. Per ADR-0040, the schema is
/// the only source of defaults — no `#[serde(default)]` or `impl Default`.
#[derive(Debug, Deserialize)]
pub struct ShipTemplate {
    /// Ship mass in kilograms.
    pub mass: PhysicalQuantityJson,
    /// Dimensionless inertia multiplier (default 1.0 from schema).
    pub inertia_scale: f32,
    /// Propulsion system configuration.
    pub propulsion: ShipPropulsionTemplate,
}

/// Sound file paths for a ship.
#[derive(Debug, Deserialize, Clone, Resource)]
pub struct ShipSounds {
    /// Engine thrust sound (looped while thrusting).
    pub thrust: Option<String>,
    /// Hit/impact sound.
    pub hit: Option<String>,
}

/// Deserialized player-controlled ship template JSON.
///
/// Extends [`ShipTemplate`] with camera definitions.
/// Used only for `entity_type` `player_controlled_ship`.
/// The template JSON is merged from the `player_controlled_ship` template
/// and the referenced ship template at load time.
#[derive(Debug, Deserialize)]
pub struct PlayerShipTemplate {
    /// Ship mass in kilograms (from merged ship template).
    pub mass: PhysicalQuantityJson,
    /// Dimensionless inertia multiplier (from merged ship template).
    pub inertia_scale: f32,
    /// Propulsion system configuration (from merged ship template).
    pub propulsion: ShipPropulsionTemplate,
    /// Camera definitions (cockpit required, others optional).
    pub cameras: ShipCamerasTemplate,
    /// Axis-aligned bounding box in ship-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBoxJson,
    /// Collision shape for the ship.
    /// Used for collision detection with asteroids.
    pub collision_shape: delta_v_types::CollisionShapeJson,
    /// Weapon configurations. Optional; ships may have no weapons (default [] from schema).
    pub weapons: Vec<delta_v_types::WeaponTemplateJson>,
    /// Ship health in hit points (default 100.0 from schema).
    /// Used for damage model (M4).
    pub health: PhysicalQuantityJson,
    /// Cockpit overlay definition with stations and gauge slots.
    pub cockpit: CockpitDefinition,
    /// Sound file paths relative to assets/audio/.
    /// Defaults to empty (no sounds) via schema default.
    /// Only player-controlled ships play sounds.
    pub sounds: ShipSounds,
}

/// Deserialized non-player ship template JSON.
///
/// Contains all ship properties except camera definitions.
/// Used for `entity_type` `ship` (non-player ships).
#[derive(Debug, Deserialize)]
pub struct StaticShipTemplate {
    /// Ship mass in kilograms.
    pub mass: PhysicalQuantityJson,
    /// Dimensionless inertia multiplier.
    pub inertia_scale: f32,
    /// Propulsion system configuration.
    pub propulsion: ShipPropulsionTemplate,
    /// Axis-aligned bounding box in ship-local coordinates (metres).
    /// Used for debug axes and spatial calculations.
    pub bounding_box: BoundingBoxJson,
    /// Collision shape for the ship.
    /// Used for collision detection with asteroids.
    pub collision_shape: delta_v_types::CollisionShapeJson,
    /// Weapon configurations. Optional; ships may have no weapons (default [] from schema).
    pub weapons: Vec<delta_v_types::WeaponTemplateJson>,
    /// Ship health in hit points (default 100.0 from schema).
    /// Used for damage model (M4).
    pub health: PhysicalQuantityJson,
}

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

/// Propulsion configuration from the ship template.
#[derive(Debug, Deserialize)]
pub struct ShipPropulsionTemplate {
    /// Main thruster configurations. For M2, exactly one is active.
    #[serde(rename = "main_thrusters")]
    pub main_thrusters: Vec<MainThrusterTemplate>,
    /// Maneuver thruster (RCS) configuration.
    pub maneuvering_thruster: ManeuveringThrusterTemplate,
}

/// Main thruster configuration from the template.
#[derive(Debug, Deserialize)]
pub struct MainThrusterTemplate {
    /// Unique identifier for this thruster within the ship.
    pub id: String,
    /// Thruster type (e.g. "chemical", "ion"). Metadata for M2.
    #[serde(rename = "type")]
    pub thruster_type: String,
    /// Maximum forward thrust.
    pub max_forward_thrust: PhysicalQuantityJson,
    /// Maximum backward/reverse thrust.
    pub max_backward_thrust: PhysicalQuantityJson,
}

/// Maneuvering thruster (RCS) configuration from the template.
#[derive(Debug, Deserialize)]
pub struct ManeuveringThrusterTemplate {
    /// Maneuvering thruster type (e.g. "rcs", "vernier"). Metadata for M2.
    #[serde(rename = "type")]
    pub thruster_type: String,
    /// Maximum torque per rotation axis.
    pub max_torque: PhysicalQuantityJson,
    /// Maximum strafe thrust per lateral/vertical axis.
    pub max_strafe_thrust: PhysicalQuantityJson,
    /// Number of ticks for torque to ramp from 0% to 100% when a rotation key
    /// is first pressed. 0 = instant full torque (no ramp).
    pub rotation_ramp_ticks: u32,
}

/// Ship propulsion configuration read from the ship template JSON.
///
/// Contains the active thruster's force/torque values that the input → forces
/// pipeline uses each tick. Inserted as a resource at ship spawn time.
/// Per ADR-0014, all gameplay values come from JSON — this resource is
/// populated from the template, not from Rust constants.
// allow-default: Bevy requires Default on resources for init_resource.
// This is runtime state, not configuration.
#[derive(Resource, Default, Debug)]
pub struct ShipPropulsionConfig {
    /// Maximum forward thrust in Newtons (applied along -Z local axis).
    pub max_forward_thrust: f32,
    /// Maximum backward/reverse thrust in Newtons (applied along +Z local axis).
    pub max_backward_thrust: f32,
    /// Maximum torque in Newton-meters per rotation axis.
    pub max_torque: f32,
    /// Maximum strafe thrust in Newtons per lateral/vertical axis.
    pub max_strafe_thrust: f32,
    /// Index of the currently active main thruster (for M3+ thruster switching).
    pub active_main_thruster_index: usize,
    /// Number of ticks for torque to ramp from 0% to 100% when a rotation key
    /// is first pressed. 0 = instant full torque (no ramp).
    pub rotation_ramp_ticks: u32,
}

/// Accumulated thrust command for the current fixed tick.
///
/// Written by the input reader system, consumed by the thrust system,
/// and cleared each tick.
///
/// The force is applied in the ship's local frame:
/// - +X = right, +Y = up, -Z = forward (per ADR-0006).
// allow-default: Bevy requires Default on resources for init_resource.
// This is a per-tick command buffer, not configuration.
#[derive(Resource, Default, Debug)]
pub struct ThrustCommand {
    /// Linear thrust force in Newtons (local frame).
    pub force: Vec3,
}

/// Accumulated torque command for the current fixed tick.
///
/// Written by the input reader system, consumed by the torque system,
/// and cleared each tick.
///
/// The torque is applied around the ship's local axes:
/// - X = pitch, Y = yaw, Z = roll (per ADR-0006).
// allow-default: Bevy requires Default on resources for init_resource.
// This is a per-tick command buffer, not configuration.
#[derive(Resource, Default, Debug)]
pub struct TorqueCommand {
    /// Torque in Newton-meters (local frame).
    pub torque: Vec3,
}
