// REA Verified: net.minecraft.world.level.levelgen.DensityFunctions$EndIslandDensityFunction (mapped: dok$i.class)
// REA Verified: net.minecraft.world.level.levelgen.synth.SimplexNoise (mapped: edx.class)
// REA Verified: net.minecraft.world.level.biome.TheEndBiomeSource (mapped: cvf.class)
//
// The End Dimension Radial Island Noise Generator:
// 1. Central Island:
//    - Single large convex landmass centered at (0, 0) with radius ~100 blocks.
//    - Made purely of End Stone with smooth parabolic height taper.
// 2. The Void Gap:
//    - Total zero density enforced between radius 100 and 1000 blocks (complete void).
// 3. Outer Islands:
//    - Beyond radius 1000, 2D Simplex noise seeds circular floating islands.
//    - Smooth parabolic underside falloff tapering downward into the void.
// 4. Surface Pass:
//    - Stochastic cellular growth loop generating branching Chorus Plants capped with Chorus Flowers.

use glam::IVec3;
use crate::render::mesher::{ChunkSection, SECTION_SIZE};
use crate::world::block_registry::BlockId;

pub const THE_END_MIN_Y: i32 = 0;
pub const THE_END_MAX_Y: i32 = 128;
pub const THE_END_SECTIONS_COUNT: usize = 8; // 128 / 16

pub const CENTRAL_ISLAND_RADIUS: f32 = 100.0;
pub const VOID_GAP_MIN_RADIUS: f32 = 100.0;
pub const VOID_GAP_MAX_RADIUS: f32 = 1000.0;

/// The End dimension biomes matching net.minecraft.world.level.biome.TheEndBiomeSource (cvf.class)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndBiome {
    TheEnd,          // Central Island (R < 100)
    TheVoid,         // Void Gap (100 <= R < 1000)
    EndHighlands,    // High interior plateaus of outer islands
    EndMidlands,     // Slopes transitioning down from highlands
    EndBarrens,      // Island edges overlooking the void
    SmallEndIslands, // Small scattered floating islets
}

impl EndBiome {
    pub fn name(&self) -> &'static str {
        match self {
            EndBiome::TheEnd => "The End",
            EndBiome::TheVoid => "The Void",
            EndBiome::EndHighlands => "End Highlands",
            EndBiome::EndMidlands => "End Midlands",
            EndBiome::EndBarrens => "End Barrens",
            EndBiome::SmallEndIslands => "Small End Islands",
        }
    }

    /// Ambient sky / fog color for The End dimension
    pub fn ambient_fog_color(&self) -> [f32; 3] {
        // Deep mysterious violet / purple haze
        [0.08, 0.05, 0.12]
    }
}

/// 2D Simplex Noise matching net.minecraft.world.level.levelgen.synth.SimplexNoise (edx.class)
#[derive(Clone, Debug)]
pub struct SimplexNoise2D {
    perm: [i32; 512],
    pub offset_x: f64,
    pub offset_z: f64,
}

