// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Notification lifecycle systems.

use bevy::prelude::*;

use crate::window::{WindowAnimation, WindowAnimationFlickerPhase};

use super::components::Notification;
use super::resources::NotificationStack;
use delta_v_core::{EntityType, WorldEntityId};

/// Despawns notification entities whose animation has finished.
///
/// Removes them from the stack, shifts remaining notifications up,
/// updates container positions, and despawns.
#[allow(clippy::needless_pass_by_value, irrefutable_let_patterns)]
pub fn notification_cleanup_system(
    mut commands: Commands<'_, '_>,
    mut stack: ResMut<'_, NotificationStack>,
    config: Res<'_, super::resources::NotificationConfig>,
    query: Query<'_, '_, (Entity, &WindowAnimation), With<Notification>>,
) {
    for (entity, animation) in &query {
        let WindowAnimation::Flicker(flicker) = animation else {
            continue;
        };
        if flicker.phase == WindowAnimationFlickerPhase::Done {
            if let Some(pos) = stack.entries.iter().position(|(e, _, _)| *e == entity) {
                let removed_height = config.window_size.y + config.stack_gap;
                let (_, container, _) = stack.entries.remove(pos);
                // Shift remaining notifications up and update their positions
                for (_, cont, y_offset) in stack.entries.iter_mut().skip(pos) {
                    *y_offset -= removed_height;
                    let new_top = config.position.y + *y_offset;
                    commands.entity(*cont).insert(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(config.position.x),
                        top: Val::Px(new_top),
                        ..default()
                    });
                }
                commands.entity(container).despawn();
            }
            commands.entity(entity).despawn();
        }
    }
}

/// Listens for `CameraSwitched` messages and spawns notification windows.
#[allow(
    clippy::needless_pass_by_value,
    clippy::literal_string_with_formatting_args
)]
pub fn notification_spawn_system(
    mut events: MessageReader<'_, '_, delta_v_core::CameraSwitched>,
    i18n: Res<'_, delta_v_core::I18n>,
    config: Res<'_, super::resources::NotificationConfig>,
    mut stack: ResMut<'_, NotificationStack>,
    theme: Res<'_, crate::window::theme::UiTheme>,
    asset_server: Res<'_, AssetServer>,
    mut commands: Commands<'_, '_>,
) {
    for event in events.read() {
        let camera_name = i18n
            .ui
            .notification
            .camera_name
            .get(&event.camera_name)
            .cloned()
            .unwrap_or_else(|| event.camera_name.clone());

        let message = i18n
            .ui
            .notification
            .camera
            .replace("{camera_name}", &camera_name);

        tracing::debug!(
            "[notification] spawning camera switch notification: '{}'",
            message
        );

        super::spawn::spawn_notification(
            &mut commands,
            &config,
            &mut stack,
            message,
            &theme,
            &asset_server,
        );
    }
}

/// Listens for `TargetSelected` messages and spawns notification windows.
#[allow(
    clippy::needless_pass_by_value,
    clippy::literal_string_with_formatting_args,
    clippy::too_many_arguments
)]
pub fn target_selected_notification_system(
    mut events: MessageReader<'_, '_, delta_v_core::TargetSelected>,
    i18n: Res<'_, delta_v_core::I18n>,
    config: Res<'_, super::resources::NotificationConfig>,
    mut stack: ResMut<'_, NotificationStack>,
    theme: Res<'_, crate::window::theme::UiTheme>,
    asset_server: Res<'_, AssetServer>,
    mut commands: Commands<'_, '_>,
    entity_query: Query<'_, '_, (Option<&Name>, &EntityType, &WorldEntityId)>,
) {
    let events_vec: Vec<_> = events.read().collect();
    if !events_vec.is_empty() {
        tracing::debug!(
            "[notification] received {} TargetSelected event(s)",
            events_vec.len()
        );
    }
    for event in events_vec {
        tracing::debug!(
            "[notification] processing TargetSelected event: target={:?} mode={:?}",
            event.target,
            event.mode
        );
        // Get entity info for the notification
        let (name_str, entity_type_str) = match entity_query.get(event.target) {
            Ok((name, entity_type, entity_id)) => {
                let name_str = name.map_or_else(|| entity_id.0.clone(), ToString::to_string);
                (name_str, entity_type.0.clone())
            }
            Err(_) => ("Unknown".to_string(), "Unknown".to_string()),
        };

        let message = match event.mode {
            delta_v_core::navigation::TargetingModeType::Combat => i18n
                .ui
                .notification
                .target_selected
                .replace("{entity_id}", &name_str)
                .replace("{entity_type}", &entity_type_str),
            delta_v_core::navigation::TargetingModeType::Nav => i18n
                .ui
                .notification
                .nav_object_selected
                .replace("{entity_id}", &name_str)
                .replace("{entity_type}", &entity_type_str),
        };

        tracing::debug!(
            "[notification] spawning target selected notification: '{}' (mode={:?})",
            message,
            event.mode
        );

        super::spawn::spawn_notification(
            &mut commands,
            &config,
            &mut stack,
            message,
            &theme,
            &asset_server,
        );
    }
}

