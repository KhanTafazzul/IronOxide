//! Ore vein generation, ellipsoid blob clustering, and vertical trapezoid distribution engine.
//! REA Reference: net.minecraft.world.level.levelgen.feature.OreFeature (dsp.class)
//! REA Reference: net.minecraft.world.level.levelgen.placement.HeightRangePlacement (dyf.class)
//! REA Reference: net.minecraft.data.worldgen.features.OreFeatures (qm.class)
//! REA Reference: net.minecraft.data.worldgen.placement.OrePlacements (qx.class)

pub mod blob;
pub mod distribution;
pub mod vein;

pub use blob::{ChunkOreBuffer, OreBlobPlacer, OreBlockAccess, OreType};
pub use distribution::{
    coal_lower_distribution, coal_upper_distribution, diamond_distribution,
    gold_badlands_extra_distribution, gold_regular_distribution, iron_middle_distribution,
    iron_small_distribution, iron_upper_distribution, redstone_lower_distribution,
    redstone_regular_distribution, HeightDistribution,
};
pub use vein::{OreGenerator, OreVeinConfig};
