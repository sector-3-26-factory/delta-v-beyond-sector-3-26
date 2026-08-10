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

//! Reusable UI layout helpers for i18n-safe label/value pairs.
//!
//! This module provides grid-based layout components that handle variable-length
//! translations gracefully. Labels wrap within their column, while values remain
//! on a single line and align across all rows.

use bevy::prelude::*;
use bevy::ui::{GridPlacement, GridTrack};

use crate::window::theme::UiTheme;

/// Spawns a group header that spans the full width of a 2-column grid.
///
/// Must be used as a child of a grid with `grid_template_columns: [fr(1), auto]`.
///
/// # Arguments
///
/// * `ui` - The child spawner commands for the grid container
/// * `theme` - The UI theme resource
/// * `text` - The header text
pub fn spawn_group_header(ui: &mut ChildSpawnerCommands<'_>, theme: &Res<'_, UiTheme>, text: &str) {
    ui.spawn((
        Name::new("GroupHeader"),
        Node {
            grid_column: GridPlacement::span(2),
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
    ))
    .with_children(|header| {
        header.spawn((
            Name::new("GroupHeaderText"),
            Text::new(text),
            TextFont {
                font: bevy::prelude::FontSource::Handle(theme.font.clone()),
                font_size: UiTheme::GROUP_HEADER_FONT_SIZE,
                ..default()
            },
            TextColor(UiTheme::GROUP_HEADER_COLOR),
            TextLayout {
                linebreak: LineBreak::NoWrap,
                ..default()
            },
        ));
    });
}

/// Spawns a label/value row in a CSS Grid container.
///
/// The parent must be a grid with `grid_template_columns: [fr(1), auto]`.
/// This function places the label in column 1 and value in column 2.
///
/// # Arguments
///
/// * `grid` - The grid container's child spawner
/// * `theme` - The UI theme resource
/// * `label` - The label text (e.g., "Thrust Forward")
/// * `value` - The value text (e.g., "W")
/// * `label_color` - Color for the label text
/// * `value_color` - Color for the value text
pub fn spawn_grid_label_value_row(
    grid: &mut ChildSpawnerCommands<'_>,
    theme: &Res<'_, UiTheme>,
    label: &str,
    value: &str,
    label_color: Color,
    value_color: Color,
) {
    // Label in column 1 (takes remaining space, wraps)
    grid.spawn((
        Name::new("Label"),
        Node {
            grid_column: GridPlacement::start(1),
            width: Val::Percent(100.0),
            ..default()
        },
    ))
    .with_children(|label_node| {
        label_node.spawn((
            Name::new("LabelText"),
            Text::new(label),
            TextFont {
                font: bevy::prelude::FontSource::Handle(theme.font.clone()),
                font_size: UiTheme::TEXT_FONT_SIZE,
                ..default()
            },
            TextColor(label_color),
            // Text wraps within the column
            TextLayout::default(),
        ));
    });

    // Value in column 2 (sizes to content, stays on one line)
    grid.spawn((
        Name::new("Value"),
        Node {
            grid_column: GridPlacement::start(2),
            ..default()
        },
    ))
    .with_children(|value_node| {
        value_node.spawn((
            Name::new("ValueText"),
            Text::new(value),
            TextFont {
                font: bevy::prelude::FontSource::Handle(theme.font.clone()),
                font_size: UiTheme::TEXT_FONT_SIZE,
                ..default()
            },
            TextColor(value_color),
            TextLayout {
                linebreak: LineBreak::NoWrap,
                ..default()
            },
        ));
    });
}

/// Spawns a grid container for label/value pairs.
///
/// Returns a closure that can be used to spawn children into the grid.
///
/// # Arguments
///
/// * `ui` - The child spawner commands for the parent container
/// * `row_gap` - Vertical gap between rows
///
/// # Returns
///
/// A `ChildSpawnerCommands` that can be used to spawn children into the grid.
pub fn spawn_label_value_grid(ui: &mut ChildSpawnerCommands<'_>, row_gap: Val) -> Entity {
    ui.spawn((
        Name::new("LabelValueGrid"),
        Node {
            width: Val::Percent(100.0),
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::fr(1.0), GridTrack::auto()],
            column_gap: Val::Px(8.0),
            row_gap,
            ..default()
        },
    ))
    .id()
}
