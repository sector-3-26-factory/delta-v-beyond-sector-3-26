// AGENTS: before modifying this file, read AGENTS.md at the repository root.
//
//! Floating origin recentering tests.
//
//! Each test function is registered in the global registry via
//! [`init_registry`] and called by the binary's `run_test_function()`
//! when `--test-function <name>` is passed.
//
//! Uses the `floating_origin` world fixture (empty — no celestial bodies)
//! so that no gravity interferes with the recentering assertions.
//
//! # Panics
//!
//! Each test runs in its own process (one `--test-function` call per
//! process), so a failed assertion panics and the binary exits 1. This
//! is the intended failure mode: the test driver checks the exit code.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp,
    unsafe_code
)]

use bevy::prelude::*;

/// Register all floating-origin test functions in the global registry.
pub fn init_registry() {
    do_register(
        "physics::floating_origin_no_recenter_below_threshold",
        test_floating_origin_no_recenter_below_threshold,
    );
    do_register(
        "physics::floating_origin_recenter_above_threshold",
        test_floating_origin_recenter_above_threshold,
    );
    do_register(
        "physics::floating_origin_accumulates_offset_with_nonzero_initial_offset",
        test_floating_origin_accumulates_offset_with_nonzero_initial_offset,
    );
    do_register(
        "physics::floating_origin_preserves_rotation_on_recenter",
        test_floating_origin_preserves_rotation_on_recenter,
    );
    do_register(
        "physics::floating_origin_preserves_scale_on_recenter",
        test_floating_origin_preserves_scale_on_recenter,
    );
}

fn test_floating_origin_no_recenter_below_threshold(app: &mut App) {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

    app.insert_resource(FloatingOrigin::new(Vec3::ZERO));
    app.insert_resource(FloatingOriginConfig {
        recenter_threshold_m: 5_000.0,
    });

    let player_id = app.world().resource::<PlayerShipEntity>().0;
    app.world_mut()
        .entity_mut(player_id)
        .insert(Transform::from_translation(Vec3::new(1_000.0, 0.0, 0.0)));

    let other_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)))
        .id();

    let original_player_pos = app.world().get::<Transform>(player_id).unwrap().translation;
    let original_other_pos = app.world().get::<Transform>(other_id).unwrap().translation;

    delta_v_test_framework::run_fixed_update(app);

    let player_transform = app.world().get::<Transform>(player_id).unwrap();
    let other_transform = app.world().get::<Transform>(other_id).unwrap();

    assert_eq!(
        player_transform.translation, original_player_pos,
        "player below threshold should not be recentered"
    );
    assert_eq!(
        other_transform.translation, original_other_pos,
        "other entity should not be recentered when player is below threshold"
    );
}

fn test_floating_origin_recenter_above_threshold(app: &mut App) {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

    app.insert_resource(FloatingOrigin::new(Vec3::ZERO));
    app.insert_resource(FloatingOriginConfig {
        recenter_threshold_m: 5_000.0,
    });

    let player_id = app.world().resource::<PlayerShipEntity>().0;
    app.world_mut()
        .entity_mut(player_id)
        .insert(Transform::from_translation(Vec3::new(6_000.0, 0.0, 0.0)));

    let other_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(7_600.0, 0.0, 0.0)))
        .id();

    delta_v_test_framework::run_fixed_update(app);

    let origin = app.world().get_resource::<FloatingOrigin>().unwrap();
    assert_eq!(
        origin.offset,
        Vec3::new(6_000.0, 0.0, 0.0),
        "origin should be recentered to player position"
    );

    let player_transform = app.world().get::<Transform>(player_id).unwrap();
    assert_eq!(
        player_transform.translation,
        Vec3::ZERO,
        "player should be at origin after recentering"
    );

    let other_transform = app.world().get::<Transform>(other_id).unwrap();
    assert_eq!(
        other_transform.translation,
        Vec3::new(1_600.0, 0.0, 0.0),
        "other entity should be at relative position after recentering"
    );
}

fn test_floating_origin_accumulates_offset_with_nonzero_initial_offset(app: &mut App) {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

    app.insert_resource(FloatingOrigin::new(Vec3::new(10_000.0, 0.0, 0.0)));
    app.insert_resource(FloatingOriginConfig {
        recenter_threshold_m: 5_000.0,
    });

    let player_id = app.world().resource::<PlayerShipEntity>().0;
    app.world_mut()
        .entity_mut(player_id)
        .insert(Transform::from_translation(Vec3::new(-6_000.0, 0.0, 0.0)));

    let other_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(-4_400.0, 0.0, 0.0)))
        .id();

    delta_v_test_framework::run_fixed_update(app);

    let origin = app.world().get_resource::<FloatingOrigin>().unwrap();
    assert_eq!(
        origin.offset,
        Vec3::new(4_000.0, 0.0, 0.0),
        "origin should be accumulated to player's local position"
    );

    let player_transform = app.world().get::<Transform>(player_id).unwrap();
    assert_eq!(
        player_transform.translation,
        Vec3::ZERO,
        "player should be at origin after recentering"
    );

    let other_transform = app.world().get::<Transform>(other_id).unwrap();
    assert_eq!(
        other_transform.translation,
        Vec3::new(1_600.0, 0.0, 0.0),
        "other entity should be at relative position after recentering"
    );
}

fn test_floating_origin_preserves_rotation_on_recenter(app: &mut App) {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

    app.insert_resource(FloatingOrigin::new(Vec3::ZERO));
    app.insert_resource(FloatingOriginConfig {
        recenter_threshold_m: 5_000.0,
    });

    let player_id = app.world().resource::<PlayerShipEntity>().0;
    app.world_mut()
        .entity_mut(player_id)
        .insert(Transform::from_translation(Vec3::new(6_000.0, 0.0, 0.0)));

    let rotation = Quat::from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_4);
    let other_id = app
        .world_mut()
        .spawn(Transform {
            translation: Vec3::new(7_600.0, 0.0, 0.0),
            rotation,
            ..default()
        })
        .id();

    delta_v_test_framework::run_fixed_update(app);

    let other_transform = app.world().get::<Transform>(other_id).unwrap();
    assert_eq!(
        other_transform.rotation, rotation,
        "other entity's rotation should be preserved after recentering"
    );
}

fn test_floating_origin_preserves_scale_on_recenter(app: &mut App) {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

    app.insert_resource(FloatingOrigin::new(Vec3::ZERO));
    app.insert_resource(FloatingOriginConfig {
        recenter_threshold_m: 5_000.0,
    });

    let player_id = app.world().resource::<PlayerShipEntity>().0;
    app.world_mut()
        .entity_mut(player_id)
        .insert(Transform::from_translation(Vec3::new(6_000.0, 0.0, 0.0)));

    let scale = Vec3::new(400.0, 400.0, 400.0);
    let other_id = app
        .world_mut()
        .spawn(Transform {
            translation: Vec3::new(7_600.0, 0.0, 0.0),
            scale,
            ..default()
        })
        .id();

    delta_v_test_framework::run_fixed_update(app);

    let other_transform = app.world().get::<Transform>(other_id).unwrap();
    assert_eq!(
        other_transform.scale, scale,
        "other entity's scale should be preserved after recentering"
    );
}

fn do_register(name: &'static str, func: fn(&mut App)) {
    let entry = delta_v_test_framework::TestEntry { name, func };
    let leaked: &'static delta_v_test_framework::TestEntry = Box::leak(Box::new(entry));
    delta_v_test_framework::register(leaked);
}
