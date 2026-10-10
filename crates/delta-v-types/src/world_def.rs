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
    /// Optional orbital parameters. If present, the entity will orbit the specified parent body.
    /// These parameters are world-specific and override any template defaults.
    pub orbital_parameters: Option<OrbitalParametersJson>,
    /// Optional asteroid belt parameters. Only present for asteroid belt entities.
    pub asteroid_belt_parameters: Option<AsteroidBeltParametersJson>,
    /// Optional asteroid field parameters. Only present for asteroid field entities.
    pub asteroid_field_parameters: Option<AsteroidFieldParametersJson>,
}

/// Orbital parameters for an entity (JSON deserialization type).
#[derive(Debug, Deserialize)]
pub struct OrbitalParametersJson {
    /// ID of the parent body this entity orbits. If absent, the entity does not orbit.
    pub orbital_parent: Option<String>,
    /// Orbital distance (semi-major axis) in metres.
    pub orbital_distance: Option<PhysicalQuantityJson>,
    /// Orbital period in seconds.
    pub orbital_period: Option<PhysicalQuantityJson>,
    /// Orbital eccentricity (0 = circular, 0.1-0.9 = increasingly elliptical). Default: 0 (circular).
    pub orbital_eccentricity: f32,
    /// Orbital inclination in degrees.
    pub orbital_inclination: PhysicalQuantityJson,
    /// Initial orbital angle in degrees.
    pub initial_orbital_angle: PhysicalQuantityJson,
    /// Rotation period in hours. None means no rotation.
    pub rotation_period: Option<PhysicalQuantityJson>,
    /// Axial tilt (obliquity) in degrees.
    pub axial_tilt: PhysicalQuantityJson,
}

/// JSON schema type for asteroid belt parameters (world-specific).
#[derive(Debug, Deserialize, Clone)]
pub struct AsteroidBeltParametersJson {
    /// Radial extent of a belt in metres (how far the ring reaches inward and outward from the orbital distance).
    pub radial_extent: PhysicalQuantityJson,
    /// Thickness of a belt in metres (how far it reaches above and below the orbital plane).
    pub thickness: PhysicalQuantityJson,
    /// Mean spacing between asteroids in metres (density of the fill).
    pub mean_spacing: PhysicalQuantityJson,
    /// Radii where nothing is placed (e.g., Kirkwood gaps).
    pub excluded_radii: Option<Vec<ExcludedRadiusJson>>,
    /// Seed for deterministic procedural generation.
    pub seed: u64,
    /// Cache granularity for belt sectors in metres (engine detail, not content).
    pub sector_size: PhysicalQuantityJson,
    /// Which asteroid sizes are placed and how often.
    pub size_distribution: Vec<SizeDistributionEntryJson>,
    /// Material density in kg/m³ for generated asteroids.
    pub density: PhysicalQuantityJson,
}

/// JSON schema type for an excluded radius (Kirkwood gap).
#[derive(Debug, Deserialize, Clone)]
pub struct ExcludedRadiusJson {
    /// Orbital distance of the gap centre in metres.
    pub orbital_distance: PhysicalQuantityJson,
    /// Radius of the gap in metres.
    pub gap_radius: PhysicalQuantityJson,
}

/// JSON schema type for a size distribution entry.
#[derive(Debug, Deserialize, Clone)]
pub struct SizeDistributionEntryJson {
    /// Asteroid radius in metres.
    pub radius: PhysicalQuantityJson,
    /// Relative weight for this size.
    pub weight: f32,
}

/// JSON schema type for asteroid field parameters (world-specific).
#[derive(Debug, Deserialize, Clone)]
pub struct AsteroidFieldParametersJson {
    /// Extent of a field region in metres (how large the region is).
    pub extent: PhysicalQuantityJson,
    /// Shape of a field region (e.g., "box").
    pub shape: String,
    /// Mean spacing between asteroids in metres (density of the fill).
    pub mean_spacing: PhysicalQuantityJson,
    /// Seed for deterministic procedural generation.
    pub seed: u64,
    /// Cache granularity for field sectors in metres (engine detail, not content).
    pub sector_size: Option<PhysicalQuantityJson>,
    /// Which asteroid sizes are placed and how often.
    pub size_distribution: Vec<SizeDistributionEntryJson>,
    /// Material density in kg/m³ for generated asteroids.
    pub density: PhysicalQuantityJson,
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
    /// Optional orbital parameters in SI units.
    pub orbital_parameters: Option<OrbitalParameters>,
    /// Optional asteroid belt parameters in SI units.
    pub asteroid_belt_parameters: Option<AsteroidBeltParameters>,
    /// Optional asteroid field parameters in SI units.
    pub asteroid_field_parameters: Option<AsteroidFieldParameters>,
}

