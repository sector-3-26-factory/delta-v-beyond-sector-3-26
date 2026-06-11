// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Asset path resolution utilities.

use delta_v_types::TemplatePath;

/// Resolves a template path relative to the `assets/templates/` root.
///
/// # Example
///
/// ```
/// use delta_v_assets::resolve_template_path;
/// let path = resolve_template_path("ships", "meshy-cargo-1");
/// // Returns: "templates/ships/meshy-cargo-1/"
/// ```
pub fn resolve_template_path(category: &str, name: &str) -> String {
    format!("templates/{}/{}/", category, name)
}

/// Resolves a template path from a TemplatePath struct.
pub fn resolve_template_path_from(template: &TemplatePath) -> String {
    format!("templates/{}/{}/", template.category, template.name)
}
