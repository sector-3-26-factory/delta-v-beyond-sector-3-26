// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Internationalization (i18n) types for compile-time verified translations.
//!
//! This module provides a typed struct hierarchy for all user-visible strings.
//! Translation keys are validated at compile time; missing or mistyped keys are
//! compiler errors, not runtime warnings.
//!
//! See ADR-0037 (Internationalization).

use serde::Deserialize;

/// Root i18n struct containing all translation hierarchies.
#[derive(Debug, Clone, Deserialize)]
pub struct I18n {
    /// UI translations.
    pub ui: UiTranslations,
}

/// UI translations.
#[derive(Debug, Clone, Deserialize)]
pub struct UiTranslations {
    /// Menu translations.
    pub menu: MenuTranslations,
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
