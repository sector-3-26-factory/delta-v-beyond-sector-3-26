// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! ADR-0057: every collision layer collides with every other layer.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code
)]

use bevy::prelude::*;
use delta_v_types::collision::layers;

fn test(_app: &mut App) {
    let all = [
        ("ship", layers::SHIP),
        ("asteroid", layers::ASTEROID),
        ("projectile", layers::PROJECTILE),
        ("celestial", layers::CELESTIAL),
    ];
    for (name, body) in all {
        assert!(
            body.mask == layers::ALL_LAYERS,
            "{name} excludes something from collision"
        );
        for (other_name, other) in all {
            assert!(
                (body.mask & other.layers) != 0,
                "{name} cannot collide with a {other_name}"
            );
        }
    }
}

pub fn init_registry() {
    do_register("physics::every_layer_collides_with_every_other", test);
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
