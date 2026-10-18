//! Ground cover scatter pass distributing flowers and wild grass.
//! REA Reference: net.minecraft.world.level.levelgen.feature.RandomPatchFeature
//! REA Reference: net.minecraft.data.worldgen.placement.VegetationPlacements

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::LegacyRandom;

/// Ground cover distribution configuration.
#[derive(Clone, Debug)]
pub struct GroundCoverConfig {
    /// Number of scatter patch attempts per 16x16 chunk column.
    pub attempts_per_chunk: usize,
    /// Spread radius around patch center.
    pub xz_spread: i32,
    /// Vertical search range around ground height.
    pub y_spread: i32,
}

impl Default for GroundCoverConfig {
    fn default() -> Self {
        Self {
            attempts_per_chunk: 32,
            xz_spread: 7,
            y_spread: 3,
        }
    }
}

/// Ground cover scatter pass distributing Poppy, Dandelion, Short Grass, and Tall Grass.
pub struct GroundCoverPlacer;

impl GroundCoverPlacer {
    /// Selects a ground cover flower or grass block based on weighted probabilities.
    pub fn pick_cover_type(rng: &mut LegacyRandom) -> SurfaceBlock {
        let roll = rng.next_int_bounded(100);
        if roll < 70 {
            SurfaceBlock::ShortGrass
        } else if roll < 85 {
            SurfaceBlock::TallGrass
        } else if roll < 93 {
            SurfaceBlock::Poppy
        } else {
            SurfaceBlock::Dandelion
        }
    }

    /// Evaluates if a block can support ground cover placement.
    #[inline]
    pub fn is_valid_soil(block: SurfaceBlock) -> bool {
        matches!(
            block,
            SurfaceBlock::GrassBlock | SurfaceBlock::Dirt | SurfaceBlock::Podzol | SurfaceBlock::CoarseDirt
        )
    }

    /// Runs a ground cover scatter pass for a 16x16 chunk column.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `chunk_x`: Chunk X coordinate.
    /// * `chunk_z`: Chunk Z coordinate.
    /// * `heightmap`: 16x16 surface Y heightmap for the chunk column.
    /// * `world_seed`: World generation seed.
    ///
    /// Returns the number of ground cover decorations placed.
    pub fn scatter_chunk(
        world: &mut impl OreBlockAccess,
        chunk_x: i32,
        chunk_z: i32,
        heightmap: &impl Fn(i32, i32) -> i32,
        world_seed: i64,
        config: &GroundCoverConfig,
    ) -> usize {
        // Ground cover salt matching vanilla decoration step
        const GROUND_COVER_SALT: i32 = 4729103;
        let mut rng = LegacyRandom::from_chunk_seed(world_seed, chunk_x, chunk_z, GROUND_COVER_SALT);

        let mut placed_count = 0;

        for _ in 0..config.attempts_per_chunk {
            // Pick a patch center in local chunk coordinates
            let patch_cx = chunk_x * 16 + rng.next_int_bounded(16);
            let patch_cz = chunk_z * 16 + rng.next_int_bounded(16);
            let base_y = heightmap(patch_cx, patch_cz);

            // Scatter flowers/grass within patch radius
            let patch_size = 8 + rng.next_int_bounded(8);
            for _ in 0..patch_size {
                let ox = patch_cx + rng.next_int_bounded(config.xz_spread * 2 + 1) - config.xz_spread;
                let oz = patch_cz + rng.next_int_bounded(config.xz_spread * 2 + 1) - config.xz_spread;
                let oy = base_y + rng.next_int_bounded(config.y_spread * 2 + 1) - config.y_spread;

                // Ground cover requires valid soil below and Air at target position
                let soil = world.get_block(ox, oy - 1, oz);
                let target = world.get_block(ox, oy, oz);

                if Self::is_valid_soil(soil) && target == SurfaceBlock::Air {
                    let cover_type = Self::pick_cover_type(&mut rng);
                    world.set_block(ox, oy, oz, cover_type);
                    placed_count += 1;
                }
            }
        }

        placed_count
    }
}
