// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Real-app integration tests for `delta-v-physics`.
//!
//! This crate is compiled into the `delta-v` binary behind the `test`
//! feature flag. It is NOT linked into production builds.
//!
//! Test functions live under [`real_app_ITs`] and are registered in the
//! global registry via [`real_app_ITs::init_registry`], which is called
//! from `main.rs` at startup when `--test-function <name>` is passed.
//!
//! See plans/testframework-poc.md Section 9 and ADR-0061.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code,
    dead_code,
    non_snake_case
)]

#[cfg(feature = "test")]
pub mod real_app_ITs;
