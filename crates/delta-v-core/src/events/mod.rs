// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Events emitted for entity spawning.
//!
//! [`SpawnEntity`] is defined here (in `delta-v-core`) so that all domain
//! crates can reference it without creating domain-to-domain dependencies.
//! The world plugin emits these events; domain plugins listen for them.
//!
//! See ADR-0005 (plugin architecture) and ADR-0038 (entity template system).

use bevy::prelude::*;
use delta_v_types::{EntityTemplate, world_def::OrbitalParameters};

/// Event emitted when an entity should be spawned from a template.
///
/// The world plugin emits these events during [`AppState::LoadingWorld`].
/// Domain-specific plugins (`ShipsPlugin`, `SunPlugin`, etc.) listen for this
/// event, filter by `template` variant, and spawn the entity with appropriate
/// components.
///
/// This decouples world loading from entity spawning, allowing each domain
/// to own its spawn logic (ADR-0005).
#[derive(Message, Debug, Clone)]
pub struct SpawnEntity {
    /// Unique identifier for this entity instance (from world definition).
    /// Used for debug filtering, save/load, networking, and player-facing UI.
    pub id: String,

    /// The loaded and validated template as a runtime type.
    /// Contains all static properties for the entity with SI units.
    /// The entity type is derived from the `EntityTemplate` variant.
    pub template: EntityTemplate,

    /// Path to the template file (e.g., `templates/ships/player_ship/template.json`).
    /// The mesh is always at `mesh.glb` in this template's directory.
    pub template_path: String,

    /// Spawn position in world coordinates (metres).
    pub position: Vec3,

    /// Optional rotation (unit quaternion). Defaults to identity.
    pub rotation: Quat,

    /// Optional scale. Defaults to 1.0 on all axes.
    pub scale: Vec3,

    /// Optional AI task assignment from the world definition.
    /// If `Some`, the entity is AI-driven and the task determines its mission.
    /// If `None`, the entity is static or player-controlled.
    pub ai_task: Option<String>,

    /// Optional mass override from the world definition.
    /// If `Some`, this value overrides the template's mass.
    /// If `None`, the template's mass is used.
    /// Mass is NOT scaled with the scale factor.
    pub mass: Option<f32>,

    /// Optional orbital parameters from the world definition.
    /// If `Some`, the entity will orbit the specified parent body.
    /// If `None`, the entity will not orbit (e.g., sun, free-floating body).
    pub orbital_parameters: Option<OrbitalParameters>,
}

impl SpawnEntity {
    /// Creates a new `SpawnEntity` event with default rotation and scale.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this entity instance.
    /// * `template` - Loaded template as runtime type.
    /// * `template_path` - Path to the template file.
    /// * `position` - Spawn position in metres.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(
        id: String,
        template: EntityTemplate,
        template_path: String,
        position: Vec3,
    ) -> Self {
        Self {
            id,
            template,
            template_path,
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            ai_task: None,
            mass: None,
            orbital_parameters: None,
        }
    }

    /// Sets the rotation for this spawn event.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the scale for this spawn event.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the AI task for this spawn event.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_ai_task(mut self, ai_task: String) -> Self {
        self.ai_task = Some(ai_task);
        self
    }

    /// Sets the mass override for this spawn event.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = Some(mass);
        self
    }

    /// Sets the orbital parameters for this spawn event.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_orbital_parameters(mut self, orbital_parameters: OrbitalParameters) -> Self {
        self.orbital_parameters = Some(orbital_parameters);
        self
    }

    /// Derives the mesh path from the template path.
    ///
    /// The mesh is always at `mesh.glb` in the template's directory.
    /// For example, `templates/ships/space-fighter/template.json` -> `templates/ships/space-fighter/mesh.glb`.
    #[must_use]
    pub fn mesh_path(&self) -> String {
        use std::path::Path;
        Path::new(&self.template_path).parent().map_or_else(
            || self.template_path.clone(),
            |p| p.join("mesh.glb").to_string_lossy().into_owned(),
        )
    }
}

/// Event emitted when the player triggers a weapon.
///
/// Emitted by the input system; consumed by the weapons plugin to
/// spawn projectiles. The source entity must have a [`Weapon`] component.
#[derive(Message, Debug)]
pub struct FireWeapon {
    /// The entity that is firing (e.g., the player ship).
    pub source: Entity,
    /// The weapon slot/index being fired.
    pub weapon_index: u32,
}

/// Event emitted when a projectile hits another entity.
///
/// Emitted by the weapons plugin when a projectile collision is detected.
/// Contains the projectile, target, damage, hit point, and pre-resolved hit sound path.
#[derive(Message, Debug)]
pub struct ProjectileHit {
    /// The projectile entity.
    pub projectile: Entity,
    /// The entity that was hit.
    pub target: Entity,
    /// The damage to apply.
    pub damage: f32,
    /// The hit point in world coordinates.
    pub hit_point: Vec3,
    /// Pre-resolved hit sound file path relative to assets/ (e.g., "audio/hit.mp3" or "components/projectiles/laser-standard/hit.mp3").
    /// None if no hit sound.
    pub hit_sound: Option<String>,
}

/// Event emitted when the active ship camera changes.
///
/// Emitted by `camera_switch_system` when the player switches cameras.
/// The cockpit overlay system listens for this to show/hide the overlay.
#[derive(Message, Debug)]
pub struct CameraSwitched {
    /// The name of the newly active camera (e.g., "cockpit", "front", "drone").
    pub camera_name: String,
}

/// Event emitted when the targeting mode changes.
///
/// Emitted by `targeting_mode_toggle_system` when the player toggles
/// between Combat and Nav targeting modes.
/// The UI system listens for this to show a notification.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetingModeChanged {
    /// The new targeting mode.
    pub mode: super::navigation::TargetingModeType,
}

/// Event emitted when the navigation list is updated.
///
/// Emitted by `update_navigation_list_system` after rebuilding the navigation
/// list entries. The navigation menu UI listens for this to refresh its content.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationListChanged;

/// Event emitted when the selected target changes.
///
/// Emitted by `cycle_target_system` when the player cycles to a new target.
/// The notification system listens for this to show a notification.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetSelected {
    /// The newly selected target entity.
    pub target: Entity,
    /// The targeting mode when the target was selected.
    pub mode: super::navigation::TargetingModeType,
}

/// Event emitted when the player selects a weapon.
///
/// Emitted by `weapon_selection_system` when the player presses a weapon
/// selection key. The notification system listens for this to show a notification.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct WeaponSelected {
    /// The weapon slot index (0-9).
    pub slot: usize,
    /// The weapon name (e.g., "laser-standard").
    pub weapon_name: String,
}

/// Event emitted when the player selects a propulsion (main thruster).
///
/// Emitted by `propulsion_selection_system` when the player presses a propulsion
/// selection key. The notification system listens for this to show a notification.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct PropulsionSelected {
    /// The thruster slot index (0-9).
    pub slot: usize,
    /// The thruster name (e.g., "chemical-main").
    pub thruster_name: String,
}
