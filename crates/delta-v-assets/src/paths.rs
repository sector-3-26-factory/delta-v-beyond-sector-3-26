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

//! Asset path resolution utilities.

use std::path::PathBuf;

use delta_v_types::TemplatePath;

/// Returns the absolute path to the workspace root directory.
///
/// This is the centralized function for getting the workspace root,
/// derived from `CARGO_MANIFEST_DIR` (which points at the crate directory).
///
/// # Panics
///
/// Panics if `CARGO_MANIFEST_DIR` is not set or the workspace directory structure
/// is unexpected. This should never happen in normal cargo builds.
#[must_use]
#[allow(clippy::expect_used)] // INVARIANT: CARGO_MANIFEST_DIR is always set by cargo; workspace structure is fixed
pub fn get_workspace_root() -> PathBuf {
    let manifest_dir = std::env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("CARGO_MANIFEST_DIR parent must exist")
        .parent()
        .expect("workspace root must exist")
        .to_path_buf()
}

/// Resolves a template path relative to the `assets/templates/` root.
///
/// # Example
///
/// ```
/// use delta_v_assets::resolve_template_path;
/// let path = resolve_template_path("ships", "meshy-cargo-1");
/// // Returns: "templates/ships/meshy-cargo-1/"
/// ```
#[must_use]
pub fn resolve_template_path(category: &str, name: &str) -> String {
    format!("templates/{category}/{name}/")
}

/// Resolves a template path from a `TemplatePath` struct.
#[must_use]
pub fn resolve_template_path_from(template: &TemplatePath) -> String {
    format!("templates/{}/{}/", template.category, template.name)
}
