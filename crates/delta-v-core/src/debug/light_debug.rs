// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Debug system to query and log all light sources in the scene.

use crate::state::AppState;
use bevy::light::{AmbientLight, DirectionalLight, PointLight, SpotLight};
use bevy::prelude::*;

/// System that logs all light sources in the scene.
/// Run this after the scene is fully loaded (e.g., in `Update` with a one-shot condition).
pub fn debug_log_all_lights(
    dir_lights: Query<'_, '_, (Entity, &DirectionalLight, Option<&Transform>, Option<&Name>)>,
    point_lights: Query<'_, '_, (Entity, &PointLight, Option<&Transform>, Option<&Name>)>,
    spot_lights: Query<'_, '_, (Entity, &SpotLight, Option<&Transform>, Option<&Name>)>,
    ambient_lights: Query<'_, '_, (Entity, &AmbientLight, Option<&Name>)>,
) {
    tracing::info!("=== LIGHT SOURCES IN SCENE ===");

    for (entity, light, transform, name) in dir_lights.iter() {
        let name_str = name.map_or("unnamed", Name::as_str);
        let pos = transform.map_or(Vec3::ZERO, |t| t.translation);
        let rot = transform.map_or(Quat::IDENTITY, |t| t.rotation);
        tracing::info!(
            "DirectionalLight: entity={:?}, name={}, illuminance={:.1} lux, color={:?}, pos={:?}, rot={:?}",
            entity,
            name_str,
            light.illuminance,
            light.color,
            pos,
            rot
        );
    }

    for (entity, light, transform, name) in point_lights.iter() {
        let name_str = name.map_or("unnamed", Name::as_str);
        let pos = transform.map_or(Vec3::ZERO, |t| t.translation);
        tracing::info!(
            "PointLight: entity={:?}, name={}, intensity={:.1} lm, color={:?}, pos={:?}, range={:.1} m",
            entity,
            name_str,
            light.intensity,
            light.color,
            pos,
            light.range
        );
    }

    for (entity, light, transform, name) in spot_lights.iter() {
        let name_str = name.map_or("unnamed", Name::as_str);
        let pos = transform.map_or(Vec3::ZERO, |t| t.translation);
        let rot = transform.map_or(Quat::IDENTITY, |t| t.rotation);
        tracing::info!(
            "SpotLight: entity={:?}, name={}, intensity={:.1} lm, color={:?}, pos={:?}, rot={:?}, inner_angle={:.2}, outer_angle={:.2}",
            entity,
            name_str,
            light.intensity,
            light.color,
            pos,
            rot,
            light.inner_angle,
            light.outer_angle
        );
    }

    for (entity, light, name) in ambient_lights.iter() {
        let name_str = name.map_or("unnamed", Name::as_str);
        tracing::info!(
            "AmbientLight: entity={:?}, name={}, color={:?}, brightness={:.1}",
            entity,
            name_str,
            light.color,
            light.brightness
        );
    }

    tracing::info!("=== END LIGHT SOURCES ===");
}

/// One-shot system to log lights once after scene load.
/// Add to `Update` schedule with `.run_once()` or similar.
pub fn debug_log_lights_once(
    mut commands: Commands<'_, '_>,
    dir_lights: Query<'_, '_, (Entity, &DirectionalLight, Option<&Transform>, Option<&Name>)>,
    point_lights: Query<'_, '_, (Entity, &PointLight, Option<&Transform>, Option<&Name>)>,
    spot_lights: Query<'_, '_, (Entity, &SpotLight, Option<&Transform>, Option<&Name>)>,
    ambient_lights: Query<'_, '_, (Entity, &AmbientLight, Option<&Name>)>,
) {
    debug_log_all_lights(dir_lights, point_lights, spot_lights, ambient_lights);
    // Remove self after running once
    commands.remove_resource::<RanOnce>();
}

// allow-default: debug-only one-shot marker resource, not JSON-backed
#[derive(Resource, Default)]
struct RanOnce;

/// Plugin to add the light debug system.
/// Enable with `--features dev` or manually add to app.
pub struct LightDebugPlugin;

impl Plugin for LightDebugPlugin {
    fn build(&self, app: &mut App) {
        // Run on OnEnter(InGame) so the world is fully spawned
        app.add_systems(OnEnter(AppState::InGame), debug_log_all_lights);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_system_compiles() {
        let mut app = App::new();
        app.add_systems(Update, debug_log_all_lights);
        // Just verify it compiles and can be added
    }
}
