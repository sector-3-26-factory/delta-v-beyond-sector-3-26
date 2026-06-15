// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for the chase-camera component and system.
//!
//! See ADR-0021 (Testing strategy).

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use bevy::prelude::*;

    use crate::camera::{CameraFollow, chase_camera_system};

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    /// Builds a minimal Bevy app containing only the systems under test.
    ///
    /// Uses `MinimalPlugins` so that no window or renderer is opened.
    fn build_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, chase_camera_system);
        app
    }

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    /// After one frame the camera translation equals the ship position plus
    /// the configured offset (ship at origin, offset (0, 8, 20)).
    #[test]
    fn test_chase_camera_offset_applied_at_origin() {
        let mut app = build_test_app();

        // Spawn a fake ship at the origin.
        let ship = app.world_mut().spawn(Transform::default()).id();

        // Spawn the camera with a follow component.
        let offset = Vec3::new(0.0, 8.0, 20.0);
        app.world_mut().spawn((
            Transform::default(),
            CameraFollow {
                target: ship,
                offset,
            },
        ));

        app.update();

        // After one frame the camera should be at `offset` (ship is at origin).
        let mut cam_query = app.world_mut().query::<(&Transform, &CameraFollow)>();
        let (cam_transform, _) = cam_query
            .iter(app.world())
            .next()
            .expect("camera entity must exist");

        let expected = Vec3::new(0.0, 8.0, 20.0);
        assert!(
            (cam_transform.translation - expected).length() < 1e-4,
            "camera should be at {expected:?}, got {:?}",
            cam_transform.translation
        );
    }

    /// When the ship moves, the camera follows.
    #[test]
    fn test_chase_camera_follows_moved_ship() {
        let mut app = build_test_app();

        // Spawn ship at the origin.
        let ship = app.world_mut().spawn(Transform::default()).id();
        let offset = Vec3::new(0.0, 8.0, 20.0);
        app.world_mut().spawn((
            Transform::default(),
            CameraFollow {
                target: ship,
                offset,
            },
        ));

        // First frame — camera settles behind origin ship.
        app.update();

        // Move the ship to (10, 0, 0).
        let ship_pos = Vec3::new(10.0, 0.0, 0.0);
        app.world_mut()
            .entity_mut(ship)
            .get_mut::<Transform>()
            .expect("ship must have Transform")
            .translation = ship_pos;

        // Second frame — camera should follow.
        app.update();

        let mut cam_query = app.world_mut().query::<(&Transform, &CameraFollow)>();
        let (cam_transform, _) = cam_query
            .iter(app.world())
            .next()
            .expect("camera entity must exist");

        let expected = ship_pos + offset;
        assert!(
            (cam_transform.translation - expected).length() < 1e-3,
            "camera should track ship to {expected:?}, got {:?}",
            cam_transform.translation
        );
    }

    /// `CameraFollow` stores the target entity correctly.
    #[test]
    fn test_camera_follow_stores_target() {
        let mut app = build_test_app();
        let ship = app.world_mut().spawn(Transform::default()).id();
        let camera_entity = app
            .world_mut()
            .spawn((
                Transform::default(),
                CameraFollow {
                    target: ship,
                    offset: Vec3::new(0.0, 8.0, 20.0),
                },
            ))
            .id();

        let follow = app
            .world()
            .entity(camera_entity)
            .get::<CameraFollow>()
            .expect("CameraFollow component must exist");

        assert_eq!(
            follow.target, ship,
            "CameraFollow.target must equal the spawned ship entity"
        );
    }
}
