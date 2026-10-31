// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Real-app integration tests for `delta-v-ui`.
//!
//! Each subdirectory owns its own test functions, driver, and fixtures.
//! Test functions are registered in the global registry via
//! [`init_registry`], called from `main.rs`.

#[cfg(feature = "test")]
pub mod ui;

/// Register all test functions in the global registry.
///
/// Called once at binary startup (from `main.rs` when the `test`
/// feature is enabled). Each test function is registered with its
/// display name and a `fn(&mut App)` entry point.
///
/// The registration calls must be inside a function body — they cannot
/// be module-level statements.
#[cfg(feature = "test")]
pub fn init_registry() {
    ui::init_registry();
}