/// Runtime asteroid belt parameters with SI units.
#[derive(Debug, Clone)]
pub struct AsteroidBeltParameters {
    /// Radial extent in metres.
    pub radial_extent: f32,
    /// Thickness in metres.
    pub thickness: f32,
    /// Mean spacing in metres.
    pub mean_spacing: f32,
    /// Excluded radii (Kirkwood gaps).
    pub excluded_radii: Vec<ExcludedRadius>,
    /// Seed for deterministic generation.
    pub seed: u64,
    /// Sector size in metres.
    pub sector_size: f32,
    /// Size distribution.
    pub size_distribution: Vec<SizeDistributionEntry>,
    /// Material density in kg/m³.
    pub density: f32,
}

/// Runtime asteroid field parameters with SI units.
#[derive(Debug, Clone)]
pub struct AsteroidFieldParameters {
    /// Extent in metres.
    pub extent: f32,
    /// Shape (e.g., "box").
    pub shape: String,
    /// Mean spacing in metres.
    pub mean_spacing: f32,
    /// Seed for deterministic generation.
    pub seed: u64,
    /// Sector size in metres (optional).
    pub sector_size: Option<f32>,
    /// Size distribution.
    pub size_distribution: Vec<SizeDistributionEntry>,
    /// Material density in kg/m³.
    pub density: f32,
}

/// Runtime excluded radius (Kirkwood gap) with SI units.
#[derive(Debug, Clone)]
pub struct ExcludedRadius {
    /// Orbital distance of the gap centre in metres.
    pub orbital_distance: f32,
    /// Radius of the gap in metres.
    pub gap_radius: f32,
}

/// Runtime size distribution entry with SI units.
#[derive(Debug, Clone)]
pub struct SizeDistributionEntry {
    /// Asteroid radius in metres.
    pub radius: f32,
    /// Relative weight for this size.
    pub weight: f32,
}

/// Orbital parameters for an entity (runtime type with SI units).
#[derive(Debug, Clone)]
pub struct OrbitalParameters {
    /// ID of the parent body this entity orbits.
    pub orbital_parent: String,
    /// Orbital distance (semi-major axis) in metres.
    pub orbital_distance: f32,
    /// Orbital period in seconds.
    pub orbital_period: f32,
    /// Orbital eccentricity (0 = circular, 0.1-0.9 = increasingly elliptical).
    pub orbital_eccentricity: f32,
    /// Orbital inclination in radians.
    pub orbital_inclination: f32,
    /// Initial orbital angle in radians.
    pub initial_orbital_angle: f32,
    /// Rotation period in seconds. None means no rotation.
    pub rotation_period: Option<f32>,
    /// Axial tilt (obliquity) in radians.
    pub axial_tilt: f32,
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
            orbital_parameters: json.orbital_parameters.map(|op| OrbitalParameters {
                orbital_parent: op.orbital_parent.unwrap_or_default(),
                orbital_distance: op.orbital_distance.map_or(0.0, |d| d.to_meters()),
                orbital_period: op.orbital_period.map_or(0.0, |p| p.to_seconds()),
                orbital_eccentricity: op.orbital_eccentricity,
                orbital_inclination: op.orbital_inclination.to_radians(),
                initial_orbital_angle: op.initial_orbital_angle.to_radians(),
                rotation_period: op.rotation_period.map(|p| p.to_seconds()),
                axial_tilt: op.axial_tilt.to_radians(),
            }),
            asteroid_belt_parameters: json.asteroid_belt_parameters.map(|bp| {
                AsteroidBeltParameters {
                    radial_extent: bp.radial_extent.to_meters(),
                    thickness: bp.thickness.to_meters(),
                    mean_spacing: bp.mean_spacing.to_meters(),
                    excluded_radii: bp
                        .excluded_radii
                        .unwrap_or_default()
                        .into_iter()
                        .map(|er| ExcludedRadius {
                            orbital_distance: er.orbital_distance.to_meters(),
                            gap_radius: er.gap_radius.to_meters(),
                        })
                        .collect(),
                    seed: bp.seed,
                    sector_size: bp.sector_size.to_meters(),
                    size_distribution: bp
                        .size_distribution
                        .into_iter()
                        .map(|se| SizeDistributionEntry {
                            radius: se.radius.to_meters(),
                            weight: se.weight,
                        })
                        .collect(),
                    density: bp.density.to_kilograms_per_cubic_meter(),
                }
            }),
            asteroid_field_parameters: json.asteroid_field_parameters.map(|fp| {
                AsteroidFieldParameters {
                    extent: fp.extent.to_meters(),
                    shape: fp.shape,
                    mean_spacing: fp.mean_spacing.to_meters(),
                    seed: fp.seed,
                    sector_size: fp.sector_size.map(|s| s.to_meters()),
                    size_distribution: fp
                        .size_distribution
                        .into_iter()
                        .map(|se| SizeDistributionEntry {
                            radius: se.radius.to_meters(),
                            weight: se.weight,
                        })
                        .collect(),
                    density: fp.density.to_kilograms_per_cubic_meter(),
                }
            }),
        }
    }
}
