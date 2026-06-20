// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Keybindings menu spawning functions.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_lunex::prelude::*;
use delta_v_core::{I18n, KeybindingsResource};

use super::components::KeybindingsMenuRoot;
use crate::window::{WindowConfig, spawn_window};

// ============================================================================
// Lunex-based keybindings menu
// ============================================================================

/// Spawns the keybindings menu using Lunex UI.
///
/// Displays a 300x200px semi-transparent window centered on screen.
/// The title and hint are positioned side by side in one row.
/// Press F1 to toggle visibility.
pub fn spawn_keybindings_menu(
    commands: &mut Commands<'_, '_>,
    i18n: &I18n,
    _keybindings: &KeybindingsResource,
) {
    let title = i18n.ui.menu.keybindings.title.clone();
    let hint = i18n.ui.menu.keybindings.close.clone();

    let menu_root = spawn_window(
        commands,
        &WindowConfig {
            title,
            hint,
            ..default()
        },
        keybindings_menu_content,
    );

    // Add the keybindings-specific marker so the toggle system can find/despawn this entity.
    commands.entity(menu_root).insert(KeybindingsMenuRoot);

    tracing::debug!("spawned keybindings menu root entity {menu_root:?}");
}

/// Recursively despawns the keybindings menu entity and all its children.
pub fn despawn_keybindings_menu(commands: &mut Commands<'_, '_>, entity: Entity) {
    commands.entity(entity).despawn();
}

/// Spawns the keybindings menu content inside the window content area.
fn keybindings_menu_content(ui: &mut ChildSpawnerCommands<'_>) {
    spawn_content_text(ui);
}

/// Spawns the content text entity inside the keybindings menu.
fn spawn_content_text(ui: &mut ChildSpawnerCommands<'_>) {
    ui.spawn((
        Name::new("ContentText"),
        UiLayout::window()
            .pos((Rl(0.0), Rl(0.0)))
            .anchor(Anchor::TOP_LEFT)
            .pack(),
        UiTextSize::from(Ab(16.0)),
        Text2d::new("hello world hallo welt hello world hallo welt "),
        TextLayout {
            justify: Justify::Left,
            linebreak: LineBreak::AnyCharacter,
        },
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Pickable::IGNORE,
        RenderLayers::layer(2),
    ));
}
