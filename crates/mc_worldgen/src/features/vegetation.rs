//! Surface ground vegetation and aquatic decoration pass.
//! REA Reference: net.minecraft.world.level.levelgen.feature.RandomPatchFeature (dxv.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.SeagrassFeature (dxc.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.KelpFeature (dwk.class)
//! REA Reference: net.minecraft.data.worldgen.placement.VegetationPlacements (ps.class)
//! REA Reference: net.minecraft.data.worldgen.placement.AquaticPlacements (po.class)

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::LegacyRandom;

/// Configuration for vegetation surface and aquatic decoration passes.
#[derive(Clone, Debug)]
pub struct VegetationConfig {
    pub surface_attempts: usize,
    pub aquatic_attempts: usize,
    pub xz_spread: i32,
    pub y_spread: i32,
}

impl Default for VegetationConfig {
    fn default() -> Self {
        Self {
            surface_attempts: 32,
            aquatic_attempts: 20,
            xz_spread: 7,
            y_spread: 3,
        }
    }
}

/// Statistics detailing the count of placed vegetation objects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VegetationPlacementStats {
    pub short_grass: usize,
    pub ferns: usize,
    pub poppies: usize,
    pub dandelions: usize,
    pub sunflowers: usize,
    pub lilypads: usize,
    pub seagrass: usize,
    pub kelp: usize,
}

impl VegetationPlacementStats {
    pub fn total(&self) -> usize {
        self.short_grass
            + self.ferns
            + self.poppies
            + self.dandelions
            + self.sunflowers
            + self.lilypads
            + self.seagrass
            + self.kelp
    }
}

/// Surface vegetation and aquatic plant placer.
pub struct VegetationFeature;

impl VegetationFeature {
    /// Deterministic 2D noise / hash function for vegetation species distribution.
    #[inline]
    pub fn local_noise(seed: i64, x: i32, z: i32) -> u32 {
        let mut h = (seed as u64)
            .wrapping_add((x as u64).wrapping_mul(3129871))
            .wrapping_add((z as u64).wrapping_mul(6182903));
        h ^= h >> 13;
        h = h.wrapping_mul(0x5bd1e9955bd1e995);
        h ^= h >> 15;
        (h & 0xFFFF_FFFF) as u32
    }

    /// Selects ground cover species based on deterministic local coordinate hashing.
    #[inline]
    pub fn pick_ground_cover(seed: i64, x: i32, z: i32) -> SurfaceBlock {
        let hash = Self::local_noise(seed, x, z);
        let roll = hash % 100;
        if roll < 55 {
            SurfaceBlock::ShortGrass
        } else if roll < 70 {
            SurfaceBlock::Fern
        } else if roll < 80 {
            SurfaceBlock::Poppy
        } else if roll < 90 {
            SurfaceBlock::Dandelion
        } else {
            SurfaceBlock::Sunflower
        }
    }

    /// Tests if a block can support land ground cover placement.
    #[inline]
    pub fn is_valid_surface_soil(block: SurfaceBlock) -> bool {
        matches!(block, SurfaceBlock::GrassBlock)
    }

    /// Tests if a block can support underwater vegetation (sand, gravel, dirt, mud).
    #[inline]
    pub fn is_valid_seabed(block: SurfaceBlock) -> bool {
        matches!(
            block,
            SurfaceBlock::Sand
                | SurfaceBlock::Gravel
                | SurfaceBlock::Dirt
                | SurfaceBlock::Mud
                | SurfaceBlock::Stone
        )
    }

