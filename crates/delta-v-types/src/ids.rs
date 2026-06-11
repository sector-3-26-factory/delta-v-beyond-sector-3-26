// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Identifier types for type-safe entity and template references.

use std::fmt;
use std::ops::Deref;

/// A type-safe identifier for an entity.
///
/// This is a newtype wrapper around `Entity` that provides type safety
/// when passing entity references between systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(u64);

impl EntityId {
    /// Creates a new `EntityId` from a raw u64.
    #[must_use]
    pub const fn from_u64(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw u64 value.
    #[must_use]
    pub const fn into_u64(self) -> u64 {
        self.0
    }
}

impl Deref for EntityId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EntityId({})", self.0)
    }
}

/// A type-safe path to a template file.
///
/// Templates are stored under `assets/templates/<category>/<name>/`.
/// This type provides a structured way to reference templates.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplatePath {
    /// The category of the template (e.g., "ships", "asteroids").
    pub category: String,
    /// The name of the template (e.g., "meshy-cargo-1").
    pub name: String,
}

impl TemplatePath {
    /// Creates a new `TemplatePath` from category and name.
    pub fn new(category: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
        }
    }

    /// Returns the path relative to `assets/templates/`.
    #[must_use]
    pub fn relative_path(&self) -> String {
        format!("{}/{}/", self.category, self.name)
    }
}

impl fmt::Display for TemplatePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.category, self.name)
    }
}
