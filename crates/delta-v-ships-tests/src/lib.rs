// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Real-app integration tests for `delta-v-ships`.
//!
//! This crate is compiled into the `delta-v` binary behind the `test`
//! feature flag. It is NOT linked into production builds.
//!
//! Test functions live under [`real_app_ITs`] and are registered in the
//! global registry via [`real_app_ITs::init_registry`], which is called
//! from `main.rs` at startup when `--test-function <name>` is passed.
//!
//! See plans/testframework-poc.md Section 9 and ADR-0061.

#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::cargo
)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    // The workspace already allows multiple_crate_versions; the
    // crate-level `clippy::cargo` warn below re-enables this lint,
    // which is noise from transitive dep version skew (avian3d 0.6.1
    // pulls Bevy 0.18.1 alongside the workspace's Bevy 0.19). Re-allow
    // it here so `cargo clippy -- -D warnings` stays green.
    clippy::multiple_crate_versions
)]

#[cfg(feature = "test")]
#[allow(non_snake_case)]
pub mod real_app_ITs;
