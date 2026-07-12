// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Projectile spawning.

use bevy::prelude::*;
use delta_v_core::Weapon;
use delta_v_physics::{CollisionLayersComponent, CollisionShape, RigidBody};

use crate::components::Projectile;
use delta_v_types::collision::layers;

/// Spawns a projectile entity.
///
/// The projectile inherits the source entity's velocity plus
/// the weapon's projectile speed in the forward direction
/// (per ADR-0006: -Z is forward).
///
/// # Arguments
///
/// * `commands` - Bevy commands buffer.
/// * `source` - The entity firing the projectile.
/// * `source_transform` - The source entity's transform.
/// * `source_body` - The source entity's rigid body.
/// * `weapon` - The weapon configuration.
///
/// Returns the spawned projectile entity.
#[allow(clippy::cast_possible_truncation)]
pub fn spawn_projectile(
    commands: &mut Commands<'_, '_>,
    source: Entity,
    source_transform: &Transform,
    source_body: &RigidBody,
    weapon: &Weapon,
) -> Entity {
    // Calculate projectile velocity: source velocity + weapon speed in forward direction
    let forward = source_transform.rotation * Vec3::NEG_Z;
    let initial_velocity = source_body.velocity + forward * weapon.projectile_speed;

    let mut rigid_body = RigidBody::new(1.0, 1.0);
    rigid_body.velocity = initial_velocity;

    commands
        .spawn((
            Transform {
                translation: source_transform.translation + forward * 1.0,
                rotation: source_transform.rotation,
                scale: Vec3::ONE,
            },
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            rigid_body,
            CollisionShape::sphere(weapon.projectile_radius, Vec3::ZERO),
            CollisionLayersComponent::new(layers::PROJECTILE),
            Projectile {
                source,
                lifetime: weapon.lifetime,
                damage: weapon.damage,
                hit_sound: weapon.hit_sound.clone(),
            },
        ))
        .id()
}
