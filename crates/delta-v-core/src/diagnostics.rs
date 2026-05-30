// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Frame-time diagnostics: warns when sustained frame time exceeds a
//! configured threshold.
//!
//! See ADR-0022 (Performance instrumentation).

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use serde::Deserialize;

/// Configuration for the frame-time watchdog.
///
/// Loaded from `assets/config/diagnostics.json` during plugin build.
/// All defaults are in the JSON schema (ADR-0039).
#[derive(Resource, Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DiagnosticsConfig {
    /// Frame time threshold in milliseconds (converted to seconds internally).
    /// Must be provided by diagnostics.json.
    #[serde(rename = "frame_time_warn_threshold_ms")]
    frame_time_warn_threshold_secs: f64,
    /// Number of consecutive frames over threshold before a WARN is emitted.
    /// Must be provided by diagnostics.json.
    pub consecutive_frames_threshold: u32,
}

impl DiagnosticsConfig {
    /// Getter for frame time threshold in seconds.
    /// JSON stores it in milliseconds; this converts to seconds.
    pub fn frame_time_warn_threshold_secs(&self) -> f64 {
        self.frame_time_warn_threshold_secs / 1000.0
    }
}

// allow-default: Bevy trait bound on init_resource::<T>(); internal state only, not configuration.
#[derive(Resource, Default)]
struct OverrunStreak(u32);

/// Bevy plugin that wires `FrameTimeDiagnosticsPlugin` and the watchdog.
pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .init_resource::<OverrunStreak>()
            .add_systems(Update, frame_time_watchdog_system);
    }
}

/// Monitors frame time and emits warnings on sustained overruns.
///
/// Runs in `Update` every frame. Tracks consecutive frames exceeding the
/// threshold and emits a single WARN when the streak reaches the configured
/// limit (ADR-0022).
#[allow(clippy::needless_pass_by_value)]
fn frame_time_watchdog_system(
    diagnostics: Res<'_, DiagnosticsStore>,
    config: Res<'_, DiagnosticsConfig>,
    mut streak: ResMut<'_, OverrunStreak>,
) {
    let Some(ft) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(bevy::diagnostic::Diagnostic::smoothed)
    else {
        return;
    };

    let threshold = config.frame_time_warn_threshold_secs();
    if ft > threshold {
        streak.0 += 1;
        if streak.0 == config.consecutive_frames_threshold {
            log::warn!(
                "frame time {:.1} ms exceeded {:.1} ms threshold for {} consecutive frames",
                ft * 1000.0,
                threshold * 1000.0,
                streak.0,
            );
        }
    } else {
        streak.0 = 0;
    }
}
