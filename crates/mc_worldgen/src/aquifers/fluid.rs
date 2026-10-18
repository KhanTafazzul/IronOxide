//! Aquifer fluid definitions and elevation constants.
//! REA Reference: net.minecraft.world.level.levelgen.Aquifer$FluidStatus (doc$b.class)
//! REA Reference: net.minecraft.world.level.levelgen.Aquifer$FluidPicker (doc$a.class)

use crate::biomes::SurfaceBlock;

/// Standard sea level in Minecraft 1.20.4.
pub const SEA_LEVEL: i32 = 63;

/// Deep underground elevation where water transitions to subterranean lava oceans/lakes.
/// REA Reference: NoiseBasedAquifer.computeFluidType ($Y < -54$ / Lava Level)
pub const LAVA_BARRIER_Y: i32 = -54;

/// Bedrock ceiling boundary where terrain is strictly unbreakable.
pub const BEDROCK_CEILING_Y: i32 = -60;

/// Lowest world boundary in Minecraft 1.20.4.
pub const MIN_Y: i32 = -64;

/// Highest world boundary in Minecraft 1.20.4.
pub const MAX_Y: i32 = 320;

/// Fluid type occupying subterranean aquifer pockets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AquiferFluidType {
    /// Dry, open air cave chamber
    Air,
    /// Flooded cavern filled with water
    Water,
    /// Deep cavern floor filled with molten lava
    Lava,
}

impl AquiferFluidType {
    /// Converts aquifer fluid type to surface block representation.
    #[inline]
    pub fn to_surface_block(self) -> SurfaceBlock {
        match self {
            AquiferFluidType::Air => SurfaceBlock::Air,
            AquiferFluidType::Water => SurfaceBlock::Water,
            AquiferFluidType::Lava => SurfaceBlock::Air, // SurfaceBlock represents solids/water, Lava can map to Air or specialized state
        }
    }
}

/// Status of an aquifer cell specifying fluid surface level and fluid block type.
/// REA Reference: net.minecraft.world.level.levelgen.Aquifer$FluidStatus (doc$b.class)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidStatus {
    /// Fluid surface level in world Y coordinates.
    pub fluid_level: i32,
    /// Fluid block type (Water, Lava, or Air).
    pub fluid_type: AquiferFluidType,
}

impl FluidStatus {
    /// Creates a new FluidStatus with specified level and fluid type.
    #[inline]
    pub const fn new(fluid_level: i32, fluid_type: AquiferFluidType) -> Self {
        Self {
            fluid_level,
            fluid_type,
        }
    }

    /// Evaluates fluid type at a specific Y elevation.
    /// If Y is below or equal to fluid_level, returns the fluid type; otherwise returns Air.
    #[inline]
    pub fn at_elevation(&self, y: i32) -> AquiferFluidType {
        if y <= self.fluid_level {
            // Below -54, fluid strictly switches to Lava
            if y < LAVA_BARRIER_Y {
                AquiferFluidType::Lava
            } else {
                self.fluid_type
            }
        } else {
            AquiferFluidType::Air
        }
    }
}
