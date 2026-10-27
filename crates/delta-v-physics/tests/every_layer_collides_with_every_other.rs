// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! ADR-0057: every collision layer collides with every other layer.
//!
//!   xvfb-run -a cargo test --test `every_layer_collides_with_every_other` -p delta-v-physics

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_app::{AppConfig, build_app};
use delta_v_test_support::{check, make_app, run_harness};
use delta_v_types::collision::layers;

fn build_test_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-physics/tests/fixtures/assets".to_string(),
        world_name: Some("floating_origin".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
}

fn test(_app: &mut App) {
    let all = [
        ("ship", layers::SHIP),
        ("asteroid", layers::ASTEROID),
        ("projectile", layers::PROJECTILE),
        ("celestial", layers::CELESTIAL),
    ];
    for (name, body) in all {
        check(
            body.mask == layers::ALL_LAYERS,
            &format!("{name} excludes something from collision"),
        );
        for (other_name, other) in all {
            check(
                (body.mask & other.layers) != 0,
                &format!("{name} cannot collide with a {other_name}"),
            );
        }
    }
}

fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(&mut app, &[("test", test)]);
}
