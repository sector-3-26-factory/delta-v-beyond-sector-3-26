// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Approach-comparison benchmarks for physics.
//!
//! Per ADR-0056 the directory name is the benchmark purpose, and every file
//! in it MUST declare that same purpose in its `BENCHMARK-PURPOSE` header.
//! This module is compiled into the binary behind the `bench` feature flag,
//! which is independent of `test` so benchmarks are excluded from day-to-day
//! `cargo test --workspace`. They still run on the same real plugin stack:
//! the binary is built with `--features test,bench`.

#[cfg(feature = "bench")]
pub mod physics_capacity_benchmarks_tests;

/// Register all benchmark functions in the global registry.
///
/// Called once at binary startup (from `main.rs` when the `bench`
/// feature is enabled). Each benchmark is registered with its display
/// name and a `fn(&mut App)` entry point.
///
/// The registration calls must be inside a function body — they cannot
/// be module-level statements.
#[cfg(feature = "bench")]
pub fn init_registry() {
    physics_capacity_benchmarks_tests::init_registry();
}
