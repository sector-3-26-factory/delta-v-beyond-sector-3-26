// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Weapon and projectile systems.

use bevy::prelude::*;
use delta_v_core::input::ActionState;
use delta_v_core::{FireWeapon, Health, PlayerShipEntity, ProjectileHit, Weapon};
use delta_v_physics::{CollisionDetected, RigidBody};
use delta_v_types::LogicalAction;

use crate::components::Projectile;
use crate::resources::WeaponState;
use crate::spawn::spawn_projectile;

/// System set for weapon systems running in `FixedUpdate`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponsSet {
    /// Process fire commands and spawn projectiles.
    ProcessFireCommands,
    /// Handle projectile collisions and apply damage.
    ProjectileCollision,
    /// Update projectile lifetimes and despawn expired ones.
    UpdateProjectiles,
}

/// Detects fire input and emits [`FireWeapon`] events.
///
/// Runs in `FixedUpdate` after the input translation systems.
/// Uses edge detection to fire on press, not on hold.
#[allow(clippy::needless_pass_by_value)]
pub fn fire_input_system(
    action_state: Res<'_, ActionState<LogicalAction>>,
    mut weapon_state: ResMut<'_, WeaponState>,
    mut events: MessageWriter<'_, FireWeapon>,
    ship_entity: Res<'_, PlayerShipEntity>,
) {
    let fire_held = action_state.pressed(&LogicalAction::FirePrimary);
    let was_held = weapon_state.fire_held_prev;

    // Edge detection: fire on press, not hold
    if fire_held && !was_held {
        tracing::debug!(
            "FirePrimary pressed, sending FireWeapon event for ship {:?}",
            ship_entity.0
        );
        events.write(FireWeapon {
            source: ship_entity.0,
            weapon_index: 0,
        });
    }

    weapon_state.fire_held_prev = fire_held;
}

/// Processes [`FireWeapon`] events and spawns projectiles.
///
/// Runs in `FixedUpdate`. Reads the source entity's transform and rigid body
/// to calculate the projectile's initial velocity (source velocity + forward speed).
#[allow(clippy::needless_pass_by_value)]
pub fn process_fire_commands(
    mut commands: Commands<'_, '_>,
    mut events: MessageReader<'_, '_, FireWeapon>,
    ship_query: Query<'_, '_, (&Transform, &RigidBody)>,
    weapon_query: Query<'_, '_, &Weapon>,
) {
    for event in events.read() {
        let Ok((transform, body)) = ship_query.get(event.source) else {
            tracing::debug!(
                "Could not get ship transform/body for entity {:?}",
                event.source
            );
            continue;
        };

        if let Ok(weapon) = weapon_query.get(event.source) {
            tracing::debug!(
                "Spawning projectile from ship {:?}, damage={}",
                event.source,
                weapon.damage
            );
            spawn_projectile(&mut commands, event.source, transform, body, weapon);
        } else {
            tracing::warn!("Ship {:?} has no Weapon component!", event.source);
        }
    }
}

/// Handles projectile collisions: applies damage to targets with [`Health`]
/// and despawns the projectile on any hit.
///
/// Runs in `FixedUpdate`. When a projectile collides with another entity,
/// the projectile is always despawned. If the target has a [`Health`] component,
/// damage is applied. A [`ProjectileHit`] event is emitted for every hit
/// (including hits on entities without health, e.g. asteroids).
///
/// This system consumes [`CollisionDetected`] events from the physics crate.
/// It checks both entities in the collision to find the projectile, since
/// the collision detection iterates all pairs without ordering guarantees.
#[allow(clippy::needless_pass_by_value)]
pub fn projectile_collision_system(
    mut commands: Commands<'_, '_>,
    mut collision_events: MessageReader<'_, '_, CollisionDetected>,
    projectile_query: Query<'_, '_, &Projectile>,
    mut health_query: Query<'_, '_, &mut Health>,
    mut hit_events: MessageWriter<'_, ProjectileHit>,
) {
    for collision in collision_events.read() {
        // Check both entities to find which one is the projectile.
        // The collision detection iterates all pairs without ordering guarantees,
        // so the projectile could be either `target` or `other`.
        let (projectile_entity, projectile, target_entity) = match (
            projectile_query.get(collision.target),
            projectile_query.get(collision.other),
        ) {
            (Ok(p), _) => (collision.target, p, collision.other),
            (Err(_), Ok(p)) => (collision.other, p, collision.target),
            (Err(_), Err(_)) => continue, // Neither entity is a projectile
        };

        // Don't hit the source (the entity that fired this projectile)
        if target_entity == projectile.source {
            continue;
        }

        // Apply damage to the target entity if it has health
        if let Ok(mut health) = health_query.get_mut(target_entity) {
            let destroyed = health.apply_damage(projectile.damage);
            if destroyed {
                tracing::info!("Entity {target_entity:?} destroyed by projectile");
            }
        }

        // Emit hit event for VFX/sound (even if target has no health, e.g. asteroids)
        hit_events.write(ProjectileHit {
            projectile: projectile_entity,
            target: target_entity,
            damage: projectile.damage,
            hit_point: collision.point,
            hit_sound: projectile.hit_sound.clone(),
        });

        // Despawn the projectile on any hit
        commands.entity(projectile_entity).despawn();
    }
}

/// Updates projectile lifetimes and despawns expired ones.
///
/// Runs in `FixedUpdate`. Decrements each projectile's lifetime by the
/// fixed timestep delta. When lifetime reaches zero, the projectile is despawned.
#[allow(clippy::needless_pass_by_value)]
pub fn update_projectiles(
    mut commands: Commands<'_, '_>,
    mut query: Query<'_, '_, (Entity, &mut Projectile)>,
    time: Res<'_, Time<Fixed>>,
) {
    let dt = time.delta_secs();
    for (entity, mut projectile) in &mut query {
        projectile.lifetime -= dt;
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
