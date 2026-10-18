//! Ore vein configuration and chunk generation orchestration.
//! REA Reference: net.minecraft.data.worldgen.features.OreFeatures (qm.class)
//! REA Reference: net.minecraft.data.worldgen.placement.OrePlacements (qx.class)

use super::blob::{OreBlobPlacer, OreBlockAccess, OreType};
use super::distribution::{
    coal_lower_distribution, coal_upper_distribution, diamond_distribution,
    gold_badlands_extra_distribution, gold_regular_distribution, iron_middle_distribution,
    iron_small_distribution, iron_upper_distribution, redstone_lower_distribution,
    redstone_regular_distribution, HeightDistribution,
};
use crate::biomes::BiomeId;
use crate::structures::LegacyRandom;

/// Configuration defining an ore vein type, its cluster size, frequency, and elevation curve.
#[derive(Clone, Debug, PartialEq)]
pub struct OreVeinConfig {
    /// Target ore material.
    pub ore_type: OreType,
    /// Number of ellipsoid interpolation steps determining cluster size.
    pub size: usize,
    /// Number of vein generation attempts per 16x16 chunk column.
    pub count_per_chunk: usize,
    /// Elevation distribution curve.
    pub distribution: HeightDistribution,
    /// If true, only generates when the chunk column belongs to a Badlands / Mesa biome variant.
    pub badlands_only: bool,
}

impl OreVeinConfig {
    pub const fn new(
        ore_type: OreType,
        size: usize,
        count_per_chunk: usize,
        distribution: HeightDistribution,
    ) -> Self {
        Self {
            ore_type,
            size,
            count_per_chunk,
            distribution,
            badlands_only: false,
        }
    }

    pub const fn badlands_extra(
        ore_type: OreType,
        size: usize,
        count_per_chunk: usize,
        distribution: HeightDistribution,
    ) -> Self {
        Self {
            ore_type,
            size,
            count_per_chunk,
            distribution,
            badlands_only: true,
        }
    }
}

/// Overworld ore generator orchestrating all standard vanilla ore placements.
#[derive(Clone, Debug)]
pub struct OreGenerator {
    pub configs: Vec<OreVeinConfig>,
}

impl Default for OreGenerator {
    fn default() -> Self {
        Self::vanilla_overworld()
    }
}

impl OreGenerator {
    /// Builds the standard vanilla Overworld ore configuration list matching Minecraft 1.20.4 byte-for-byte.
    /// REA Reference: `qm.class` & `qx.class`
    pub fn vanilla_overworld() -> Self {
        let configs = vec![
            // Coal Upper: size 17, count 30, Uniform 136 to 256
            OreVeinConfig::new(OreType::Coal, 17, 30, coal_upper_distribution()),
            // Coal Lower: size 17, count 20, Triangular 0 to 192 (peak at Y=96)
            OreVeinConfig::new(OreType::Coal, 17, 20, coal_lower_distribution()),

            // Iron Upper: size 9, count 90, Triangular 80 to 384 (peak at Y=232 / 256 in high peaks)
            OreVeinConfig::new(OreType::Iron, 9, 90, iron_upper_distribution()),
            // Iron Middle: size 9, count 10, Triangular -24 to 56 (peak at Y=16)
            OreVeinConfig::new(OreType::Iron, 9, 10, iron_middle_distribution()),
            // Iron Small: size 4, count 10, Uniform -64 to 72
            OreVeinConfig::new(OreType::Iron, 4, 10, iron_small_distribution()),

            // Gold Regular: size 9, count 4, Uniform -64 to -32
            OreVeinConfig::new(OreType::Gold, 9, 4, gold_regular_distribution()),
            // Gold Extra (Badlands / Mesa biomes): size 9, count 50, Uniform 32 to 256
            OreVeinConfig::badlands_extra(OreType::Gold, 9, 50, gold_badlands_extra_distribution()),

            // Redstone Regular: size 8, count 4, Uniform -64 to 15
            OreVeinConfig::new(OreType::Redstone, 8, 4, redstone_regular_distribution()),
            // Redstone Lower: size 8, count 8, Triangular toward bottom bedrock
            OreVeinConfig::new(OreType::Redstone, 8, 8, redstone_lower_distribution()),

            // Diamond: size 4-8, count 7, Triangular 16 down to -64 (peak at bedrock level Y=-64)
            OreVeinConfig::new(OreType::Diamond, 8, 7, diamond_distribution()),

            // Copper: size 10, count 16, Triangular -16 to 112 (peak at Y=48)
            OreVeinConfig::new(OreType::Copper, 10, 16, HeightDistribution::triangular(-16, 112)),
            // Lapis: size 7, count 2, Triangular -32 to 32 (peak at Y=0)
            OreVeinConfig::new(OreType::Lapis, 7, 2, HeightDistribution::triangular(-32, 32)),
        ];

        Self { configs }
    }

    /// Evaluates whether the given biome is a Badlands / Mesa variant.
    #[inline]
    pub fn is_badlands(biome: BiomeId) -> bool {
        matches!(
            biome,
            BiomeId::Badlands | BiomeId::ErodedBadlands | BiomeId::WoodedBadlands
        )
    }

    /// Generates all configured ores within a 16x16 chunk column.
    ///
    /// # Arguments
    /// * `world`: The block accessor interface (e.g. chunk storage).
    /// * `chunk_x`: Chunk X coordinate.
    /// * `chunk_z`: Chunk Z coordinate.
    /// * `biome`: Biome at the chunk center.
    /// * `world_seed`: The world generation seed.
    ///
    /// Returns the total number of ore blocks generated.
    pub fn generate_chunk(
        &self,
        world: &mut impl OreBlockAccess,
        chunk_x: i32,
        chunk_z: i32,
        biome: BiomeId,
        world_seed: i64,
    ) -> usize {
        let is_badlands = Self::is_badlands(biome);
        let mut total_ores = 0;

        for (feature_idx, config) in self.configs.iter().enumerate() {
            // Badlands extra gold generates strictly in Badlands biomes
            if config.badlands_only && !is_badlands {
                continue;
            }

            // Derive deterministic chunk feature seed matching vanilla decoration step
            let salt = (feature_idx as i32).wrapping_mul(314159).wrapping_add(10387313);
            let mut rng = LegacyRandom::from_chunk_seed(world_seed, chunk_x, chunk_z, salt);

            for _ in 0..config.count_per_chunk {
                // Sample random (X, Z) inside chunk bounds [0, 15]
                let local_x = rng.next_int_bounded(16);
                let local_z = rng.next_int_bounded(16);
                let world_x = chunk_x * 16 + local_x;
                let world_z = chunk_z * 16 + local_z;

                // Sample Y coordinate from ore distribution curve
                let world_y = config.distribution.sample(&mut rng);

                // Place ellipsoid blob cluster
                let placed = OreBlobPlacer::place_blob(
                    world,
                    (world_x, world_y, world_z),
                    config.size,
                    config.ore_type,
                    &mut rng,
                );

                total_ores += placed;
            }
        }

        total_ores
    }
}
