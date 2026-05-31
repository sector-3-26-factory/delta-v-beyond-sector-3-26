// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Events emitted by the world system for entity spawning.
//!
//! See ADR-0038 (Entity template system).

use bevy::prelude::*;
use serde_json::Value;

/// Event emitted when an entity should be spawned from a template.
///
/// Domain-specific plugins (`ShipsPlugin`, `SunPlugin`, etc.) listen for this
/// event, filter by `entity_type`, and spawn the entity with appropriate
/// components.
///
/// This decouples world loading from entity spawning, allowing each domain
/// to own its spawn logic (ADR-0005).
#[derive(Event, Debug, Clone)]
pub struct SpawnEntity {
    /// Unique identifier for this entity instance (from world definition).
    /// Used for debug filtering, save/load, networking, and player-facing UI.
    pub id: String,

    /// The entity type discriminator (e.g., `"local_player_ship"`, `"sun"`).
    /// Must match an `entity_type` in a template JSON file.
    pub entity_type: String,

    /// The loaded and validated template JSON.
    /// Contains all static properties for the entity.
    pub template: Value,

    /// Spawn position in world coordinates (metres).
    pub position: Vec3,

    /// Optional rotation (unit quaternion). Defaults to identity.
    pub rotation: Quat,

    /// Optional scale. Defaults to 1.0 on all axes.
    pub scale: Vec3,
}

impl SpawnEntity {
    /// Creates a new `SpawnEntity` event with default rotation and scale.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this entity instance.
    /// * `entity_type` - Discriminator for the entity type.
    /// * `template` - Loaded template JSON.
    /// * `position` - Spawn position in metres.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(id: String, entity_type: String, template: Value, position: Vec3) -> Self {
        Self {
            id,
            entity_type,
            template,
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
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
}
