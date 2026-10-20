//! Complete end-to-end 6-stage chunk generation pipeline:
//! Noise -> Carvers -> Surface -> Ores -> Foliage -> Structures.

use crate::biomes::{BiomeId, SurfaceBlock};
use crate::features::ores::OreGenerator;
pub use crate::seed_pipeline::terrain::{OverworldGenerator, MAX_Y, MIN_Y, SEA_LEVEL};
use crate::structures::{BuriedTreasureGenerator, LegacyRandom, MineshaftGenerator, MineshaftType, StructureType};
use crate::surface::SurfaceSystem;
use crate::vegetation::{TreePlacer, TreeType};

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = (MAX_Y - MIN_Y) as usize; // 384
pub const CHUNK_VOLUME: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_WIDTH; // 98,304

/// Complete 16x384x16 chunk column buffer (Y: -64 to 320)
#[derive(Clone, Debug)]
pub struct ChunkColumnBuffer {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub biome: BiomeId,
    pub blocks: Box<[SurfaceBlock; CHUNK_VOLUME]>,
}

impl ChunkColumnBuffer {
    pub fn new(chunk_x: i32, chunk_z: i32, biome: BiomeId) -> Self {
        Self {
            chunk_x,
            chunk_z,
            biome,
            blocks: Box::new([SurfaceBlock::Air; CHUNK_VOLUME]),
        }
    }

    #[inline(always)]
    pub fn block_index(lx: usize, ly: usize, lz: usize) -> usize {
        ly * 256 + lz * 16 + lx
    }

    #[inline(always)]
    pub fn get_local(&self, lx: usize, ly: usize, lz: usize) -> SurfaceBlock {
        if lx < 16 && ly < CHUNK_HEIGHT && lz < 16 {
            self.blocks[Self::block_index(lx, ly, lz)]
        } else {
            SurfaceBlock::Air
        }
    }

    #[inline(always)]
    pub fn set_local(&mut self, lx: usize, ly: usize, lz: usize, block: SurfaceBlock) {
        if lx < 16 && ly < CHUNK_HEIGHT && lz < 16 {
            let idx = Self::block_index(lx, ly, lz);
            self.blocks[idx] = block;
        }
    }
}

impl crate::features::ores::OreBlockAccess for ChunkColumnBuffer {
    fn get_block(&self, x: i32, y: i32, z: i32) -> SurfaceBlock {
        let lx = x - self.chunk_x * 16;
        let ly = y - MIN_Y; // y - (-64) = y + 64
        let lz = z - self.chunk_z * 16;
        if lx >= 0 && lx < 16 && ly >= 0 && ly < CHUNK_HEIGHT as i32 && lz >= 0 && lz < 16 {
            self.get_local(lx as usize, ly as usize, lz as usize)
        } else {
            SurfaceBlock::Air
        }
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, block: SurfaceBlock) {
        let lx = x - self.chunk_x * 16;
        let ly = y - MIN_Y;
        let lz = z - self.chunk_z * 16;
        if lx >= 0 && lx < 16 && ly >= 0 && ly < CHUNK_HEIGHT as i32 && lz >= 0 && lz < 16 {
            self.set_local(lx as usize, ly as usize, lz as usize, block);
        }
    }
}

impl crate::ores::OreBlockAccess for ChunkColumnBuffer {
    fn get_block(&self, x: i32, y: i32, z: i32) -> SurfaceBlock {
        crate::features::ores::OreBlockAccess::get_block(self, x, y, z)
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, block: SurfaceBlock) {
        crate::features::ores::OreBlockAccess::set_block(self, x, y, z, block)
    }
}

