// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Weapon and projectile systems.

use bevy::gltf::Gltf;
use bevy::prelude::*;
use delta_v_core::input::ActionState;
use delta_v_core::{FireWeapon, Health, PlayerShipEntity, ProjectileHit, SelectedWeapon, Weapon};
use delta_v_physics::{CollisionDetected, RigidBody};
use delta_v_types::LogicalAction;

use crate::components::Projectile;
use crate::resources::WeaponState;
use crate::spawn::spawn_projectile;
use delta_v_core::WeaponSelected;
use delta_v_spawn::mesh_attachment::PendingMesh;

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

/// Detects weapon selection input and updates the [`SelectedWeapon`] resource.
///
/// Runs in `FixedUpdate` after the input translation systems.
/// Handles `SelectWeapon1` through `SelectWeapon10` actions.
/// Uses edge detection to select on press, not on hold.
/// Emits [`WeaponSelected`] event for notification display.
/// Only allows selecting weapons that exist on the ship.
#[allow(clippy::needless_pass_by_value, clippy::missing_panics_doc)]
pub fn weapon_selection_system(
    action_state: Res<'_, ActionState<LogicalAction>>,
    mut selected_weapon: ResMut<'_, SelectedWeapon>,
    mut events: MessageWriter<'_, WeaponSelected>,
    ship_entity: Res<'_, PlayerShipEntity>,
    children_query: Query<'_, '_, &Children>,
    weapon_query: Query<'_, '_, &Weapon>,
) {
    // Check each weapon selection action and update the selected index
    // on press (edge detection)
    let new_index = if action_state.just_pressed(&LogicalAction::SelectWeapon1) {
        0
    } else if action_state.just_pressed(&LogicalAction::SelectWeapon2) {
        1
    } else if action_state.just_pressed(&LogicalAction::SelectWeapon3) {
        2
    } else if action_state.just_pressed(&LogicalAction::SelectWeapon4) {
        3
    } else if action_state.just_pressed(&LogicalAction::SelectWeapon5) {
        4
    } else if action_state.just_pressed(&LogicalAction::SelectWeapon6) {
        5
    } else if action_state.just_pressed(&LogicalAction::SelectWeapon7) {
        6
    } else if action_state.just_pressed(&LogicalAction::SelectWeapon8) {
        7
    } else if action_state.just_pressed(&LogicalAction::SelectWeapon9) {
        8
    } else if action_state.just_pressed(&LogicalAction::SelectWeapon10) {
        9
    } else {
        return; // No weapon selection action pressed
    };

    if selected_weapon.index == new_index {
        return;
    }

    // Only allow selecting weapons that exist (use max_weapons_count from resource)
    if new_index >= selected_weapon.max_weapons_count {
        return;
    }

    // Get the weapon at the new index to verify it exists
    let weapon_entity = children_query.get(ship_entity.0).ok().and_then(|children| {
        children
            .iter()
            .filter(|child| weapon_query.get(*child).is_ok())
            .nth(new_index)
    });

    // If no weapon exists at this slot, don't allow selection
    let Some(weapon_entity) = weapon_entity else {
        return;
    };

    // INVARIANT: weapon_entity was found by the filter above, so it must have a Weapon component
    #[allow(clippy::unwrap_used)]
    let weapon = weapon_query.get(weapon_entity).unwrap();
    let weapon_name = weapon.weapon_name.clone();

    selected_weapon.index = new_index;
    tracing::info!("Weapon selected: {} (slot {})", weapon_name, new_index + 1);
    events.write(WeaponSelected {
        slot: new_index,
        weapon_name,
    });
}

/// Detects fire input and emits [`FireWeapon`] events.
///
/// Runs in `FixedUpdate` after the input translation systems.
/// Uses edge detection to fire on press, not on hold.
/// Fires from the currently selected weapon (child entity).
#[allow(clippy::needless_pass_by_value)]
pub fn fire_input_system(
    action_state: Res<'_, ActionState<LogicalAction>>,
    mut weapon_state: ResMut<'_, WeaponState>,
    mut events: MessageWriter<'_, FireWeapon>,
    ship_entity: Res<'_, PlayerShipEntity>,
    selected_weapon: Res<'_, SelectedWeapon>,
    children_query: Query<'_, '_, &Children>,
    weapon_query: Query<'_, '_, &Weapon>,
) {
    let fire_held = action_state.pressed(&LogicalAction::FirePrimary);
    let was_held = weapon_state.fire_held_prev;

    // Edge detection: fire on press, not hold
    if fire_held && !was_held {
        // Get the selected weapon from the ship's children
        let weapon_entity = children_query
            .get(ship_entity.0)
            .ok()
            .and_then(|children| {
                children
                    .iter()
                    .filter(|child| weapon_query.get(*child).is_ok())
                    .nth(selected_weapon.index)
            })
            .unwrap_or(ship_entity.0);

        if let Ok(weapon) = weapon_query.get(weapon_entity) {
            tracing::debug!(
                "FirePrimary pressed, sending FireWeapon event for ship {:?}, weapon slot {}",
                ship_entity.0,
                weapon.slot
            );
            events.write(FireWeapon {
                source: ship_entity.0,
                weapon_index: weapon.slot,
            });
        } else {
            tracing::debug!(
                "FirePrimary pressed, sending FireWeapon event for ship {:?} (no weapon children)",
                ship_entity.0
            );
            events.write(FireWeapon {
                source: ship_entity.0,
                weapon_index: 0,
            });
        }
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
    asset_server: Res<'_, AssetServer>,
    mut events: MessageReader<'_, '_, FireWeapon>,
    ship_query: Query<'_, '_, (&Transform, &RigidBody)>,
    children_query: Query<'_, '_, &Children>,
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

        // Find the weapon with the matching slot index
        let weapon = children_query.get(event.source).ok().and_then(|children| {
            children
                .iter()
                .find(|child| {
                    weapon_query
                        .get(*child)
                        .map(|w| w.slot == event.weapon_index)
                        .unwrap_or(false)
                })
                .and_then(|child| weapon_query.get(child).ok())
        });

        if let Some(weapon) = weapon {
            tracing::debug!(
                "Spawning projectile from ship {:?}, damage={}",
                event.source,
                weapon.damage
            );
            spawn_projectile(
                &mut commands,
                &asset_server,
                event.source,
                transform,
                body,
                weapon,
            );
        } else {
            tracing::warn!(
                "Ship {:?} has no Weapon component with slot {}!",
                event.source,
                event.weapon_index
            );
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

/// Attaches loaded projectile meshes to entities.
///
/// Runs in `Update` after asset loading. When a projectile's glTF asset
/// finishes loading, this system extracts all scenes from the glTF and
/// attaches them as children of the projectile entity, then removes the
/// pending mesh marker.
///
/// Per ADR-0044, all visual data comes from external files loaded via the
/// asset pipeline.
#[allow(clippy::needless_pass_by_value)]
pub fn attach_projectile_meshes(
    mut commands: Commands<'_, '_>,
    gltf_assets: Res<'_, Assets<Gltf>>,
    query: Query<'_, '_, (Entity, &crate::spawn::PendingProjectileMesh)>,
) {
    for (entity, pending) in &query {
        if let Some(gltf) = gltf_assets.get(pending.gltf_handle()) {
            if gltf.scenes.is_empty() {
                continue;
            }
            for scene_handle in &gltf.scenes {
                let child = commands.spawn(SceneRoot(scene_handle.clone())).id();
                commands.entity(entity).add_child(child);
            }
            commands
                .entity(entity)
                .remove::<crate::spawn::PendingProjectileMesh>();
        }
    }
}
