// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Notification spawning functions.

use bevy::prelude::*;
use delta_v_core::RenderLayer;

use crate::window::theme::UiTheme;
use crate::window::{WindowAnimation, WindowConfig, spawn_window};

use super::components::Notification;
use super::resources::{NotificationConfig, NotificationStack};

/// Spawns a notification window with the given translated message.
///
/// The y-position is calculated from the notification stack so that new
/// notifications appear below existing ones.
///
/// Returns the spawned window root entity.
#[allow(clippy::needless_pass_by_value)]
pub fn spawn_notification(
    commands: &mut Commands<'_, '_>,
    config: &NotificationConfig,
    stack: &mut NotificationStack,
    message: String,
    theme: &Res<'_, UiTheme>,
    asset_server: &Res<'_, AssetServer>,
) -> Entity {
    // Calculate y-offset: bottom of last notification + gap, or 0 for first
    let y_offset = if let Some((_, _, prev_y)) = stack.entries.last() {
        *prev_y + config.window_size.y + config.stack_gap
    } else {
        0.0
    };

    let window_config = WindowConfig {
        title: String::new(),
        hint: String::new(),
        size: config.window_size,
        header_height: 0.0,
        animation: Some(WindowAnimation::flicker(
            config.animation.flicker_in_duration,
            config.animation.hold_duration,
            config.animation.flicker_out_duration,
            config.animation.flicker_count,
        )),
        scrollable: false,
    };

    let window_entity = spawn_window(commands, &window_config, asset_server, theme, |ui| {
        ui.spawn((
            Name::new("NotificationTextWrapper"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|wrapper| {
            wrapper.spawn((
                Name::new("NotificationText"),
                Text::new(&message),
                TextFont {
                    font: theme.font.clone(),
                    font_size: config.font_size,
                    ..default()
                },
                TextColor(config.color),
            ));
        });
    });

    // Override root node to shrink-to-fit
    commands.entity(window_entity).insert(Node {
        width: Val::Px(config.window_size.x),
        height: Val::Px(config.window_size.y),
        ..default()
    });

    // Wrap in positioned container
    let container_entity = commands
        .spawn((
            Name::new("NotificationContainer"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(config.position.x),
                top: Val::Px(config.position.y + y_offset),
                ..default()
            },
            RenderLayer::Menu.render_layers(),
        ))
        .add_child(window_entity)
        .id();

    commands.entity(window_entity).insert(Notification);

    // Add to stack: (window_entity, container_entity, y_offset)
    stack
        .entries
        .push((window_entity, container_entity, y_offset));

    window_entity
}
