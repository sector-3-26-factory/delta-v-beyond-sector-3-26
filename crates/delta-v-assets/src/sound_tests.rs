// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Tests for sound asset resolution.

use super::sound::resolve_sound_path;

#[test]
fn test_resolve_sound_path_fallback() {
    // When component-specific sound doesn't exist and no fallback audio exists,
    // should return None (audio files will be added in Step 7)
    let path = resolve_sound_path("components/weapons/laser-standard", "fire");
    assert_eq!(path, None);
}
