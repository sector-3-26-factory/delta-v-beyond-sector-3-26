// AGENTS: before modifying this file, read AGENTS.md at the repository root.

use serde::Deserialize;

use delta_v_types::Vec3Json;

/// A single camera definition with position, target (look-at point), and availability.
#[derive(Debug, Deserialize)]
pub struct CameraDefinition {
    /// Camera position relative to ship center (metres).
    pub position: Vec3Json,
    /// Point the camera looks at, relative to ship center (metres).
    /// Direction = normalize(target - position).
    pub target: Vec3Json,
    /// If true, this camera is physically present and accessible.
    /// If false, the position/target are computed but not available to the player.
    pub available: bool,
}

/// Camera definitions from the ship template.
///
/// All 8 cameras are required in the schema. Each has a position, target, and availability flag.
#[derive(Debug, Deserialize)]
pub struct ShipCamerasTemplate {
    /// Cockpit camera (inside the cockpit, typically front-upper-center).
    pub cockpit: CameraDefinition,
    /// Drone camera (behind and above the ship).
    pub drone: CameraDefinition,
    /// Rear view camera (behind at cockpit height).
    pub rear: CameraDefinition,
    /// Front/nose camera (forward view).
    pub front: CameraDefinition,
    /// Left side view camera.
    pub left: CameraDefinition,
    /// Right side view camera.
    pub right: CameraDefinition,
    /// Top-down view camera.
    pub top: CameraDefinition,
    /// Bottom-up view camera.
    pub bottom: CameraDefinition,
}