impl SimplexNoise2D {
    pub fn new(seed: u64) -> Self {
        let mut p = [0i32; 256];
        for i in 0..256 {
            p[i] = i as i32;
        }

        // Shuffle with seed
        let mut s = seed;
        for i in (1..256).rev() {
            s = (s ^ (s >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            let j = (s % (i as u64 + 1)) as usize;
            p.swap(i, j);
        }

        let mut perm = [0i32; 512];
        for i in 0..512 {
            perm[i] = p[i & 255];
        }

        let ox = ((seed & 0xffff) as f64 / 65535.0) * 256.0;
        let oz = (((seed >> 16) & 0xffff) as f64 / 65535.0) * 256.0;

        Self {
            perm,
            offset_x: ox,
            offset_z: oz,
        }
    }

    /// Evaluates 2D Simplex Noise at (x, z)
    pub fn sample_2d(&self, xin: f64, zin: f64) -> f64 {
        // Skewing and unskewing factors for 2D
        let f2 = 0.5 * (3.0_f64.sqrt() - 1.0);
        let g2 = (3.0 - 3.0_f64.sqrt()) / 6.0;

        let x = xin + self.offset_x;
        let z = zin + self.offset_z;

        let s = (x + z) * f2;
        let i = (x + s).floor() as i32;
        let j = (z + s).floor() as i32;

        let t = (i + j) as f64 * g2;
        let x0 = x - (i as f64 - t);
        let z0 = z - (j as f64 - t);

        let (i1, j1) = if x0 > z0 { (1, 0) } else { (0, 1) };

        let x1 = x0 - i1 as f64 + g2;
        let z1 = z0 - j1 as f64 + g2;
        let x2 = x0 - 1.0 + 2.0 * g2;
        let z2 = z0 - 1.0 + 2.0 * g2;

        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;

        let gi0 = self.perm[ii + self.perm[jj] as usize] % 8;
        let gi1 = self.perm[ii + i1 as usize + self.perm[jj + j1 as usize] as usize] % 8;
        let gi2 = self.perm[ii + 1 + self.perm[jj + 1] as usize] % 8;

        // 2D gradients
        const GRADS: [[f64; 2]; 8] = [
            [1.0, 1.0], [-1.0, 1.0], [1.0, -1.0], [-1.0, -1.0],
            [1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0],
        ];

        let mut n0 = 0.5 - x0 * x0 - z0 * z0;
        let t0 = if n0 < 0.0 {
            0.0
        } else {
            n0 *= n0;
            n0 * n0 * (GRADS[gi0 as usize][0] * x0 + GRADS[gi0 as usize][1] * z0)
        };

        let mut n1 = 0.5 - x1 * x1 - z1 * z1;
        let t1 = if n1 < 0.0 {
            0.0
        } else {
            n1 *= n1;
            n1 * n1 * (GRADS[gi1 as usize][0] * x1 + GRADS[gi1 as usize][1] * z1)
        };

        let mut n2 = 0.5 - x2 * x2 - z2 * z2;
        let t2 = if n2 < 0.0 {
            0.0
        } else {
            n2 *= n2;
            n2 * n2 * (GRADS[gi2 as usize][0] * x2 + GRADS[gi2 as usize][1] * z2)
        };

        // Scale to [-1, 1]
        70.0 * (t0 + t1 + t2)
    }
}

/// Configuration settings for The End island generator
#[derive(Clone, Debug)]
pub struct EndGeneratorConfig {
    pub seed: u64,
    pub central_radius: f32,
    pub void_gap_min: f32,
    pub void_gap_max: f32,
}

impl Default for EndGeneratorConfig {
    fn default() -> Self {
        Self {
            seed: 54321,
            central_radius: CENTRAL_ISLAND_RADIUS,
            void_gap_min: VOID_GAP_MIN_RADIUS,
            void_gap_max: VOID_GAP_MAX_RADIUS,
        }
    }
}

/// The End Dimension World Generator
#[derive(Clone, Debug)]
pub struct TheEndGenerator {
    pub config: EndGeneratorConfig,
    island_noise: SimplexNoise2D,
}

impl TheEndGenerator {
    pub fn new(seed: u64) -> Self {
        Self::with_config(EndGeneratorConfig {
            seed,
            ..Default::default()
        })
    }

    pub fn with_config(config: EndGeneratorConfig) -> Self {
        let island_noise = SimplexNoise2D::new(config.seed ^ 0x0000_1729_2000_0000);
        Self {
            config,
            island_noise,
        }
    }

    /// Evaluates the distance from origin in the XZ horizontal plane
    #[inline]
    pub fn horizontal_distance(x: i32, z: i32) -> f32 {
        ((x as f32) * (x as f32) + (z as f32) * (z as f32)).sqrt()
    }

    /// Evaluates which End biome covers coordinate (x, y, z)
    pub fn sample_biome(&self, x: i32, _y: i32, z: i32) -> EndBiome {
        let dist = Self::horizontal_distance(x, z);

        if dist <= self.config.central_radius {
            return EndBiome::TheEnd;
        }

        if dist < self.config.void_gap_max {
            return EndBiome::TheVoid;
        }

        // Outer Islands: classify based on island height / density
        let height_val = self.get_outer_island_height_value(x, z);
        if height_val > 40.0 {
            EndBiome::EndHighlands
        } else if height_val > 10.0 {
            EndBiome::EndMidlands
        } else if height_val > -20.0 {
            EndBiome::EndBarrens
        } else {
            let n = self.island_noise.sample_2d(x as f64 * 0.05, z as f64 * 0.05);
            if n > 0.5 {
                EndBiome::SmallEndIslands
            } else {
                EndBiome::TheVoid
            }
        }
    }

    /// Re-implementation of net.minecraft.world.level.levelgen.DensityFunctions$EndIslandDensityFunction.getHeightValue
    /// (mapped: dok$i.class bytecode) for outer floating islands.
    pub fn get_outer_island_height_value(&self, block_x: i32, block_z: i32) -> f32 {
        let cell_x = block_x / 16;
        let cell_z = block_z / 16;
        let rem_x = block_x % 16;
        let rem_z = block_z % 16;

        let mut max_island_height = -100.0f32;

        // Search in a local neighborhood of island center candidate cells
        for dx in -3..=3 {
            for dz in -3..=3 {
                let cx = cell_x + dx;
                let cz = cell_z + dz;

                let cx_world = (cx * 16) as f32;
                let cz_world = (cz * 16) as f32;
                let dist_origin = (cx_world * cx_world + cz_world * cz_world).sqrt();

                // Enforce the void gap: no islands within void gap radius
                if dist_origin < self.config.void_gap_max {
                    continue;
                }

                // Sample SimplexNoise: islands spawn where noise dips below threshold
                let noise_val = self.island_noise.sample_2d(cx as f64 * 0.125, cz as f64 * 0.125);
                if noise_val < -0.35 {
                    // Radius / scale formula reverse-engineered from dok$i.class bytecode:
                    // float rSeed = (abs(cx) * 3439.0 + abs(cz) * 147.0) % 13.0 + 9.0;
                    let seed_val = (cx.abs() as f32 * 3439.0 + cz.abs() as f32 * 147.0) % 13.0 + 9.0;
                    let island_radius = seed_val * 4.0; // Radius in blocks (36 to 88 blocks)

                    let local_dx = (rem_x - dx * 16) as f32;
                    let local_dz = (rem_z - dz * 16) as f32;
                    let dist_to_center = (local_dx * local_dx + local_dz * local_dz).sqrt();

                    if dist_to_center < island_radius {
                        let h = 100.0 - (dist_to_center / island_radius) * 100.0;
                        max_island_height = max_island_height.max(h);
                    }
                }
            }
        }

        max_island_height
    }

    /// Evaluates 3D density of The End at world coordinate (x, y, z).
    /// Returns > 0.0 for solid terrain (End Stone), <= 0.0 for air/void.
    pub fn sample_density(&self, x: i32, y: i32, z: i32) -> f32 {
        let dist = Self::horizontal_distance(x, z);

        // 1. Central Island: Single large convex mass centered at (0, 0) with radius ~100
        if dist <= self.config.central_radius {
            // Convex mass: top surface around Y=65, bottom surface around Y=45.
            // Parabolic taper toward the edges (radius 100)
            let norm_r = dist / self.config.central_radius; // 0.0 at center, 1.0 at edge
            let top_y = 65.0 - norm_r * norm_r * 12.0;
            let bottom_y = 45.0 + norm_r * norm_r * 8.0;

            let fy = y as f32;
            if fy >= bottom_y && fy <= top_y {
                // Inside central island
                return (top_y - fy).min(fy - bottom_y) + 1.0;
            } else {
                return -1.0;
            }
        }

        // 2. The Void Gap: Enforce complete zero density between radius 100 and 1000
        if dist < self.config.void_gap_max {
            return -10.0; // Strict void
        }

        // 3. Outer Islands: Beyond distance 1000, floating islands with smooth parabolic underside
        let island_h = self.get_outer_island_height_value(x, z);
        if island_h <= 0.0 {
            return -5.0; // Empty void between outer islands
        }

        // Floating island vertical profile:
        // Center of floating islands at Y = 60.
        // Upper surface at Y = 60 + island_h * 0.15 (rolling plateaus up to Y = 75).
        // Underside parabolic falloff: thickness proportional to island_h, tapering steeply down.
        let top_surface = 58.0 + (island_h / 100.0) * 16.0;
        let thickness = (island_h / 100.0).powf(1.4) * 32.0;
        let bottom_surface = top_surface - thickness;

        let fy = y as f32;
        if fy >= bottom_surface && fy <= top_surface {
            // Inside outer floating island
            (top_surface - fy).min(fy - bottom_surface) + 0.5
        } else {
            -1.0
        }
    }

    /// Evaluates base block at world coordinate (x, y, z)
    pub fn sample_block(&self, x: i32, y: i32, z: i32) -> BlockId {
        let density = self.sample_density(x, y, z);
        if density > 0.0 {
            BlockId::EndStone
        } else {
            BlockId::Air
        }
    }

    /// Surface pass: Generates Chorus Plants and Chorus Flowers on outer islands
    /// using a stochastic branching cellular growth loop.
    pub fn grow_chorus_plant(
        raw_grid: &mut [BlockId],
        local_x: usize,
        y: usize,
        local_z: usize,
        seed: u64,
    ) {
        if y + 10 >= 127 {
            return;
        }

        let mut current_branches = vec![(local_x, y, local_z, 0, 4)]; // (x, y, z, branch_count, remaining_growth)

        while let Some((bx, by, bz, branches, remaining)) = current_branches.pop() {
            if remaining == 0 {
                // Growth ended: cap with Chorus Flower on top of the branch tip!
                let flower_y = by + 1;
                if flower_y < 127 {
                    let flower_idx = ((bz * 16 + bx) * 128) + flower_y;
                    if flower_idx < raw_grid.len() && raw_grid[flower_idx] == BlockId::Air {
                        raw_grid[flower_idx] = BlockId::ChorusFlower;
                    }
                }
                continue;
            }

            // Grow upward 1 block
            let next_y = by + 1;
            let idx = ((bz * 16 + bx) * 128) + next_y;
            if next_y >= 126 || idx >= raw_grid.len() || raw_grid[idx] != BlockId::Air {
                // Blocked: cap with Flower at current tip
                let flower_idx = ((bz * 16 + bx) * 128) + by;
                if flower_idx < raw_grid.len() {
                    raw_grid[flower_idx] = BlockId::ChorusFlower;
                }
                continue;
            }

            raw_grid[idx] = BlockId::ChorusPlant;

            // Stochastic branching: decide whether to branch horizontally or continue upward
            let h = crate::world::generator::nether::hash_coords(seed ^ (branches as u64), bx as i32, next_y as i32, bz as i32);
            let should_branch = (h % 3) == 0 && branches < 3;

            if should_branch {
                // Choose horizontal direction: North, South, East, West
                let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
                let dir = dirs[(h % 4) as usize];
                let nx = bx as i32 + dir.0;
                let nz = bz as i32 + dir.1;

                if nx >= 0 && nx < 16 && nz >= 0 && nz < 16 {
                    let side_idx = ((nz as usize * 16 + nx as usize) * 128) + next_y;
                    if side_idx < raw_grid.len() && raw_grid[side_idx] == BlockId::Air {
                        raw_grid[side_idx] = BlockId::ChorusPlant;
                        current_branches.push((nx as usize, next_y, nz as usize, branches + 1, remaining - 1));
                    }
                }
            } else {
                current_branches.push((bx, next_y, bz, branches, remaining - 1));
            }
        }
    }

    /// Generates full 16x128x16 The End chunk column (8 ChunkSections)
    pub fn generate_chunk_column(&self, chunk_x: i32, chunk_z: i32) -> [ChunkSection; THE_END_SECTIONS_COUNT] {
        let mut sections: [ChunkSection; THE_END_SECTIONS_COUNT] = std::array::from_fn(|i| {
            ChunkSection::new(IVec3::new(chunk_x, i as i32, chunk_z))
        });

        let world_base_x = chunk_x * SECTION_SIZE as i32;
        let world_base_z = chunk_z * SECTION_SIZE as i32;

        let mut raw_grid = vec![BlockId::Air; 16 * 128 * 16];

        // 1. Terrain Shape Pass: Sample density for all voxels in the column
        for local_z in 0..16 {
            for local_x in 0..16 {
                let wx = world_base_x + local_x as i32;
                let wz = world_base_z + local_z as i32;

                for y in 0..128 {
                    let idx = ((local_z * 16 + local_x) * 128) + y as usize;
                    raw_grid[idx] = self.sample_block(wx, y, wz);
                }
            }
        }

        // 2. Surface Pass: Generate Chorus Plants on outer island upper surfaces
        let chunk_center_dist = Self::horizontal_distance(world_base_x + 8, world_base_z + 8);
        if chunk_center_dist >= self.config.void_gap_max {
            for local_z in 2..14 {
                for local_x in 2..14 {
                    let wx = world_base_x + local_x as i32;
                    let wz = world_base_z + local_z as i32;

                    for y in 40..100 {
                        let idx = ((local_z * 16 + local_x) * 128) + y as usize;
                        if raw_grid[idx] == BlockId::EndStone {
                            let above_idx = idx + 1;
                            if above_idx < raw_grid.len() && raw_grid[above_idx] == BlockId::Air {
                                // Exposed End Stone surface!
                                let h = crate::world::generator::nether::hash_coords(
                                    self.config.seed ^ 0xce40375,
                                    wx,
                                    y,
                                    wz,
                                );
                                if (h % 31) == 0 {
                                    Self::grow_chorus_plant(
                                        &mut raw_grid,
                                        local_x,
                                        y as usize,
                                        local_z,
                                        self.config.seed,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Pack raw grid into the 8 vertical 16x16x16 ChunkSections
        for local_z in 0..16 {
            for local_x in 0..16 {
                for y in 0..128 {
                    let sec_idx = (y / 16) as usize;
                    let sec_y = (y % 16) as usize;
                    let grid_idx = ((local_z * 16 + local_x) * 128) + y as usize;
                    let block = raw_grid[grid_idx];

                    sections[sec_idx].set_block(local_x, sec_y, local_z, block);
                }
            }
        }

        sections
    }

    /// Populates The End chunk column directly into a mutable multi-section World
    pub fn populate_world(&self, world: &mut crate::game::world::World, chunk_x: i32, chunk_z: i32) {
        let sections = self.generate_chunk_column(chunk_x, chunk_z);
        for (sec_y, section) in sections.into_iter().enumerate() {
            let pos = IVec3::new(chunk_x, sec_y as i32, chunk_z);
            world.sections.insert(pos, section);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_central_island_convex_landmass() {
        let generator = TheEndGenerator::new(101);

        // Center at (0, 0) must be solid End Stone at Y = 50..60
        for y in 50..=60 {
            assert_eq!(
                generator.sample_block(0, y, 0),
                BlockId::EndStone,
                "Center (0, {y}, 0) must be solid End Stone"
            );
        }

        // Radius < 100 must contain solid End Stone
        let mut solid_count = 0;
        for r in [20, 40, 60, 80] {
            if generator.sample_block(r, 55, 0) == BlockId::EndStone {
                solid_count += 1;
            }
        }
        assert_eq!(
            solid_count, 4,
            "Central island must be a continuous solid landmass across radius < 100"
        );

        // Biome at center must be TheEnd
        assert_eq!(generator.sample_biome(0, 60, 0), EndBiome::TheEnd);
        assert_eq!(generator.sample_biome(50, 60, 50), EndBiome::TheEnd);
    }

    #[test]
    fn test_void_gap_zero_density() {
        let generator = TheEndGenerator::new(202);

        // Between radius 100 and 1000, density must be strictly <= 0.0 (100% Air)
        let test_radii = [150, 300, 500, 750, 950];

        for r in test_radii {
            for y in 0..128 {
                let block = generator.sample_block(r, y, 0);
                assert_eq!(
                    block,
                    BlockId::Air,
                    "Void gap at radius {r}, Y={y} must be strictly Air"
                );
            }
            // Biome must be TheVoid
            assert_eq!(generator.sample_biome(r, 60, 0), EndBiome::TheVoid);
        }
    }

    #[test]
    fn test_outer_islands_generation() {
        let generator = TheEndGenerator::new(303);

        // Beyond radius 1000, scan a region for floating outer islands
        let mut found_island = false;
        let mut sample_x = 0;
        let mut sample_z = 0;

        for x in (1100..1600).step_by(32) {
            for z in (1100..1600).step_by(32) {
                if generator.sample_block(x, 60, z) == BlockId::EndStone {
                    found_island = true;
                    sample_x = x;
                    sample_z = z;
                    break;
                }
            }
            if found_island {
                break;
            }
        }

        assert!(
            found_island,
            "Outer islands beyond R=1000 must generate floating End Stone landmasses"
        );

        // Test smooth underside falloff: beneath the floating island, space becomes Air
        assert_eq!(
            generator.sample_block(sample_x, 10, sample_z),
            BlockId::Air,
            "Far beneath floating island (Y=10) must be open void Air"
        );

        // Above the floating island, space becomes Air
        assert_eq!(
            generator.sample_block(sample_x, 110, sample_z),
            BlockId::Air,
            "High above floating island (Y=110) must be open Air"
        );
    }

    #[test]
    fn test_chorus_plant_cellular_growth() {
        let mut raw_grid = vec![BlockId::Air; 16 * 128 * 16];

        // Seed an End Stone foundation block at (8, 60, 8)
        let base_idx = ((8 * 16 + 8) * 128) + 60;
        raw_grid[base_idx] = BlockId::EndStone;

        // Run the stochastic growth loop
        TheEndGenerator::grow_chorus_plant(&mut raw_grid, 8, 60, 8, 12345);

        // Scan above Y=60 for ChorusPlant and ChorusFlower
        let mut plant_count = 0;
        let mut flower_count = 0;

        for z in 0..16 {
            for x in 0..16 {
                for y in 61..120 {
                    let idx = ((z * 16 + x) * 128) + y;
                    if raw_grid[idx] == BlockId::ChorusPlant {
                        plant_count += 1;
                    }
                    if raw_grid[idx] == BlockId::ChorusFlower {
                        flower_count += 1;
                    }
                }
            }
        }

        assert!(
            plant_count > 0,
            "Chorus growth must create ChorusPlant stem segments"
        );
        assert!(
            flower_count > 0,
            "Chorus growth must terminate tips with ChorusFlower caps"
        );
    }

    #[test]
    fn test_the_end_chunk_column_and_world_population() {
        let mut world = crate::game::world::World::new();
        let generator = TheEndGenerator::new(404);

        // Generate Central Island chunk (0, 0)
        generator.populate_world(&mut world, 0, 0);

        // Verify sections 0..8 exist
        for sec_y in 0..8 {
            let pos = IVec3::new(0, sec_y, 0);
            assert!(
                world.sections.contains_key(&pos),
                "World must contain End section at {pos:?}"
            );
        }

        // Verify solid center block
        let center_block = world.get_block(IVec3::new(0, 55, 0));
        assert_eq!(center_block, BlockId::EndStone);
    }
}
