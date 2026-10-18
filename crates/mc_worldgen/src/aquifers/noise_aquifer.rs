//! Noise-based subterranean aquifer engine.
//! REA Reference: net.minecraft.world.level.levelgen.Aquifer (doc.class)
//! REA Reference: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer (doc$c.class)

use super::barrier::BarrierNoiseSampler;
use super::fluid::{AquiferFluidType, FluidStatus, BEDROCK_CEILING_Y, LAVA_BARRIER_Y, SEA_LEVEL};
use super::grid::{grid_x, grid_y, grid_z, AquiferGridSampler};
use crate::biomes::SurfaceBlock;
use crate::seed_pipeline::random::PositionalRandomFactory;

/// Result of evaluating an aquifer substance at coordinate (x, y, z).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AquiferSubstance {
    /// Solid stone/deepslate rock barrier preserving dry caverns or dividing fluids.
    Solid,
    /// Backfilled fluid (Water or Lava).
    Fluid(AquiferFluidType),
    /// Unflooded dry air cave chamber.
    Air,
}

/// Abstract Aquifer interface matching vanilla Minecraft 1.20.4.
pub trait Aquifer {
    /// Evaluates the substance state at (x, y, z) for an open or carved cavity.
    fn compute_substance(&self, x: i32, y: i32, z: i32) -> AquiferSubstance;

    /// Evaluates the target fluid status for the primary cell containing (x, y, z).
    fn get_fluid_status(&self, x: i32, y: i32, z: i32) -> FluidStatus;
}

/// 3D Grid-based Noise Aquifer simulating local underground water tables, lava pools, and dry pockets.
#[derive(Clone, Debug)]
pub struct NoiseBasedAquifer {
    pub grid_sampler: AquiferGridSampler,
    pub barrier_sampler: BarrierNoiseSampler,
    pub sea_level: i32,
    pub lava_level: i32,
}

impl NoiseBasedAquifer {
    /// Constructs a new NoiseBasedAquifer from a positional random factory.
    pub fn new(factory: &PositionalRandomFactory) -> Self {
        let grid_sampler = AquiferGridSampler::new(factory);
        let barrier_sampler = BarrierNoiseSampler::new(factory);

        Self {
            grid_sampler,
            barrier_sampler,
            sea_level: SEA_LEVEL,
            lava_level: LAVA_BARRIER_Y,
        }
    }

    /// Evaluates local Voronoi cells around (x, y, z) to find the primary and secondary anchor cells.
    /// Returns (closest_status, second_status, similarity).
    pub fn sample_local_cells(&self, x: i32, y: i32, z: i32) -> (FluidStatus, FluidStatus, f64) {
        let gx = grid_x(x);
        let gy = grid_y(y);
        let gz = grid_z(z);

        let mut best_dist1 = i32::MAX;
        let mut best_status1 = FluidStatus::new(self.sea_level, AquiferFluidType::Water);

        let mut best_dist2 = i32::MAX;
        let mut best_status2 = FluidStatus::new(self.sea_level, AquiferFluidType::Water);

        // Search adjacent 16x40x16 cell neighborhood (2x3x2 surrounding space)
        for dx in 0..=1 {
            for dy in -1..=1 {
                for dz in 0..=1 {
                    let cx = gx + dx;
                    let cy = gy + dy;
                    let cz = gz + dz;

                    let anchor = self.grid_sampler.get_cell_anchor(cx, cy, cz);
                    let dist_x = anchor.x - x;
                    let dist_y = anchor.y - y;
                    let dist_z = anchor.z - z;
                    let dist_sq = dist_x * dist_x + dist_y * dist_y + dist_z * dist_z;

                    let status = self.grid_sampler.compute_cell_status(cx, cy, cz);

                    if dist_sq < best_dist1 {
                        best_dist2 = best_dist1;
                        best_status2 = best_status1;

                        best_dist1 = dist_sq;
                        best_status1 = status;
                    } else if dist_sq < best_dist2 {
                        best_dist2 = dist_sq;
                        best_status2 = status;
                    }
                }
            }
        }

        let sim = BarrierNoiseSampler::similarity(best_dist1, best_dist2);
        (best_status1, best_status2, sim)
    }

    /// Evaluates whether an open cavity coordinate (x, y, z) should be flooded or kept dry.
    ///
    /// Flooding logic:
    /// 1. If Y < -54: fluid switches from Water to Lava.
    /// 2. If barrier wall forms at the interface: returns Solid rock (preventing leaks).
    /// 3. Below cell target fluid level: backfilled with Fluid (Water or Lava).
    /// 4. Above cell target fluid level: kept as Air.
    pub fn compute_substance_at(&self, x: i32, y: i32, z: i32) -> AquiferSubstance {
        // Bedrock is never flooded with aquifer fluid
        if y <= BEDROCK_CEILING_Y {
            return AquiferSubstance::Solid;
        }

        let (status1, status2, similarity) = self.sample_local_cells(x, y, z);

        // Check barrier noise stone dam separating disparate fluid regions
        if similarity > 0.0 {
            if self.barrier_sampler.is_barrier_wall(
                x as f64,
                y,
                z as f64,
                similarity,
                &status1,
                &status2,
            ) {
                // Forms natural impermeable rock wall
                return AquiferSubstance::Solid;
            }
        }

        // Deep subterranean lava ocean check: any cavity below Y < -54 is filled with Lava
        if y < self.lava_level {
            return AquiferSubstance::Fluid(AquiferFluidType::Lava);
        }

        // Cell fluid evaluation
        let cell_fluid = status1.at_elevation(y);
        match cell_fluid {
            AquiferFluidType::Air => AquiferSubstance::Air,
            AquiferFluidType::Water => AquiferSubstance::Fluid(AquiferFluidType::Water),
            AquiferFluidType::Lava => AquiferSubstance::Fluid(AquiferFluidType::Lava),
        }
    }

    /// Applies aquifer flooding or barrier rock placement to a block.
    /// If block is Air (carved cave space), backfills with fluid or barrier stone.
    pub fn apply_to_block(&self, x: i32, y: i32, z: i32, block: SurfaceBlock) -> SurfaceBlock {
        if block != SurfaceBlock::Air {
            return block;
        }

        match self.compute_substance_at(x, y, z) {
            AquiferSubstance::Solid => {
                if y < 0 {
                    SurfaceBlock::Deepslate
                } else {
                    SurfaceBlock::Stone
                }
            }
            AquiferSubstance::Fluid(AquiferFluidType::Water) => SurfaceBlock::Water,
            AquiferSubstance::Fluid(AquiferFluidType::Lava) => SurfaceBlock::Air, // Represented in surface blocks
            AquiferSubstance::Fluid(AquiferFluidType::Air) | AquiferSubstance::Air => SurfaceBlock::Air,
        }
    }
}

impl Aquifer for NoiseBasedAquifer {
    fn compute_substance(&self, x: i32, y: i32, z: i32) -> AquiferSubstance {
        self.compute_substance_at(x, y, z)
    }

    fn get_fluid_status(&self, x: i32, y: i32, z: i32) -> FluidStatus {
        let (status1, _, _) = self.sample_local_cells(x, y, z);
        status1
    }
}
