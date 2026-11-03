// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Collision detection real-app integration tests.

#[cfg(feature = "test")]
pub mod box_box;
#[cfg(feature = "test")]
pub mod collision_response;
#[cfg(feature = "test")]
pub mod collision_shape;
#[cfg(feature = "test")]
pub mod convex_hull;
#[cfg(feature = "test")]
pub mod distance_to_surface;
#[cfg(feature = "test")]
pub mod every_layer;
#[cfg(feature = "test")]
pub mod floating_origin;
#[cfg(feature = "test")]
pub mod force_magnitude;
#[cfg(feature = "test")]
pub mod integration;
#[cfg(feature = "test")]
pub mod normal_direction;
#[cfg(feature = "test")]
pub mod offset;
#[cfg(feature = "test")]
pub mod oriented_box;
#[cfg(feature = "test")]
pub mod penetration_depth;
#[cfg(feature = "test")]
pub mod relevance;
#[cfg(feature = "test")]
pub mod ship_bounces;
#[cfg(feature = "test")]
pub mod ship_collides;
#[cfg(feature = "test")]
pub mod sphere_box;
#[cfg(feature = "test")]
pub mod sphere_sphere;
#[cfg(feature = "test")]
pub mod two_celestial_bodies;

/// Register all collision test functions in the global registry.
///
/// Called from [`super::init_registry`] (the crate-level
/// `real_app_ITs::init_registry`).
#[cfg(feature = "test")]
pub fn init_registry() {
    box_box::init_registry();
    collision_response::init_registry();
    collision_shape::init_registry();
    convex_hull::init_registry();
    distance_to_surface::init_registry();
    every_layer::init_registry();
    floating_origin::init_registry();
    force_magnitude::init_registry();
    integration::init_registry();
    normal_direction::init_registry();
    offset::init_registry();
    oriented_box::init_registry();
    penetration_depth::init_registry();
    relevance::init_registry();
    ship_bounces::init_registry();
    ship_collides::init_registry();
    sphere_box::init_registry();
    sphere_sphere::init_registry();
    two_celestial_bodies::init_registry();
}
