//! Entity Component System (ECS) architecture, AABB physics integration,
//! and goal-selector AI engine that powers all 60+ vanilla Minecraft mobs.

pub mod ai;
pub mod aquatic;
pub mod components;
pub mod hostile;
pub mod passive;
pub mod villagers;

pub use ai::*;
pub use aquatic::*;
pub use components::*;
pub use hostile::*;
pub use passive::*;
pub use villagers::*;



