// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Generic mesh attachment system for glTF loading.

use bevy::hierarchy::BuildChildren;
use bevy::prelude::{Assets, Commands, Component, Entity, Gltf, Query, Res, SceneBundle};

/// Trait for pending mesh marker components.
///
/// Domain spawners implement this trait for their specific mesh marker type.
pub trait PendingMesh {
    /// Returns the glTF handle for this pending mesh.
    fn gltf_handle(&self) -> &bevy::asset::Handle<Gltf>;
}

/// Generic system that attaches loaded glTF scenes to entities with a pending marker.
///
/// The marker component type is generic — each domain defines its own.
/// When the glTF asset finishes loading, this system extracts all scenes from the
/// glTF and attaches them as children of the entity, then removes the pending marker.
#[allow(clippy::needless_pass_by_value)]
pub fn attach_meshes<T: Component + PendingMesh>(
    mut commands: Commands<'_, '_>,
    gltf_assets: Res<'_, Assets<Gltf>>,
    query: Query<'_, '_, (Entity, &T)>,
) {
    for (entity, pending) in &query {
        if let Some(gltf) = gltf_assets.get(pending.gltf_handle()) {
            if gltf.scenes.is_empty() {
                continue;
            }
            commands.entity(entity).with_children(|parent| {
                for scene_handle in &gltf.scenes {
                    parent.spawn(SceneBundle {
                        scene: scene_handle.clone(),
                        ..Default::default()
                    });
                }
            });
            commands.entity(entity).remove::<T>();
        }
    }
}
