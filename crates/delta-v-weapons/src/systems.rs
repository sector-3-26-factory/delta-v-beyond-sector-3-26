// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Weapon and projectile systems.

use bevy::prelude::*;
use delta_v_core::{
    ActiveActions, FireWeapon, Health, LogicalAction, PlayerShipEntity, ProjectileHit, Weapon,
};
use delta_v_physics::{CollisionDetected, RigidBody};

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
/// Runs in `FixedUpdate` after [`delta_v_core::InputSet::Translate`].
/// Uses edge detection to fire on press, not on hold.
#[allow(clippy::needless_pass_by_value)]
pub fn fire_input_system(
    active: Res<'_, ActiveActions>,
    mut weapon_state: ResMut<'_, WeaponState>,
    mut events: EventWriter<'_, FireWeapon>,
    ship_entity: Res<'_, PlayerShipEntity>,
) {
    let fire_held = active.0.contains(&LogicalAction::FirePrimary);
    let was_held = weapon_state.fire_held_prev;

    // Edge detection: fire on press, not hold
    if fire_held && !was_held {
        log::debug!(
            "FirePrimary pressed, sending FireWeapon event for ship {:?}",
            ship_entity.0
        );
        events.send(FireWeapon {
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
    mut events: EventReader<'_, '_, FireWeapon>,
    ship_query: Query<'_, '_, (&Transform, &RigidBody)>,
    weapon_query: Query<'_, '_, &Weapon>,
) {
    for event in events.read() {
        let Ok((transform, body)) = ship_query.get(event.source) else {
            log::debug!(
                "Could not get ship transform/body for entity {:?}",
                event.source
            );
            continue;
        };

        if let Ok(weapon) = weapon_query.get(event.source) {
            log::debug!(
                "Spawning projectile from ship {:?}, damage={}",
                event.source,
                weapon.damage
            );
            spawn_projectile(&mut commands, event.source, transform, body, weapon);
        } else {
            log::warn!("Ship {:?} has no Weapon component!", event.source);
        }
    }
}

/// Handles projectile collisions and applies damage.
///
/// Runs in `FixedUpdate`. When a projectile collides with another entity,
/// applies damage to the target's [`Health`] component and emits a
/// [`ProjectileHit`] event.
#[allow(clippy::needless_pass_by_value)]
pub fn projectile_collision_system(
    mut collision_events: EventReader<'_, '_, CollisionDetected>,
    projectile_query: Query<'_, '_, &Projectile>,
    mut health_query: Query<'_, '_, &mut Health>,
    mut hit_events: EventWriter<'_, ProjectileHit>,
) {
    for collision in collision_events.read() {
        // Check if the target is a projectile
        let Ok(projectile) = projectile_query.get(collision.target) else {
            continue;
        };

        // Don't hit the source
        if collision.other == projectile.source {
            continue;
        }

        // Apply damage to the other entity
        if let Ok(mut health) = health_query.get_mut(collision.other) {
            let destroyed = health.apply_damage(projectile.damage);
            hit_events.send(ProjectileHit {
                projectile: collision.target,
                target: collision.other,
                damage: projectile.damage,
                hit_point: collision.point,
            });

            if destroyed {
                log::info!("Entity {:?} destroyed by projectile", collision.other);
            }
        }
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
    let dt = time.delta_seconds();
    for (entity, mut projectile) in &mut query {
        projectile.lifetime -= dt;
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
