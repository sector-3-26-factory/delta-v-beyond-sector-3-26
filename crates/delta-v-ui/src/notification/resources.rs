// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Notification configuration and stack resource.

use bevy::prelude::*;

// allow-default: NotificationConfig is a pure Rust UI resource, not JSON-backed.
// It holds technical window layout values (pixel size, screen position, font
// size, color, animation durations, stack gap). These are not gameplay knobs in
// the sense of ADR-0014 (ship mass, weapon damage, etc.) — they control UI
// rendering placement only. Initialized via `app.init_resource`, matching the
// precedent set by `WindowConfig` in `window/spawn.rs`.
/// Default configuration for notifications.
///
/// This is a resource. Override at runtime by inserting a new value.
#[derive(Resource, Debug)]
pub struct NotificationConfig {
    /// Window size (width, height) in pixels.
    pub window_size: Vec2,
    /// Screen position for the first notification window (top-left corner).
    pub position: Vec2,
    /// Font size for notification text.
    pub font_size: bevy::prelude::FontSize,
    /// Text color.
    pub color: Color,
    /// Flicker animation configuration.
    pub animation: NotificationAnimation,
    /// Vertical gap between stacked notifications (pixels).
    pub stack_gap: f32,
}

/// Flicker animation configuration for a notification.
#[derive(Debug, Clone)]
pub struct NotificationAnimation {
    /// Duration of the flicker-in phase (seconds).
    pub flicker_in_duration: f32,
    /// Duration of the hold phase (seconds).
    pub hold_duration: f32,
    /// Duration of the flicker-out phase (seconds).
    pub flicker_out_duration: f32,
    /// Number of flicker oscillations during flicker-in/out.
    pub flicker_count: u32,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            window_size: Vec2::new(250.0, 30.0),
            position: Vec2::new(10.0, 10.0),
            font_size: bevy::prelude::FontSize::Px(16.0),
            color: Color::srgb(0.0, 1.0, 1.0),
            animation: NotificationAnimation {
                flicker_in_duration: 1.0,
                hold_duration: 2.0,
                flicker_out_duration: 1.0,
                flicker_count: 5,
            },
            stack_gap: 4.0,
        }
    }
}

/// Tracks active notifications for vertical stacking.
///
/// Each entry is `(window_entity, container_entity, y_offset)`.
/// When a notification is despawned, remaining entries shift up.
// allow-default: NotificationStack is pure runtime state tracking active
// notification entities. It is never loaded from JSON, never user-configured,
// and its only valid initial state is an empty Vec. This is not a gameplay
// value or silent fallback — it is empty initial state for a runtime tracker.
#[derive(Resource, Debug, Default)]
pub struct NotificationStack {
    /// Active notifications: (window entity, container entity, y-offset).
    pub entries: Vec<(Entity, Entity, f32)>,
}
