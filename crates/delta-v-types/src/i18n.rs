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

//! Internationalization (i18n) types for compile-time verified translations.
//!
//! This module provides a typed struct hierarchy for all user-visible strings.
//! Translation keys are validated at compile time; missing or mistyped keys are
//! compiler errors, not runtime warnings.
//!
//! See ADR-0037 (Internationalization).

use bevy::prelude::Resource;
use serde::Deserialize;

/// Root i18n struct containing all translation hierarchies.
#[derive(Debug, Clone, Deserialize, Resource)]
pub struct I18n {
    /// UI translations.
    pub ui: UiTranslations,
}

/// UI translations.
#[derive(Debug, Clone, Deserialize)]
pub struct UiTranslations {
    /// Menu translations.
    pub menu: MenuTranslations,
    /// Notification translations.
    pub notification: NotificationTranslations,
}

/// Menu translations.
#[derive(Debug, Clone, Deserialize)]
pub struct MenuTranslations {
    /// Keybindings menu translations.
    pub keybindings: KeybindingsMenuTranslations,
}

/// Keybindings menu translations.
#[derive(Debug, Clone, Deserialize)]
pub struct KeybindingsMenuTranslations {
    /// Menu title.
    pub title: String,
    /// Close button hint.
    pub close: String,
    /// Action group names.
    pub group: std::collections::HashMap<String, String>,
    /// Logical action names.
    pub action: std::collections::HashMap<String, String>,
    /// Physical key names.
    pub key: std::collections::HashMap<String, String>,
}

/// Notification translations.
#[derive(Debug, Clone, Deserialize)]
pub struct NotificationTranslations {
    /// Camera notification template. Use `{camera_name}` as placeholder.
    pub camera: String,
    /// Camera name translations.
    pub camera_name: std::collections::HashMap<String, String>,
}