/// Listens for `TargetingModeChanged` messages and spawns notification windows.
///
/// Reads the `TargetingMode` resource directly to ensure consistency with
/// the navigation menu content.
#[allow(
    clippy::needless_pass_by_value,
    clippy::literal_string_with_formatting_args,
    clippy::too_many_arguments
)]
pub fn targeting_mode_notification_system(
    mut events: MessageReader<'_, '_, delta_v_core::TargetingModeChanged>,
    targeting_mode: Res<'_, delta_v_core::TargetingMode>,
    i18n: Res<'_, delta_v_core::I18n>,
    config: Res<'_, super::resources::NotificationConfig>,
    mut stack: ResMut<'_, NotificationStack>,
    theme: Res<'_, crate::window::theme::UiTheme>,
    asset_server: Res<'_, AssetServer>,
    mut commands: Commands<'_, '_>,
) {
    // Only spawn notification if there was a mode change event
    let Some(event) = events.read().next() else {
        return;
    };

    // Read the current mode from the resource for consistency
    let resource_mode = targeting_mode.mode;
    let event_mode = event.mode;

    tracing::debug!(
        "[notification] received TargetingModeChanged event: event_mode={:?}, resource_mode={:?}",
        event_mode,
        resource_mode
    );

    let mode_name = match resource_mode {
        delta_v_core::TargetingModeType::Combat => {
            i18n.ui.notification.targeting_mode_name.combat.clone()
        }
        delta_v_core::TargetingModeType::Nav => {
            i18n.ui.notification.targeting_mode_name.nav.clone()
        }
    };

    let message = i18n
        .ui
        .notification
        .targeting_mode
        .replace("{mode}", &mode_name);

    tracing::debug!(
        "[notification] spawning targeting mode notification: '{}' (resource_mode={:?})",
        message,
        resource_mode
    );

    super::spawn::spawn_notification(
        &mut commands,
        &config,
        &mut stack,
        message,
        &theme,
        &asset_server,
    );
}

/// Listens for `WeaponSelected` messages and spawns notification windows.
///
/// Displays the selected weapon name.
#[allow(
    clippy::needless_pass_by_value,
    clippy::literal_string_with_formatting_args,
    clippy::too_many_arguments
)]
pub fn weapon_selected_notification_system(
    mut events: MessageReader<'_, '_, delta_v_core::WeaponSelected>,
    i18n: Res<'_, delta_v_core::I18n>,
    config: Res<'_, super::resources::NotificationConfig>,
    mut stack: ResMut<'_, NotificationStack>,
    theme: Res<'_, crate::window::theme::UiTheme>,
    asset_server: Res<'_, AssetServer>,
    mut commands: Commands<'_, '_>,
) {
    for event in events.read() {
        let message = i18n
            .ui
            .notification
            .weapon_selected
            .replace("{name}", &event.weapon_name);

        tracing::debug!(
            "[notification] spawning weapon selected notification: {}",
            event.weapon_name
        );

        super::spawn::spawn_notification(
            &mut commands,
            &config,
            &mut stack,
            message,
            &theme,
            &asset_server,
        );
    }
}

/// Listens for `PropulsionSelected` messages and spawns notification windows.
///
/// Displays the selected propulsion name.
#[allow(
    clippy::needless_pass_by_value,
    clippy::literal_string_with_formatting_args,
    clippy::too_many_arguments
)]
pub fn propulsion_selected_notification_system(
    mut events: MessageReader<'_, '_, delta_v_core::PropulsionSelected>,
    i18n: Res<'_, delta_v_core::I18n>,
    config: Res<'_, super::resources::NotificationConfig>,
    mut stack: ResMut<'_, NotificationStack>,
    theme: Res<'_, crate::window::theme::UiTheme>,
    asset_server: Res<'_, AssetServer>,
    mut commands: Commands<'_, '_>,
) {
    for event in events.read() {
        let message = i18n
            .ui
            .notification
            .propulsion_selected
            .replace("{name}", &event.thruster_name);

        super::spawn::spawn_notification(
            &mut commands,
            &config,
            &mut stack,
            message,
            &theme,
            &asset_server,
        );
    }
}
