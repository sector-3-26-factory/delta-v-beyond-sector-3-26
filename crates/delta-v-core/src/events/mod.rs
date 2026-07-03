// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Events emitted for entity spawning.
//!
//! [`SpawnEntity`] is defined here (in `delta-v-core`) so that all domain
//! crates can reference it without creating domain-to-domain dependencies.
//! The world plugin emits these events; domain plugins listen for them.
//!
//! See ADR-0005 (plugin architecture) and ADR-0038 (entity template system).

use bevy::prelude::*;
use serde_json::Value;

/// Event emitted when an entity should be spawned from a template.
///
/// The world plugin emits these events during [`AppState::LoadingWorld`].
/// Domain-specific plugins (`ShipsPlugin`, `SunPlugin`, etc.) listen for this
/// event, filter by `entity_type`, and spawn the entity with appropriate
/// components.
///
/// This decouples world loading from entity spawning, allowing each domain
/// to own its spawn logic (ADR-0005).
#[derive(Message, Debug, Clone)]
pub struct SpawnEntity {
    /// Unique identifier for this entity instance (from world definition).
    /// Used for debug filtering, save/load, networking, and player-facing UI.
    pub id: String,

    /// The entity type discriminator (e.g., `"player_controlled_ship"`, `"sun"`).
    /// Must match an `entity_type` in a template JSON file.
    pub entity_type: String,

    /// The loaded and validated template JSON.
    /// Contains all static properties for the entity.
    pub template: Value,

    /// Path to the template file (e.g., `templates/ships/player_ship/template.json`).
    pub template_path: String,

    /// Path to the template that contains the mesh (e.g., `templates/ships/space-fighter-comrade1280/template.json`).
    /// For standalone ship templates, this is the same as `template_path`.
    /// For `player_controlled_ship`, this is the referenced `ship_template` path.
    /// The mesh is always at `mesh.glb` in this template's directory.
    pub mesh_template_path: String,

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
}

impl SpawnEntity {
    /// Creates a new `SpawnEntity` event with default rotation and scale.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this entity instance.
    /// * `entity_type` - Discriminator for the entity type.
    /// * `template` - Loaded template JSON.
    /// * `template_path` - Path to the template file.
    /// * `mesh_template_path` - Path to the template containing the mesh.
    /// * `position` - Spawn position in metres.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(
        id: String,
        entity_type: String,
        template: Value,
        template_path: String,
        mesh_template_path: String,
        position: Vec3,
    ) -> Self {
        Self {
            id,
            entity_type,
            template,
            template_path,
            mesh_template_path,
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            ai_task: None,
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
/// Contains the projectile, target, damage, and hit point for VFX/sound.
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
    pub mode: TargetingModeType,
}

/// The targeting mode type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetingModeType {
    /// Combat mode: targeting enemies.
    Combat,
    /// Navigation mode: selecting objects to navigate to.
    Nav,
}
