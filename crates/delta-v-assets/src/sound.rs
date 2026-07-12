// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Sound asset resolution with component-directory-first fallback to assets/audio/.

use std::path::Path;

/// Supported audio extensions in priority order.
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "ogg"];

/// Resolves a sound file path for a component at spawn/load time.
///
/// Search order:
/// 1. `{component_dir}/{sound_name}.{ext}` for each extension
/// 2. `assets/audio/{sound_name}.{ext}` for each extension (fallback)
///
/// Returns the path relative to the asset root (e.g., "components/projectiles/laser-standard/hit.mp3"
/// or "audio/hit.mp3") suitable for `AssetServer::load()`.
///
/// This should be called ONCE at spawn/template load time, not per-frame.
#[must_use]
pub fn resolve_sound_path(component_dir: &str, sound_name: &str) -> Option<String> {
    // Check component directory first
    for ext in AUDIO_EXTENSIONS {
        let path = format!("{component_dir}/{sound_name}.{ext}");
        if Path::new(&format!("assets/{path}")).exists() {
            return Some(path);
        }
    }

    // Fallback to assets/audio/
    for ext in AUDIO_EXTENSIONS {
        let path = format!("audio/{sound_name}.{ext}");
        if Path::new(&format!("assets/{path}")).exists() {
            return Some(path);
        }
    }

    None
}
