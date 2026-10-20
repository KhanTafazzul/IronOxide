//! Entity Component System (ECS) architecture, AABB physics integration,
//! and goal-selector AI engine that powers all 60+ vanilla Minecraft mobs.

pub mod ai;
pub mod components;
pub mod passive;

pub use ai::*;
pub use components::*;
pub use passive::*;
