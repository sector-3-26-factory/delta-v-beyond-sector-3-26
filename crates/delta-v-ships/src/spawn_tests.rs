// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for ship components.
//!
//! Full integration tests with entity spawning are deferred to future
//! milestones. These tests verify component trait bounds.

use crate::PlayerShip;

#[test]
fn test_player_ship_is_send_sync() {
    // Verify PlayerShip is Send + Sync for use in Bevy ECS.
    // This is a compile-time test; if it compiles, the assertion passes.
    const fn assert_send_sync<T: Send + Sync>() {}
    const fn check() {
        assert_send_sync::<PlayerShip>();
    }
    check();
}