/// Dispatches the complete 6-stage generation pipeline for a 16x384x16 chunk column:
/// Stage 1: Noise (3D cell-interpolated density)
/// Stage 2: Carvers (Cheese, Spaghetti, Noodle caves and ravines)
/// Stage 3: Surface (Biome strata, grass, dirt, sand, sandstone, terracotta, ocean water)
/// Stage 4: Ores (Vanilla trapezoidal ore blobs: Coal, Iron, Copper, Gold, Redstone, Lapis, Diamond)
/// Stage 5: Foliage (Procedural trees & vegetation placement based on biome climate)
/// Stage 6: Structures (Mineshafts, Buried Treasure, etc.)
pub fn generate_chunk_column_pipeline(
    generator: &OverworldGenerator,
    chunk_x: i32,
    chunk_z: i32,
) -> ChunkColumnBuffer {
    // Determine dominant chunk biome at center (8, 64, 8)
    let center_x = chunk_x * 16 + 8;
    let center_z = chunk_z * 16 + 8;
    let biome_str = generator.sample_biome(center_x >> 2, 16, center_z >> 2);
    let chunk_biome = BiomeId::from_name(biome_str).unwrap_or(BiomeId::Plains);

    let mut buffer = ChunkColumnBuffer::new(chunk_x, chunk_z, chunk_biome);

    // ------------------------------------------------------------------------
    // STAGE 0 & 1: NOISE DENSITY & CARVERS
    // ------------------------------------------------------------------------
    // Evaluate 16x16x16 section densities (-4 to 19, i.e. Y = -64 to 320).
    // calculate_density internally merges 3D terrain noise with cave carver routers.
    for sec_y in -4..=19 {
        let sec_density = generator.generate_section_density(chunk_x, sec_y, chunk_z);
        for ly in 0..16 {
            let world_y = sec_y * 16 + ly as i32;
            let buf_y = (world_y - MIN_Y) as usize;

            for lz in 0..16 {
                for lx in 0..16 {
                    let is_solid = sec_density.is_solid(lx, ly, lz);
                    if is_solid {
                        let block = if world_y == -64 {
                            SurfaceBlock::Bedrock
                        } else if world_y < 0 {
                            SurfaceBlock::Deepslate
                        } else {
                            SurfaceBlock::Stone
                        };
                        buffer.set_local(lx, buf_y, lz, block);
                    } else {
                        buffer.set_local(lx, buf_y, lz, SurfaceBlock::Air);
                    }
                }
            }
        }
    }

    // Add bedrock floor roughness (Y = -64 to -60)
    let mut bedrock_rng = LegacyRandom::from_chunk_seed(generator.seed, chunk_x, chunk_z, 8888);
    for lz in 0..16 {
        for lx in 0..16 {
            buffer.set_local(lx, 0, lz, SurfaceBlock::Bedrock);
            for y_off in 1..5 {
                if bedrock_rng.next_int_bounded(5) > y_off {
                    buffer.set_local(lx, y_off as usize, lz, SurfaceBlock::Bedrock);
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // STAGE 2: SURFACE RULES & BIOME STRATA
    // ------------------------------------------------------------------------
    let surface_system = SurfaceSystem::new(generator.seed as u64);

    for lz in 0..16 {
        let world_z = chunk_z * 16 + lz as i32;
        for lx in 0..16 {
            let world_x = chunk_x * 16 + lx as i32;

            // Find surface_y (highest solid block)
            let mut surface_y = -64;
            for y in (-64..=319).rev() {
                let buf_y = (y - MIN_Y) as usize;
                let b = buffer.get_local(lx, buf_y, lz);
                if b != SurfaceBlock::Air && b != SurfaceBlock::Water {
                    surface_y = y;
                    break;
                }
            }

            let col_biome_str = generator.sample_biome(world_x >> 2, surface_y >> 2, world_z >> 2);
            let col_biome = BiomeId::from_name(col_biome_str).unwrap_or(chunk_biome);

            // Ocean / river water column filling above surface up to sea level (Y=63)
            if surface_y < SEA_LEVEL {
                for y in (surface_y + 1)..=SEA_LEVEL {
                    let w_buf_y = (y - MIN_Y) as usize;
                    buffer.set_local(lx, w_buf_y, lz, SurfaceBlock::Water);
                }
            }

            // Apply surface rules from surface_y down into terrain (topsoil layers)
            let eval_bottom = (surface_y - 8).max(-64);
            for y in eval_bottom..=surface_y {
                let current_b = buffer.get_local(lx, (y - MIN_Y) as usize, lz);
                if current_b == SurfaceBlock::Bedrock || current_b == SurfaceBlock::Air {
                    continue;
                }
                let surf_b = surface_system.get_surface_block(world_x, y, world_z, surface_y, col_biome, 0.0);
                if surf_b != SurfaceBlock::Air {
                    buffer.set_local(lx, (y - MIN_Y) as usize, lz, surf_b);
                }
            }

            // Snowy biomes: place thin snow layer atop solid surface
            if col_biome.is_snowy() && surface_y >= SEA_LEVEL {
                let above_buf_y = (surface_y + 1 - MIN_Y) as usize;
                if above_buf_y < CHUNK_HEIGHT && buffer.get_local(lx, above_buf_y, lz) == SurfaceBlock::Air {
                    buffer.set_local(lx, above_buf_y, lz, SurfaceBlock::SnowLayer);
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // STAGE 3: ORES (17 vanilla ore passes with height trapezoids)
    // ------------------------------------------------------------------------
    let ore_gen = OreGenerator::vanilla_overworld();
    ore_gen.generate_chunk(&mut buffer, chunk_x, chunk_z, chunk_biome, generator.seed);

    // ------------------------------------------------------------------------
    // STAGE 4: FOLIAGE & VEGETATION
    // ------------------------------------------------------------------------
    let (tree_type, tree_attempts) = match chunk_biome {
        BiomeId::Forest | BiomeId::FlowerForest => (TreeType::Oak, 8),
        BiomeId::BirchForest | BiomeId::OldGrowthBirchForest => (TreeType::Birch, 8),
        BiomeId::Taiga | BiomeId::OldGrowthPineTaiga | BiomeId::OldGrowthSpruceTaiga | BiomeId::SnowyTaiga => (TreeType::Spruce, 8),
        BiomeId::DarkForest => (TreeType::DarkOak, 10),
        BiomeId::Jungle | BiomeId::SparseJungle => (TreeType::MegaJungle, 6),
        BiomeId::Plains | BiomeId::SunflowerPlains | BiomeId::Meadow => (TreeType::Oak, 1),
        _ => (TreeType::Oak, 0),
    };

    if tree_attempts > 0 {
        let mut veg_rng = LegacyRandom::from_chunk_seed(generator.seed, chunk_x, chunk_z, 54321);
        for _ in 0..tree_attempts {
            let lx = 2 + veg_rng.next_int_bounded(12);
            let lz = 2 + veg_rng.next_int_bounded(12);
            let world_x = chunk_x * 16 + lx;
            let world_z = chunk_z * 16 + lz;

            // Find surface_y
            let mut surface_y = -64;
            for y in (-64..=319).rev() {
                let buf_y = (y - MIN_Y) as usize;
                let b = buffer.get_local(lx as usize, buf_y, lz as usize);
                if b != SurfaceBlock::Air && b != SurfaceBlock::Water {
                    surface_y = y;
                    break;
                }
            }

            if surface_y >= SEA_LEVEL {
                let soil = buffer.get_local(lx as usize, (surface_y - MIN_Y) as usize, lz as usize);
                if TreePlacer::is_valid_soil(soil) {
                    TreePlacer::place_tree(&mut buffer, (world_x, surface_y + 1, world_z), tree_type, &mut veg_rng);
                }
            }
        }
    }

    // Ground vegetation for green biomes
    if matches!(chunk_biome, BiomeId::Plains | BiomeId::SunflowerPlains | BiomeId::Forest | BiomeId::Meadow) {
        let mut ground_rng = LegacyRandom::from_chunk_seed(generator.seed, chunk_x, chunk_z, 99999);
        for _ in 0..16 {
            let lx = ground_rng.next_int_bounded(16) as usize;
            let lz = ground_rng.next_int_bounded(16) as usize;
            for y in (SEA_LEVEL..=200).rev() {
                let buf_y = (y - MIN_Y) as usize;
                if buffer.get_local(lx, buf_y, lz) == SurfaceBlock::GrassBlock {
                    if buf_y + 1 < CHUNK_HEIGHT && buffer.get_local(lx, buf_y + 1, lz) == SurfaceBlock::Air {
                        let plant = match ground_rng.next_int_bounded(6) {
                            0 => SurfaceBlock::Poppy,
                            1 => SurfaceBlock::Dandelion,
                            _ => SurfaceBlock::ShortGrass,
                        };
                        buffer.set_local(lx, buf_y + 1, lz, plant);
                    }
                    break;
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // STAGE 5: STRUCTURES
    // ------------------------------------------------------------------------
    let structures = generator.get_structures_in_chunk(chunk_x, chunk_z);
    for s in structures {
        match s.structure_type {
            StructureType::Mineshaft => {
                let center_pos = (chunk_x * 16 + 8, -20, chunk_z * 16 + 8);
                MineshaftGenerator::generate(&mut buffer, center_pos, MineshaftType::Normal, generator.seed);
            }
            StructureType::BuriedTreasure => {
                let mut surf_y = 62;
                for y in (50..=80).rev() {
                    let buf_y = (y - MIN_Y) as usize;
                    let b = buffer.get_local(8, buf_y, 8);
                    if b == SurfaceBlock::Sand || b == SurfaceBlock::Gravel {
                        surf_y = y;
                        break;
                    }
                }
                BuriedTreasureGenerator::generate(&mut buffer, (chunk_x * 16 + 8, surf_y, chunk_z * 16 + 8));
            }
            _ => {}
        }
    }

    buffer
}
