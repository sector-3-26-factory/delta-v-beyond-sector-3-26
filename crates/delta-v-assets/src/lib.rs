// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Asset loading and template management.
//!
//! This crate owns:
//! - Asset path resolution
//! - Template loading
//! - Template merging
//!
//! See ADR-0049 for the template system reorganization.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(unreachable_pub)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::dbg_macro)]

use bevy::app::Plugin;

pub mod error;
pub mod paths;
pub mod template;

// Re-exports for convenience
pub use error::AssetError;
pub use paths::resolve_template_path;
pub use template::{load_asteroid, load_player_controlled_ship, load_ship, load_template};

/// Plugin for asset loading and template management.
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, _app: &mut bevy::app::App) {
        // Asset plugin initialization
        // Template loading is done on demand via the template module functions
    }
}
