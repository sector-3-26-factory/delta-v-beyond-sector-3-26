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

/// Tracks frame-time watchdog state during gameplay.
///
/// **Why `saw_good_frame`?**
///
/// When entering `InGame`, the first frame-time measurement often includes
/// GPU shader compilation and other one-time startup costs, resulting in a
/// very high frame time (~16+ seconds). This is not representative of actual
/// gameplay performance and should not trigger performance warnings.
///
/// The watchdog only starts counting overrun frames *after* observing at
/// least one frame with acceptable frame time. This ensures that:
/// - Initial GPU setup and shader compilation don't trigger false alarms
/// - Warnings reflect sustained gameplay performance issues, not startup hiccups
/// - The player sees meaningful performance data once the game is running
///
/// See ADR-0022 (Performance instrumentation) for rationale.
// allow-default: Bevy trait bound on init_resource::<T>(); internal runtime state only, not configuration.
#[derive(Resource, Default)]
struct DiagnosticsState {
    /// Number of consecutive frames exceeding the threshold.
    overrun_streak: u32,
    /// Whether we've observed at least one frame within acceptable time.
    /// Prevents startup GPU-compilation frames from triggering false warnings.
    saw_good_frame: bool,
}

/// Bevy plugin that registers the frame-time watchdog.
///
/// The `FrameTimeDiagnosticsPlugin` itself is registered on enter of
/// `InGame` state, so frame-time measurement only happens during active
/// gameplay (not during loading, menus, or pauses).
pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        use crate::AppState;
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<DiagnosticsState>()
            .add_systems(OnEnter(AppState::InGame), reset_frame_time_diagnostics)
            .add_systems(
                Update,
                frame_time_watchdog_system.run_if(in_state(AppState::InGame)),
            );
    }
}

/// Resets frame-time diagnostics when entering `InGame` state.
///
/// Resets the overrun streak and `saw_good_frame` flag so that loading-time
/// frame stutters do not trigger false performance warnings (ADR-0022).
fn reset_frame_time_diagnostics(mut state: ResMut<'_, DiagnosticsState>) {
    state.overrun_streak = 0;
    state.saw_good_frame = false;
}

/// Monitors frame time and emits warnings on sustained overruns.
///
/// Runs in `Update` during `InGame` state only. Tracks consecutive frames
/// exceeding the threshold and emits a single WARN when the streak reaches
/// the configured limit.
///
/// **Warmup behavior:** The watchdog only starts counting overrun frames after
/// observing at least one frame with acceptable frame time. This prevents
/// false alarms from GPU shader compilation on first entry to `InGame`.
/// Once `saw_good_frame` is true, any overrun resets the streak back to zero.
///
/// See ADR-0022 (Performance instrumentation) for rationale.
#[allow(clippy::needless_pass_by_value)]
fn frame_time_watchdog_system(
    diagnostics: Res<'_, DiagnosticsStore>,
    config: Res<'_, DiagnosticsConfig>,
    mut state: ResMut<'_, DiagnosticsState>,
) {
    let Some(ft) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(bevy::diagnostic::Diagnostic::smoothed)
    else {
        return;
    };

    let threshold = config.frame_time_warn_threshold_secs();

    // Track whether we've seen a good frame (for warmup).
    if ft <= threshold {
        state.saw_good_frame = true;
    }

    // Only count overruns after seeing at least one good frame.
    if state.saw_good_frame {
        if ft > threshold {
            state.overrun_streak += 1;
            if state.overrun_streak == config.consecutive_frames_threshold {
                log::warn!(
                    "frame time {:.1} ms exceeded {:.1} ms threshold for {} consecutive frames",
                    ft * 1000.0,
                    threshold * 1000.0,
                    state.overrun_streak,
                );
            }
        } else {
            state.overrun_streak = 0;
        }
    }
}
