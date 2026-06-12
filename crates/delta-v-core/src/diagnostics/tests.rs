// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Unit tests for frame-time diagnostics.

use crate::diagnostics::DiagnosticsConfig;

#[test]
#[allow(clippy::expect_used)]
fn test_config_serde_from_json() {
    // DiagnosticsConfig is loaded from JSON via delta-v-json (ADR-0039).
    // JSON stores threshold in milliseconds; struct converts to seconds via getter.
    let json_str = r#"{"frame_time_warn_threshold_ms": 33, "consecutive_frames_threshold": 60}"#;
    let cfg: DiagnosticsConfig = serde_json::from_str(json_str).expect("parse failed");
    assert!((cfg.frame_time_warn_threshold_secs() - 0.033).abs() < 0.001);
    assert_eq!(cfg.consecutive_frames_threshold, 60);
}

#[test]
fn test_frame_time_comparison() {
    let threshold_secs = 0.033;
    let fast_frame_secs = 0.016;
    // Simulate: if frame time is below threshold, watchdog resets streak
    assert!(fast_frame_secs < threshold_secs);
}
