//! World generation features and placement decorators.
//! REA Reference: net.minecraft.world.level.levelgen.feature (dru.class)
//! REA Reference: net.minecraft.world.level.levelgen.placement (dyn.class)

pub mod ores;
pub mod trees;
pub mod vegetation;

pub use ores::*;
pub use trees::*;
pub use vegetation::*;
