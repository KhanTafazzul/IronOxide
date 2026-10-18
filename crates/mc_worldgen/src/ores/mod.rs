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
    coal_lower_distribution, coal_upper_distribution, copper_dripstone_distribution,
    copper_regular_distribution, diamond_distribution, gold_badlands_extra_distribution,
    gold_regular_distribution, gold_regular_triangular_distribution, iron_middle_distribution,
    iron_small_distribution, iron_upper_distribution, lapis_buried_distribution,
    lapis_regular_distribution, redstone_lower_distribution, redstone_regular_distribution,
    HeightDistribution,
};
pub use vein::{OreGenerator, OreVeinConfig};
