//! 16x40x16 Grid cell sampler for subterranean aquifers.
//! REA Reference: net.minecraft.world.level.levelgen.NoiseBasedAquifer (doc$c.class)
//! REA Reference: methods gridX (476), gridY (480), gridZ (484), computeRandomizedFluidSurfaceLevel (615), computeFluidType (646)

use super::fluid::{AquiferFluidType, FluidStatus, LAVA_BARRIER_Y, SEA_LEVEL};
use crate::seed_pipeline::noise::NormalNoise;
use crate::seed_pipeline::random::PositionalRandomFactory;

/// Horizontal cell size along X axis (blocks).
pub const CELL_SIZE_X: i32 = 16;
/// Vertical cell size along Y axis (blocks).
pub const CELL_SIZE_Y: i32 = 40;
/// Horizontal cell size along Z axis (blocks).
pub const CELL_SIZE_Z: i32 = 16;

/// Converts block X coordinate to 16-block cell grid X.
#[inline]
pub fn grid_x(x: i32) -> i32 {
    x.div_euclid(CELL_SIZE_X)
}

/// Converts block Y coordinate to 40-block cell grid Y.
#[inline]
pub fn grid_y(y: i32) -> i32 {
    y.div_euclid(CELL_SIZE_Y)
}

/// Converts block Z coordinate to 16-block cell grid Z.
#[inline]
pub fn grid_z(z: i32) -> i32 {
    z.div_euclid(CELL_SIZE_Z)
}

/// Anchor point coordinates representing the center of an aquifer cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellAnchor {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Grid-based aquifer cell sampler managing fluid levels and fluid types across 16x40x16 cells.
#[derive(Clone, Debug)]
pub struct AquiferGridSampler {
    pub fluid_level_floodedness: NormalNoise,
    pub fluid_level_spread: NormalNoise,
    pub lava_noise: NormalNoise,
    pub random_factory: PositionalRandomFactory,
}

impl AquiferGridSampler {
    /// Constructs a new grid sampler from vanilla noise parameters.
    pub fn new(factory: &PositionalRandomFactory) -> Self {
        // REA Reference: data/minecraft/worldgen/noise/aquifer_fluid_level_floodedness.json (firstOctave: -7, [1.0])
        let mut flood_rng = factory.from_hash_of("minecraft:aquifer_fluid_level_floodedness");
        let fluid_level_floodedness = NormalNoise::from_rng(&mut flood_rng, -7, &[1.0]);

        // REA Reference: data/minecraft/worldgen/noise/aquifer_fluid_level_spread.json (firstOctave: -5, [1.0])
        let mut spread_rng = factory.from_hash_of("minecraft:aquifer_fluid_level_spread");
        let fluid_level_spread = NormalNoise::from_rng(&mut spread_rng, -5, &[1.0]);

        // REA Reference: data/minecraft/worldgen/noise/aquifer_lava.json (firstOctave: -1, [1.0])
        let mut lava_rng = factory.from_hash_of("minecraft:aquifer_lava");
        let lava_noise = NormalNoise::from_rng(&mut lava_rng, -1, &[1.0]);

        Self {
            fluid_level_floodedness,
            fluid_level_spread,
            lava_noise,
            random_factory: factory.clone(),
        }
    }

    /// Computes the deterministically jittered anchor point for cell (cell_x, cell_y, cell_z).
    /// Matches vanilla positional random offset within cell bounds.
    pub fn get_cell_anchor(&self, cell_x: i32, cell_y: i32, cell_z: i32) -> CellAnchor {
        let mut rng = self.random_factory.at(cell_x, cell_y, cell_z);
        let offset_x = rng.next_int_bounded(10);
        let offset_y = rng.next_int_bounded(9);
        let offset_z = rng.next_int_bounded(10);

        CellAnchor {
            x: cell_x * CELL_SIZE_X + offset_x,
            y: cell_y * CELL_SIZE_Y + offset_y,
            z: cell_z * CELL_SIZE_Z + offset_z,
        }
    }

    /// Computes the fluid status (target level and fluid type) for a specific cell coordinate.
    ///
    /// Rules:
    /// 1. Base level = cell_y * 40 + 20.
    /// 2. Spread noise modulates fluid elevation by +/- 10 blocks.
    /// 3. Floodedness noise evaluates if the cell is an unflooded dry cave chamber.
    /// 4. If fluid level < -54, fluid switches from Water to Lava.
    /// 5. If elevation <= -10 and abs(lava_noise) > 0.3, fluid switches to Lava.
    pub fn compute_cell_status(&self, cell_x: i32, cell_y: i32, cell_z: i32) -> FluidStatus {
        let bx = (cell_x * CELL_SIZE_X) as f64;
        let by = (cell_y * CELL_SIZE_Y) as f64;
        let bz = (cell_z * CELL_SIZE_Z) as f64;

        // Sample floodedness noise: negative values create dry cave air pockets
        let floodedness = self.fluid_level_floodedness.sample(bx, by, bz);
        if floodedness < -0.3 {
            // Completely dry cavern cell: no standing water or lava
            return FluidStatus::new(i32::MIN, AquiferFluidType::Air);
        }

        // Base elevation for 40-block cell interval
        let base_level = cell_y * CELL_SIZE_Y + 20;

        // Spread noise: modulates fluid level by +/- 10 blocks
        let spread = self.fluid_level_spread.sample(bx, by, bz);
        let offset = (spread * 10.0).round() as i32;
        let target_level = (base_level + offset).min(SEA_LEVEL);

        // Determine fluid type:
        // Rule: If target level or cell floor < -54, fluid strictly switches to Lava
        let fluid_type = if target_level < LAVA_BARRIER_Y || cell_y * CELL_SIZE_Y < LAVA_BARRIER_Y {
            AquiferFluidType::Lava
        } else if target_level <= -10 {
            // Sample lava noise on 64x40x64 grid
            let lava_val = self.lava_noise.sample(bx * 0.25, by * 0.25, bz * 0.25);
            if lava_val.abs() > 0.3 {
                AquiferFluidType::Lava
            } else {
                AquiferFluidType::Water
            }
        } else {
            AquiferFluidType::Water
        };

        FluidStatus::new(target_level, fluid_type)
    }
}
