// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Floating origin recentering tests.
//!
//! Registered as a `[[test]]` target with `harness = false` in Cargo.toml so
//! that `cargo test --test floating_origin_tests` runs `main()` on the
//! **main thread**, which is required by Bevy's `WinitPlugin`.
//!
//! Uses the `floating_origin` world fixture (empty — no celestial bodies) so
//! that no gravity interferes with the recentering assertions. The `test`
//! world fixture places Earth at the origin, which would pull the player and
//! break these tests.
//!
//!   xvfb-run -a cargo test --test `floating_origin_tests` -p delta-v-physics
//!
//! See ADR-0007 (floating origin) and ADR-0060 (Test framework single
//! source of truth).

use bevy::prelude::*;
use delta_v_app::{AppConfig, build_app};
use delta_v_test_framework::{make_app, run_fixed_update, run_harness};

/// Build the test `App` via `build_app` (Single Source of Truth per ADR-0060).
/// Uses the `floating_origin` world fixture (empty — no celestial bodies).
fn build_test_app() -> App {
    build_app(AppConfig {
        asset_root_suffix: "crates/delta-v-physics/tests/fixtures/assets".to_string(),
        world_name: Some("floating_origin".to_string()),
        log_filter: Some("warn,delta_v=debug".to_string()),
    })
}

// ---------------------------------------------------------------------------
// P3: Floating origin recentering tests
// ---------------------------------------------------------------------------
//
// These tests exercise `check_and_recenter_origin_system` from
// `delta-v-physics/src/floating_origin_systems.rs` against a shared `App`
// built from the empty `floating_origin` world fixture.
// ---------------------------------------------------------------------------

fn test_floating_origin_no_recenter_below_threshold(app: &mut App) {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

    app.insert_resource(FloatingOrigin::new(Vec3::ZERO));
    app.insert_resource(FloatingOriginConfig {
        recenter_threshold_m: 5_000.0,
    });

    // Move the world's player ship to a position below the recentering threshold.
    // The player ship is provided by the `floating_origin` world fixture — we
    // never spawn a second ship (only one entity may be player-controlled).
    let player_id = app.world().resource::<PlayerShipEntity>().0;
    app.world_mut()
        .entity_mut(player_id)
        .insert(Transform::from_translation(Vec3::new(1_000.0, 0.0, 0.0)));

    // Spawn another entity to verify it doesn't get recentered
    let other_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)))
        .id();

    let original_player_pos = app.world().get::<Transform>(player_id).unwrap().translation;
    let original_other_pos = app.world().get::<Transform>(other_id).unwrap().translation;

    run_fixed_update(app);

    let player_transform = app.world().get::<Transform>(player_id).unwrap();
    let other_transform = app.world().get::<Transform>(other_id).unwrap();

    // Player should still be at the same position (no recentering below threshold)
    assert_eq!(
        player_transform.translation, original_player_pos,
        "player below threshold should not be recentered"
    );
    // Other entity should also not be recentered
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

    // Move the world's player ship to a position above the recentering threshold.
    let player_id = app.world().resource::<PlayerShipEntity>().0;
    app.world_mut()
        .entity_mut(player_id)
        .insert(Transform::from_translation(Vec3::new(6_000.0, 0.0, 0.0)));

    // Spawn another entity at 1.6km from player (in front)
    let other_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(7_600.0, 0.0, 0.0))) // 6000 + 1600
        .id();

    run_fixed_update(app);

    // Check that the origin was recentered to the player's position
    let origin = app.world().get_resource::<FloatingOrigin>().unwrap();
    assert_eq!(
        origin.offset,
        Vec3::new(6_000.0, 0.0, 0.0),
        "origin should be recentered to player position"
    );

    // Check that the player is now at the origin (local position 0,0,0)
    let player_transform = app.world().get::<Transform>(player_id).unwrap();
    assert_eq!(
        player_transform.translation,
        Vec3::ZERO,
        "player should be at origin after recentering"
    );

    // Check that the other entity is now at 1600 (relative to player)
    let other_transform = app.world().get::<Transform>(other_id).unwrap();
    assert_eq!(
        other_transform.translation,
        Vec3::new(1_600.0, 0.0, 0.0),
        "other entity should be at relative position after recentering"
    );
}

