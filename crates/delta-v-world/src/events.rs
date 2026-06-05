// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Events emitted by the world system for entity spawning.
//!
//! [`SpawnEntity`] is now defined in `delta-v-core` (see
//! [`delta_v_core::events`]) so that all domain crates can reference it
//! without creating domain-to-domain dependencies. This module re-exports
//! it for backward compatibility.
//!
//! See ADR-0005 (plugin architecture) and ADR-0038 (entity template system).

// Re-export SpawnEntity from delta-v-core to avoid domain-to-domain dependencies.
pub use delta_v_core::SpawnEntity;
