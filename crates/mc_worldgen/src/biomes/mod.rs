//! Biome classification and surface rule stratification modules.

pub mod registry;
pub mod surface_rules;

pub use registry::{BiomeCategory, BiomeId};
pub use surface_rules::{SurfaceBlock, SurfaceRuleEngine, TerracottaBands, MAX_Y, MIN_Y, SEA_LEVEL};
