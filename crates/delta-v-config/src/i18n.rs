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

//! Internationalization (i18n) loading and validation.
//!
//! Loads the English translation file and validates it against the i18n schema.
//! The loaded translations are inserted as an [`I18n`] resource for use by UI systems.
//!
//! See ADR-0037 (Internationalization).

use std::path::PathBuf;

use crate::ConfigError;
use delta_v_assets::get_workspace_root;
use delta_v_json::loader as json_loader;
use delta_v_types::I18n;

/// Loads and validates the i18n file.
///
/// Uses [`json_loader::load_validated`] to load and validate the English
/// translation file against the i18n schema.
///
/// # Errors
///
/// Returns [`ConfigError`] if the file cannot be read, parsed, or validated.
pub fn load_i18n() -> Result<I18n, ConfigError> {
    let json_path: PathBuf = get_workspace_root().join("assets/i18n/en.json");
    let schema_path: PathBuf = get_workspace_root().join("assets/json/schema/i18n.schema.json");

    let value = json_loader::load_validated(&json_path, &schema_path)
        .map_err(|e| ConfigError::from_json_error(e, &json_path))?;

    serde_json::from_value(value).map_err(|e| ConfigError::Parse {
        path: json_path,
        source: e,
    })
}
