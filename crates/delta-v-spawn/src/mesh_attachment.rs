// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Generic mesh attachment system for glTF loading.

use bevy::prelude::{Assets, Commands, Component, Entity, Gltf, Query, Res};

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
#[allow(clippy::missing_const_for_fn)]
pub fn attach_meshes<T: Component + PendingMesh>(
    mut _commands: Commands<'_, '_>,
    _gltf_assets: Res<'_, Assets<Gltf>>,
    _query: Query<'_, '_, (Entity, &T)>,
) {
    // TODO: Implement proper glTF scene attachment
    // This is a placeholder that compiles - actual implementation will be done
    // when the domain crates are updated to use this system.
}