fn test_floating_origin_accumulates_offset_with_nonzero_initial_offset(app: &mut App) {
    use delta_v_core::{FloatingOrigin, FloatingOriginConfig, PlayerShipEntity};

    // Start with a non-zero offset to test accumulation
    app.insert_resource(FloatingOrigin::new(Vec3::new(10_000.0, 0.0, 0.0)));
    app.insert_resource(FloatingOriginConfig {
        recenter_threshold_m: 5_000.0,
    });

    // Move the world's player ship to a local position (-6000, 0, 0) which
    // has distance 6000 from origin. This will trigger recentering since
    // 6000 > 5000 threshold. Absolute position = (-6000, 0, 0) + (10000, 0, 0)
    // = (4000, 0, 0).
    let player_id = app.world().resource::<PlayerShipEntity>().0;
    app.world_mut()
        .entity_mut(player_id)
        .insert(Transform::from_translation(Vec3::new(-6_000.0, 0.0, 0.0)));

    // Spawn another entity at 1.6km from player (in +X direction in local space)
    // Other entity at local position (-6000 + 1600, 0, 0) = (-4400, 0, 0)
    // Absolute position = (-4400, 0, 0) + (10000, 0, 0) = (5600, 0, 0)
    let other_id = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(-4_400.0, 0.0, 0.0)))
        .id();

    run_fixed_update(app);

    // After recentering:
    // Player was at local position (-6000, 0, 0) relative to origin at (10000, 0, 0)
    // New offset should be (10000, 0, 0) + (-6000, 0, 0) = (4000, 0, 0)
    let origin = app.world().get_resource::<FloatingOrigin>().unwrap();
    assert_eq!(
        origin.offset,
        Vec3::new(4_000.0, 0.0, 0.0),
        "origin should be accumulated to player's local position"
    );

    // Player should be at origin (0, 0, 0)
    let player_transform = app.world().get::<Transform>(player_id).unwrap();
    assert_eq!(
        player_transform.translation,
        Vec3::ZERO,
        "player should be at origin after recentering"
    );

    // Other entity should be at 1600 (relative to player)
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

    // Move the world's player ship to a position above threshold (6km away)
    let player_id = app.world().resource::<PlayerShipEntity>().0;
    app.world_mut()
        .entity_mut(player_id)
        .insert(Transform::from_translation(Vec3::new(6_000.0, 0.0, 0.0)));

    // Spawn another entity with a specific rotation
    let rotation = Quat::from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_4); // 45 degrees
    let other_id = app
        .world_mut()
        .spawn(Transform {
            translation: Vec3::new(7_600.0, 0.0, 0.0),
            rotation,
            ..default()
        })
        .id();

    run_fixed_update(app);

    // Check that the other entity's rotation is preserved
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

    // Move the world's player ship to a position above threshold (6km away)
    let player_id = app.world().resource::<PlayerShipEntity>().0;
    app.world_mut()
        .entity_mut(player_id)
        .insert(Transform::from_translation(Vec3::new(6_000.0, 0.0, 0.0)));

    // Spawn another entity with a non-default scale (e.g., 400.0 for a scaled-down sun)
    let scale = Vec3::new(400.0, 400.0, 400.0);
    let other_id = app
        .world_mut()
        .spawn(Transform {
            translation: Vec3::new(7_600.0, 0.0, 0.0),
            scale,
            ..default()
        })
        .id();

    run_fixed_update(app);

    // Check that the other entity's scale is preserved after recentering
    let other_transform = app.world().get::<Transform>(other_id).unwrap();
    assert_eq!(
        other_transform.scale, scale,
        "other entity's scale should be preserved after recentering"
    );
}

/// Entry point — runs on the main thread so `WinitPlugin` can initialize.
fn main() {
    let mut app = make_app(60, build_test_app);
    run_harness(
        &mut app,
        &[
            (
                "no_recenter_below_threshold",
                test_floating_origin_no_recenter_below_threshold,
            ),
            (
                "recenter_above_threshold",
                test_floating_origin_recenter_above_threshold,
            ),
            (
                "accumulates_offset_with_nonzero_initial_offset",
                test_floating_origin_accumulates_offset_with_nonzero_initial_offset,
            ),
            (
                "preserves_rotation_on_recenter",
                test_floating_origin_preserves_rotation_on_recenter,
            ),
            (
                "preserves_scale_on_recenter",
                test_floating_origin_preserves_scale_on_recenter,
            ),
        ],
    );
}
