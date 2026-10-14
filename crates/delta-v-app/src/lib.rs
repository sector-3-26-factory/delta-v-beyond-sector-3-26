// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Central app builder — Single Source of Truth for plugin stack and order.
//!
//! This crate exports `build_app` which is the ONLY function that registers
//! the full plugin stack. Both `main.rs` and ALL integration tests MUST use
//! `delta_v_app::build_app`.
//!
//! See ADR-0060 for the architectural rule.

#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::cargo
)]
#![allow(clippy::multiple_crate_versions)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]

use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::asset::AssetPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::PluginGroup;
use bevy::window::{Window, WindowPlugin};
use bevy::winit::WinitPlugin;

use delta_v_ai::AiPlugin;
use delta_v_assets::AssetsPlugin;
use delta_v_config::ConfigPlugin;
use delta_v_core::CorePlugin;
use delta_v_items::ItemsPlugin;
use delta_v_net::NetPlugin;
use delta_v_physics::PhysicsPlugin;
use delta_v_propulsion::PropulsionPlugin;
use delta_v_ships::ShipsPlugin;
use delta_v_stations::StationsPlugin;
use delta_v_ui::UiPlugin;
use delta_v_weapons::WeaponsPlugin;
use delta_v_world::{WorldPath, WorldPlugin};

/// Configuration for app creation.
///
/// All defaults defined centrally here; call sites must not set their own.
/// `asset_root_suffix` is a path fragment (e.g., "assets" or "tests/fixtures/assets"),
/// NOT a full path. delta-v-assets resolves the absolute path via `get_workspace_root()`.
// allow-default: AppConfig has no JSON source; defaults are centralized here per ADR-0039
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Headless mode: no window, no Winit, for CI/tests.
    pub is_headless: bool,
    /// Asset root suffix (path fragment under workspace root).
    /// Default: "assets" (production). Tests: "tests/fixtures/assets".
    pub asset_root_suffix: String,
    /// Optional world name (directory name under `asset_root/worlds`/).
    /// Inserted as `WorldPath` resource.
    pub world_name: Option<String>,
    /// Optional log filter (overrides default).
    pub log_filter: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            is_headless: false,
            asset_root_suffix: "assets".to_string(),
            world_name: Some("default".to_string()),
            log_filter: None,
        }
    }
}

/// Central app creation — Single Source of Truth for plugin stack and order.
pub fn build_app(config: AppConfig) -> App {
    let mut app = App::new();

    // 1. DefaultPlugins with headless support
    let plugins = if config.is_headless {
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                ..Default::default()
            })
            .disable::<WinitPlugin>()
    } else {
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Delta-V beyond Sector 3.26".to_string(),
                resolution: (1280, 720).into(),
                ..Default::default()
            }),
            ..Default::default()
        })
    };

    // 2. AssetPlugin with path resolved by delta-v-assets
    let asset_path = delta_v_assets::resolve_asset_path(&config.asset_root_suffix);
    let plugins = plugins.set(AssetPlugin {
        file_path: asset_path,
        ..Default::default()
    });

    // 3. LogPlugin with optional filter (defaults per ADR-0015)
    let filter = config.log_filter.unwrap_or_else(|| {
        #[cfg(feature = "dev")]
        {
            "warn,delta_v=debug,delta_v_core=debug,delta_v_config=debug,\
             delta_v_physics=debug,delta_v_assets=debug,delta_v_ships=debug,\
             delta_v_propulsion=debug,delta_v_weapons=debug,delta_v_stations=debug,\
             delta_v_items=debug,delta_v_world=debug,delta_v_ai=debug"
                .to_string()
        }
        #[cfg(not(feature = "dev"))]
        {
            "warn,delta_v=info,delta_v_core=info,delta_v_config=info,\
             delta_v_physics=info,delta_v_assets=info,delta_v_ships=info,\
             delta_v_propulsion=info,delta_v_weapons=info,delta_v_stations=info,\
             delta_v_items=info,delta_v_world=info,delta_v_ai=info"
                .to_string()
        }
    });
    let plugins = plugins.set(LogPlugin {
        filter,
        level: bevy::log::Level::TRACE,
        ..Default::default()
    });
    app.add_plugins(plugins);

    // 4. WorldPath resource if world name provided
    if let Some(world_name) = config.world_name {
        let world_path = delta_v_assets::resolve_world_path(&config.asset_root_suffix, &world_name);
        app.insert_resource(WorldPath(world_path));
    }

    // 5. EXACT plugin registration order from main.rs (Single Source of Truth!)
    // Technical plugins — must come before domain plugins.
    app.add_plugins(CorePlugin) // CorePlugin owns AppState, must come first
        .add_plugins(ConfigPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(AssetsPlugin)
        .add_plugins(NetPlugin);

    // Domain plugins.
    app.add_plugins(ShipsPlugin)
        .add_plugins(PropulsionPlugin)
        .add_plugins(WeaponsPlugin)
        .add_plugins(StationsPlugin)
        .add_plugins(ItemsPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(AiPlugin);

    // UI plugin last.
    app.add_plugins(UiPlugin);

    app
}