    /// Executes the full surface decoration pass across a 16x16 chunk column.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `chunk_x`: Chunk column X coordinate.
    /// * `chunk_z`: Chunk column Z coordinate.
    /// * `heightmap`: Surface Y coordinate query function for given (X, Z).
    /// * `world_seed`: World seed.
    /// * `config`: Placement parameters.
    pub fn decorate_chunk(
        world: &mut dyn OreBlockAccess,
        chunk_x: i32,
        chunk_z: i32,
        heightmap: &dyn Fn(i32, i32) -> i32,
        world_seed: i64,
        config: &VegetationConfig,
    ) -> VegetationPlacementStats {
        const VEGETATION_SALT: i32 = 7291834;
        let mut rng = LegacyRandom::from_chunk_seed(world_seed, chunk_x, chunk_z, VEGETATION_SALT);
        let mut stats = VegetationPlacementStats::default();

        // 1. Surface Ground Cover Pass (Short Grass, Ferns, Poppies, Dandelions, Sunflowers)
        for _ in 0..config.surface_attempts {
            let patch_cx = chunk_x * 16 + rng.next_int_bounded(16);
            let patch_cz = chunk_z * 16 + rng.next_int_bounded(16);
            let base_y = heightmap(patch_cx, patch_cz);

            let patch_size = 6 + rng.next_int_bounded(8);
            for _ in 0..patch_size {
                let ox = patch_cx + rng.next_int_bounded(config.xz_spread * 2 + 1) - config.xz_spread;
                let oz = patch_cz + rng.next_int_bounded(config.xz_spread * 2 + 1) - config.xz_spread;
                let oy = base_y + rng.next_int_bounded(config.y_spread * 2 + 1) - config.y_spread;

                let soil = world.get_block(ox, oy - 1, oz);
                let target = world.get_block(ox, oy, oz);

                if Self::is_valid_surface_soil(soil) && target == SurfaceBlock::Air {
                    let cover = Self::pick_ground_cover(world_seed, ox, oz);
                    world.set_block(ox, oy, oz, cover);
                    match cover {
                        SurfaceBlock::ShortGrass => stats.short_grass += 1,
                        SurfaceBlock::Fern => stats.ferns += 1,
                        SurfaceBlock::Poppy => stats.poppies += 1,
                        SurfaceBlock::Dandelion => stats.dandelions += 1,
                        SurfaceBlock::Sunflower => stats.sunflowers += 1,
                        _ => {}
                    }
                }
            }
        }

        // 2. Aquatic Vegetation Pass (Lilypads, Seagrass, Kelp)
        for _ in 0..config.aquatic_attempts {
            let cx = chunk_x * 16 + rng.next_int_bounded(16);
            let cz = chunk_z * 16 + rng.next_int_bounded(16);
            let surface_y = heightmap(cx, cz);

            // A. Surface Lilypads on still water
            if world.get_block(cx, surface_y, cz) == SurfaceBlock::Water
                && world.get_block(cx, surface_y + 1, cz) == SurfaceBlock::Air
            {
                if rng.next_int_bounded(4) == 0 {
                    world.set_block(cx, surface_y + 1, cz, SurfaceBlock::LilyPad);
                    stats.lilypads += 1;
                }
            }

            // B. Underwater Seagrass and Kelp
            // Search down to find waterbed
            if world.get_block(cx, surface_y, cz) == SurfaceBlock::Water {
                let mut floor_y = surface_y;
                while floor_y > -64 && world.get_block(cx, floor_y, cz) == SurfaceBlock::Water {
                    floor_y -= 1;
                }

                let floor_block = world.get_block(cx, floor_y, cz);
                let water_depth = surface_y - floor_y;

                if Self::is_valid_seabed(floor_block) && water_depth >= 1 {
                    if water_depth >= 3 && rng.next_int_bounded(3) == 0 {
                        // Kelp column: grows upward from seabed towards surface
                        let kelp_height = 2 + rng.next_int_bounded(water_depth - 1);
                        let top_kelp_y = (floor_y + kelp_height).min(surface_y - 1);

                        for ky in (floor_y + 1)..=top_kelp_y {
                            if world.get_block(cx, ky, cz) == SurfaceBlock::Water {
                                world.set_block(cx, ky, cz, SurfaceBlock::Kelp);
                                stats.kelp += 1;
                            }
                        }
                    } else if water_depth >= 1 {
                        // Seagrass 1-block in water
                        let plant_y = floor_y + 1;
                        if world.get_block(cx, plant_y, cz) == SurfaceBlock::Water {
                            world.set_block(cx, plant_y, cz, SurfaceBlock::Seagrass);
                            stats.seagrass += 1;
                        }
                    }
                }
            }
        }

        stats
    }
}
