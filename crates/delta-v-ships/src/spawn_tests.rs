// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for ship spawning.
//!
//! TODO (M2/M3): Add integration tests for `ShipsPlugin`.
//! Current limitation: Unit tests with `MinimalPlugins` cannot properly test
//! the full plugin lifecycle because:
//! 1. `ShipsPlugin` depends on `AppState` transitions that require `ConfigPlugin`,
//!    `WorldPlugin`, and other plugins to run in sequence
//! 2. The `SpawnEntity` event system requires the full world loading pipeline
//!    to populate required resources (`WorldDefResource`, templates, etc.)
//! 3. Full integration tests require a real asset loader and proper state machine
//!
//! When adding tests: use `bevy::app::AppBuilder` with all required plugins,
//! or defer to end-to-end tests that run `cargo run` and validate the scene.
//! See Track 4 Definition of Done: `test_player_ship_spawns_ok`,
//! `test_player_ship_has_correct_position`, `test_player_ship_entity_stored_in_resource`.
