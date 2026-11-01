//! 1:1 Seed-for-seed terrain heightmap, cell-interpolated density, and biome generator.

use super::climate::{ClimateSampler, OverworldBiomeRegistry};
use super::density::{BlendedNoise, CaveNoiseRouter};
use crate::carvers::CaveCarverRouter;
use crate::structures::{StructureLocator, StructureRoot};

pub const SEA_LEVEL: i32 = 63;
pub const MIN_Y: i32 = -64;
pub const MAX_Y: i32 = 320;

/// A 16x16x16 chunk section containing trilinearly interpolated density values (-64 to 320 Y).
#[derive(Clone, Debug)]
pub struct ChunkSectionDensity {
    pub chunk_x: i32,
    pub section_y: i32, // -4 to 19 (-64 to 320)
    pub chunk_z: i32,
    pub densities: Box<[f64; 16 * 16 * 16]>,
}

impl ChunkSectionDensity {
    /// Gets the density value at local coordinate (0..16, 0..16, 0..16).
    #[inline]
    pub fn get(&self, lx: usize, ly: usize, lz: usize) -> f64 {
        self.densities[ly * 256 + lz * 16 + lx]
    }

    /// Returns true if the block at local coordinates is solid terrain.
    #[inline]
    pub fn is_solid(&self, lx: usize, ly: usize, lz: usize) -> bool {
        self.get(lx, ly, lz) > 0.0
    }
}

/// Overworld generator matching vanilla seed-for-seed terrain, cave carvers, and biome arrays.
pub struct OverworldGenerator {
    pub seed: i64,
    pub climate_sampler: ClimateSampler,
    pub biome_registry: OverworldBiomeRegistry,
    pub blended_noise: BlendedNoise,
    pub cave_router: CaveNoiseRouter,
    pub carver_router: CaveCarverRouter,
    pub structure_locator: StructureLocator,
}

impl OverworldGenerator {
    pub fn new(seed: i64) -> Self {
        let climate_sampler = ClimateSampler::new(seed);
        let biome_registry = OverworldBiomeRegistry::load_embedded();
        let blended_noise = BlendedNoise::new(seed);
        let cave_router = CaveNoiseRouter::new(seed);
        let carver_router = CaveCarverRouter::new(seed);
        let structure_locator = StructureLocator::new(seed);

        Self {
            seed,
            climate_sampler,
            biome_registry,
            blended_noise,
            cave_router,
            carver_router,
            structure_locator,
        }
    }

    /// Checks if this chunk is a structure starting location and instantiates the structure root.
    #[inline]
    pub fn get_structures_in_chunk(&self, chunk_x: i32, chunk_z: i32) -> Vec<StructureRoot> {
        self.structure_locator.get_structures_in_chunk(chunk_x, chunk_z)
    }

