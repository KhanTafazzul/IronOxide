//! Local aquifers and subterranean fluid flooding engine.
//! REA Reference: net.minecraft.world.level.levelgen.Aquifer (doc.class)
//! REA Reference: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer (doc$c.class)

pub mod barrier;
pub mod fluid;
pub mod grid;
pub mod noise_aquifer;

pub use barrier::BarrierNoiseSampler;
pub use fluid::{
    AquiferFluidType, FluidStatus, BEDROCK_CEILING_Y, LAVA_BARRIER_Y, MAX_Y, MIN_Y, SEA_LEVEL,
};
pub use grid::{
    grid_x, grid_y, grid_z, AquiferGridSampler, CellAnchor, CELL_SIZE_X, CELL_SIZE_Y, CELL_SIZE_Z,
};
pub use noise_aquifer::{Aquifer, AquiferSubstance, NoiseBasedAquifer};
