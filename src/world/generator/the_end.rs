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

/// Canonical End obsidian pillar configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndSpike {
    pub center_x: i32,
    pub center_z: i32,
    pub radius: i32,
    pub height: i32,
    pub guarded: bool,
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

    /// Returns the 10 canonical obsidian spike configurations arranged around (0, 0)
    pub fn get_end_spikes() -> [EndSpike; 10] {
        let mut spikes = [EndSpike {
            center_x: 0,
            center_z: 0,
            radius: 2,
            height: 76,
            guarded: false,
        }; 10];
        for i in 0..10 {
            let angle = (i as f32) * std::f32::consts::TAU / 10.0;
            let cx = (42.0 * angle.cos()).round() as i32;
            let cz = (42.0 * angle.sin()).round() as i32;
            let radius = 2 + (i % 2) as i32;
            let height = 76 + (i as i32) * 3;
            let guarded = (i == 3) || (i == 7);
            spikes[i] = EndSpike {
                center_x: cx,
                center_z: cz,
                radius,
                height,
                guarded,
            };
        }
        spikes
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

        // 3. Central Island Features Pass: Obsidian Pillars, Exit Fountain, Gateway Portals
        if chunk_center_dist <= 120.0 {
            // A. 10 Obsidian Pillars arranged circularly at radius ~42 around (0, 0)
            let spikes = Self::get_end_spikes();
            for spike in &spikes {
                let r_padded = spike.radius + 2;
                if spike.center_x + r_padded >= world_base_x
                    && spike.center_x - r_padded < world_base_x + 16
                    && spike.center_z + r_padded >= world_base_z
                    && spike.center_z - r_padded < world_base_z + 16
                {
                    for local_z in 0..16 {
                        for local_x in 0..16 {
                            let wx = world_base_x + local_x as i32;
                            let wz = world_base_z + local_z as i32;
                            let dx = wx - spike.center_x;
                            let dz = wz - spike.center_z;
                            let dist_sq = dx * dx + dz * dz;

                            if dist_sq <= spike.radius * spike.radius {
                                for y in 50..=spike.height {
                                    if (y as usize) < 128 {
                                        let idx = ((local_z * 16 + local_x) * 128) + y as usize;
                                        if dx == 0 && dz == 0 && y == spike.height {
                                            raw_grid[idx] = BlockId::Bedrock;
                                        } else {
                                            raw_grid[idx] = BlockId::Obsidian;
                                        }
                                    }
                                }
                                // End crystal at apex
                                if dx == 0 && dz == 0 && (spike.height + 1) < 128 {
                                    let crystal_idx = ((local_z * 16 + local_x) * 128) + (spike.height + 1) as usize;
                                    raw_grid[crystal_idx] = BlockId::Torch;
                                }
                            }

                            // Iron bar / glass protective cage on guarded spikes
                            if spike.guarded && dx.abs() <= 2 && dz.abs() <= 2 {
                                for dy in 1..=3 {
                                    let gy = spike.height + dy;
                                    if (gy as usize) < 128 {
                                        let is_wall = dx.abs() == 2 || dz.abs() == 2;
                                        let is_roof = dy == 3 && dx.abs() <= 2 && dz.abs() <= 2;
                                        if is_wall || is_roof {
                                            let cage_idx = ((local_z * 16 + local_x) * 128) + gy as usize;
                                            raw_grid[cage_idx] = BlockId::Glass;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // B. Exit Fountain Portal at (0, 65, 0)
            if world_base_x <= 4 && world_base_x + 16 > -4 && world_base_z <= 4 && world_base_z + 16 > -4 {
                for dz in -4..=4 {
                    for dx in -4..=4 {
                        let wx = dx;
                        let wz = dz;
                        if wx >= world_base_x && wx < world_base_x + 16 && wz >= world_base_z && wz < world_base_z + 16 {
                            let lx = (wx - world_base_x) as usize;
                            let lz = (wz - world_base_z) as usize;
                            let dist_sq = dx * dx + dz * dz;

                            if dist_sq <= 3 {
                                // Inner pool: Bedrock base at Y=64, EndPortal at Y=65
                                let base_idx = ((lz * 16 + lx) * 128) + 64;
                                raw_grid[base_idx] = BlockId::Bedrock;
                                let portal_idx = ((lz * 16 + lx) * 128) + 65;
                                raw_grid[portal_idx] = BlockId::EndPortal;
                            } else if dist_sq <= 8 {
                                // Outer raised Bedrock rim
                                let rim_base = ((lz * 16 + lx) * 128) + 64;
                                raw_grid[rim_base] = BlockId::Bedrock;
                                let rim_idx = ((lz * 16 + lx) * 128) + 65;
                                raw_grid[rim_idx] = BlockId::Bedrock;
                            }
                        }
                    }
                }

                // Central Bedrock pillar with torch at (0, 0)
                if world_base_x <= 0 && world_base_x + 16 > 0 && world_base_z <= 0 && world_base_z + 16 > 0 {
                    let lx = (0 - world_base_x) as usize;
                    let lz = (0 - world_base_z) as usize;
                    for py in 65..=68 {
                        let p_idx = ((lz * 16 + lx) * 128) + py;
                        raw_grid[p_idx] = BlockId::Bedrock;
                    }
                    let torch_idx = ((lz * 16 + lx) * 128) + 69;
                    raw_grid[torch_idx] = BlockId::Torch;
                }
            }

            // C. Gateway Portals at Island Perimeter (radius ~96 blocks)
            let gateway_coords = [(96, 75, 0), (-96, 75, 0), (0, 75, 96), (0, 75, -96)];
            for (gx, gy, gz) in gateway_coords {
                if gx + 2 >= world_base_x
                    && gx - 2 < world_base_x + 16
                    && gz + 2 >= world_base_z
                    && gz - 2 < world_base_z + 16
                {
                    for dz in -1i32..=1i32 {
                        for dx in -1i32..=1i32 {
                            for dy in -1i32..=1i32 {
                                let wx = gx + dx;
                                let wy = gy + dy;
                                let wz = gz + dz;

                                if wx >= world_base_x && wx < world_base_x + 16 && wz >= world_base_z && wz < world_base_z + 16 && wy >= 0 && wy < 128 {
                                    let lx = (wx - world_base_x) as usize;
                                    let lz = (wz - world_base_z) as usize;
                                    let g_idx = ((lz * 16 + lx) * 128) + wy as usize;

                                    if dx == 0 && dy == 0 && dz == 0 {
                                        raw_grid[g_idx] = BlockId::EndPortal;
                                    } else if dx.abs() + dy.abs() + dz.abs() <= 2 {
                                        raw_grid[g_idx] = BlockId::Bedrock;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 4. Pack raw grid into the 8 vertical 16x16x16 ChunkSections
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

    #[test]
    fn test_end_obsidian_pillars_and_crystals() {
        let generator = TheEndGenerator::new(505);
        let spikes = TheEndGenerator::get_end_spikes();
        assert_eq!(spikes.len(), 10);

        for (i, spike) in spikes.iter().enumerate() {
            let dist = ((spike.center_x as f32).powi(2) + (spike.center_z as f32).powi(2)).sqrt();
            assert!(
                (dist - 42.0).abs() <= 1.5,
                "Spike {i} distance {dist} should be ~42 blocks from origin"
            );
            assert!(spike.height >= 76 && spike.height <= 103);
            if i == 3 || i == 7 {
                assert!(spike.guarded, "Spike {i} should be guarded");
            }
        }

        // Generate chunk containing spike 0 (located at (42, 0) -> chunk_x = 2, chunk_z = 0)
        let chunk_x = spikes[0].center_x / 16;
        let chunk_z = spikes[0].center_z / 16;
        let mut world = crate::game::world::World::new();
        generator.populate_world(&mut world, chunk_x, chunk_z);

        // Check obsidian shaft
        let shaft_block = world.get_block(IVec3::new(spikes[0].center_x, 60, spikes[0].center_z));
        assert_eq!(shaft_block, BlockId::Obsidian);

        // Check bedrock cap at height
        let cap_block = world.get_block(IVec3::new(spikes[0].center_x, spikes[0].height, spikes[0].center_z));
        assert_eq!(cap_block, BlockId::Bedrock);

        // Check crystal torch above cap
        let crystal_block = world.get_block(IVec3::new(spikes[0].center_x, spikes[0].height + 1, spikes[0].center_z));
        assert_eq!(crystal_block, BlockId::Torch);
    }

    #[test]
    fn test_end_exit_fountain_and_gateway() {
        let generator = TheEndGenerator::new(606);
        let mut world = crate::game::world::World::new();

        // 1. Exit fountain in chunk (0, 0)
        generator.populate_world(&mut world, 0, 0);

        // Center pillar
        assert_eq!(world.get_block(IVec3::new(0, 65, 0)), BlockId::Bedrock);
        assert_eq!(world.get_block(IVec3::new(0, 68, 0)), BlockId::Bedrock);
        assert_eq!(world.get_block(IVec3::new(0, 69, 0)), BlockId::Torch);

        // Portal basin inner tile
        assert_eq!(world.get_block(IVec3::new(1, 65, 0)), BlockId::EndPortal);
        assert_eq!(world.get_block(IVec3::new(1, 64, 0)), BlockId::Bedrock);

        // Outer rim
        assert_eq!(world.get_block(IVec3::new(2, 65, 1)), BlockId::Bedrock);

        // 2. Gateway portal at (96, 75, 0) spanning chunk boundary (5, 0) and (6, 0)
        generator.populate_world(&mut world, 5, 0);
        generator.populate_world(&mut world, 6, 0);
        assert_eq!(world.get_block(IVec3::new(96, 75, 0)), BlockId::EndPortal);
        assert_eq!(world.get_block(IVec3::new(96, 74, 0)), BlockId::Bedrock);
        assert_eq!(world.get_block(IVec3::new(96, 76, 0)), BlockId::Bedrock);
        assert_eq!(world.get_block(IVec3::new(97, 75, 0)), BlockId::Bedrock);
        assert_eq!(world.get_block(IVec3::new(95, 75, 0)), BlockId::Bedrock);
    }
}
