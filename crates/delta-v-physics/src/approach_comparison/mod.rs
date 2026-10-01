// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Approach-comparison benchmarks for this crate.
//!
//! Per ADR-0056 the directory name is the benchmark purpose, and every file
//! in it MUST declare that same purpose in its `BENCHMARK-PURPOSE` header.
//! This module is compiled only when the `bench` feature is enabled, so the
//! benchmarks never run as part of day-to-day `cargo test --workspace`.
//!
//! Each file names its own subject, so a purpose may hold as many benchmark
//! files as it needs:
//!
//! - `performance_benchmarks_tests.rs`
//!
//! Run them with:
//! `cargo test -p delta-v-physics --features bench -- --ignored --nocapture`

#[cfg(test)]
#[path = "performance_benchmarks_tests.rs"]
mod performance_benchmarks_tests;