    /// Samples climate target and finds exact vanilla biome name at quart coordinates.
    #[inline]
    pub fn sample_biome(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> &str {
        let block_x = quart_x << 2;
        let block_y = quart_y << 2;
        let block_z = quart_z << 2;
        let target = self.climate_sampler.sample(block_x, block_y, block_z);
        self.biome_registry.find_biome(&target)
    }

    /// Evaluates 3D final density at block coordinate (x, y, z) merging base terrain with cave carving.
    pub fn calculate_density(&self, x: i32, y: i32, z: i32) -> f64 {
        if y >= 256 {
            return -100.0;
        }
        if y <= -64 {
            return 100.0;
        }

        // Base 3D blended terrain noise
        let base_3d = self.blended_noise.sample(x, y, z);

        // Continentalness, erosion, and ridges offset spline calculation
        let (raw_c, raw_e, raw_w) = self.climate_sampler.sample_raw_climate(x, z);
        let c = raw_c as f32;
        let e = raw_e as f32;
        let w = raw_w as f32;

        let offset = super::spline::evaluate_terrain_offset(c, e, w) as f64;
        let base_height = 128.0 * (1.0 + offset);
        let depth = (base_height - (y as f64)) / 128.0;
        let factor = super::spline::evaluate_terrain_factor(c, e, w, w) as f64;
        let scaled_depth = if depth < 0.0 { depth * factor * 0.25 } else { depth * factor };
        let depth_density = 4.0 * scaled_depth;

        let sloped_density = if y >= 240 {
            let t = ((y - 240) as f64) / 16.0;
            let slide = (1.0 - t).clamp(0.0, 1.0);
            (depth_density + base_3d) * slide - (1.0 - slide) * 0.5
        } else {
            depth_density + base_3d
        };

        // Merge base 3D terrain density with cave carving values using vanilla min/max density blending:
        // Subterranean cheese caverns only carve when deep underground (sloped_density >= 1.5625)
        if sloped_density >= 1.5625 {
            if let Some(_cave_type) = self.carver_router.sample_cave(x as f64, y as f64, z as f64) {
                // Carved deep subterranean cave void: carve out air pocket by blending negative density
                sloped_density.min(-0.35)
            } else {
                sloped_density
            }
        } else {
            // Near surface (sloped_density < 1.5625): only cave entrances carve if entrances * 5.0 is negative
            let entrances = self.cave_router.sample_caves_entrances(
                (x as f64) * 0.05,
                (y as f64) * 0.05,
                (z as f64) * 0.05,
            );
            if entrances < 0.0 {
                sloped_density.min(entrances * 5.0)
            } else {
                sloped_density
            }
        }
    }

    /// Implements canonical 4x8x4 trilinear interpolation across cell boundaries into 16x16x16 chunk sections (-64 to 320 Y).
    pub fn generate_section_density(
        &self,
        chunk_x: i32,
        section_y: i32,
        chunk_z: i32,
    ) -> ChunkSectionDensity {
        let base_x = chunk_x * 16;
        let base_y = section_y * 16;
        let base_z = chunk_z * 16;

        // Sample 5x3x5 grid corners for 4x8x4 cell subdivision (horizontal size=4, vertical size=8)
        let mut corners = [[[0.0f64; 5]; 3]; 5];
        for cy in 0..3 {
            for cz in 0..5 {
                for cx in 0..5 {
                    let bx = base_x + cx * 4;
                    let by = base_y + cy * 8;
                    let bz = base_z + cz * 4;
                    corners[cx as usize][cy as usize][cz as usize] =
                        self.calculate_density(bx, by, bz);
                }
            }
        }

        let mut densities = Box::new([0.0f64; 16 * 16 * 16]);

        // Trilinearly interpolate inside each 4x8x4 cell (4 in X, 2 in Y, 4 in Z) matching Horizon and Minecraft
        for cell_y in 0..2 {
            for cell_z in 0..4 {
                for cell_x in 0..4 {
                    let c000 = corners[cell_x][cell_y][cell_z];
                    let c100 = corners[cell_x + 1][cell_y][cell_z];
                    let c010 = corners[cell_x][cell_y + 1][cell_z];
                    let c110 = corners[cell_x + 1][cell_y + 1][cell_z];
                    let c001 = corners[cell_x][cell_y][cell_z + 1];
                    let c101 = corners[cell_x + 1][cell_y][cell_z + 1];
                    let c011 = corners[cell_x][cell_y + 1][cell_z + 1];
                    let c111 = corners[cell_x + 1][cell_y + 1][cell_z + 1];

                    for dz in 0..4 {
                        let fz = (dz as f64) * 0.25;
                        let z00 = c000 + fz * (c001 - c000);
                        let z01 = c010 + fz * (c011 - c010);
                        let z10 = c100 + fz * (c101 - c100);
                        let z11 = c110 + fz * (c111 - c110);
                        for dx in 0..4 {
                            let fx = (dx as f64) * 0.25;
                            let low = z00 + fx * (z10 - z00);
                            let high = z01 + fx * (z11 - z01);
                            let inc = (high - low) * 0.125;
                            for dy in 0..8 {
                                let final_val = low + (dy as f64) * inc;

                                let lx = cell_x * 4 + dx;
                                let ly = cell_y * 8 + dy;
                                let lz = cell_z * 4 + dz;

                                densities[ly * 256 + lz * 16 + lx] = final_val;
                            }
                        }
                    }
                }
            }
        }

        ChunkSectionDensity {
            chunk_x,
            section_y,
            chunk_z,
            densities,
        }
    }

    /// Computes the exact surface height (OCEAN_FLOOR_WG) at block column (x, z).
    pub fn get_surface_height(&self, x: i32, z: i32) -> i32 {
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        let lx = (x & 15) as usize;
        let lz = (z & 15) as usize;

        for section_y in (-4..20).rev() {
            let section = self.generate_section_density(chunk_x, section_y, chunk_z);
            for ly in (0..16).rev() {
                if section.is_solid(lx, ly, lz) {
                    return section_y * 16 + (ly as i32) + 1;
                }
            }
        }
        MIN_Y
    }

    /// Generates full 16x16 heightmap array for a chunk matching vanilla Minecraft 1.20+ OCEAN_FLOOR_WG.
    pub fn generate_chunk_heightmap(&self, chunk_x: i32, chunk_z: i32) -> [i32; 256] {
        let mut heightmap = [MIN_Y; 256];
        let mut remaining = 256;

        for section_y in (-4..20).rev() {
            let section = self.generate_section_density(chunk_x, section_y, chunk_z);
            for cz in 0..16 {
                for cx in 0..16 {
                    let idx = cz * 16 + cx;
                    if heightmap[idx] == MIN_Y {
                        for ly in (0..16).rev() {
                            if section.is_solid(cx, ly, cz) {
                                heightmap[idx] = section_y * 16 + (ly as i32) + 1;
                                remaining -= 1;
                                break;
                            }
                        }
                    }
                }
            }
            if remaining == 0 {
                break;
            }
        }

        heightmap
    }
}
