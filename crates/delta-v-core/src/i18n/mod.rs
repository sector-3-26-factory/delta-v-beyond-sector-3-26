// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Internationalization (i18n) types for compile-time verified translations.
//!
//! This module re-exports the i18n types from `delta-v-types` for convenience.
//! Translation keys are validated at compile time; missing or mistyped keys are
//! compiler errors, not runtime warnings.
//!
//! See ADR-0037 (Internationalization).

pub use delta_v_types::i18n::{
    I18n, KeybindingsMenuTranslations, MenuTranslations, UiTranslations,
};
