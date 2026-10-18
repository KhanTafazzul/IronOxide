//! Chunk carving applicator evaluating 3D noise carvers and applying block replacement rules.
//! REA Reference: net.minecraft.world.level.levelgen.carver.CaveWorldCarver (dqs.class)
//! REA Reference: net.minecraft.world.level.levelgen.carver.WorldCarver (dqv.class)

use super::config::{carved_fluid_or_air, is_carver_replaceable, CarvedState, BEDROCK_CEILING_Y};
use super::noise_caves::{CaveCarverRouter, CaveType};
use crate::biomes::SurfaceBlock;

/// Result of evaluating a voxel against the 3D cave carvers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarveResult {
    pub carved: bool,
    pub new_block: SurfaceBlock,
    pub cave_type: Option<CaveType>,
}

/// Carves a single block coordinate (x, y, z) given its initial solid block state.
///
/// Rules:
/// 1. Bedrock (Y <= -60 or Bedrock block) is strictly protected and NEVER carved.
/// 2. If the block is not carver-replaceable (e.g. Bedrock), carving is skipped.
/// 3. If carved by 3D noise (Cheese, Spaghetti, or Noodle):
///    - Above sea level (Y > 63): replaced with Air.
///    - Below sea level (Y <= 63): replaced with Water (subterranean aquifer).
///    - Deep cavern floor (Y <= -55): replaced with Lava pool.
#[inline]
pub fn carve_block(
    x: i32,
    y: i32,
    z: i32,
    current_block: SurfaceBlock,
    router: &CaveCarverRouter,
) -> CarveResult {
    // Bedrock floor is NEVER replaced
    if y <= BEDROCK_CEILING_Y || current_block == SurfaceBlock::Bedrock {
        return CarveResult {
            carved: false,
            new_block: current_block,
            cave_type: None,
        };
    }

    // Only replaceable blocks (stone, deepslate, dirt, sand, gravel, terracotta, etc.) are carved
    if !is_carver_replaceable(current_block) {
        return CarveResult {
            carved: false,
            new_block: current_block,
            cave_type: None,
        };
    }

    // Sample 3D noise cave router (Cheese, Spaghetti, Noodle)
    if let Some(cave_type) = router.sample_cave(x as f64, y as f64, z as f64) {
        let new_block = match carved_fluid_or_air(y) {
            CarvedState::Air => SurfaceBlock::Air,
            CarvedState::Water => SurfaceBlock::Water,
            CarvedState::Lava => SurfaceBlock::Air, // In surface block representation, Air/Lava
        };

        CarveResult {
            carved: true,
            new_block,
            cave_type: Some(cave_type),
        }
    } else {
        CarveResult {
            carved: false,
            new_block: current_block,
            cave_type: None,
        }
    }
}

/// Helper to carve an entire 16x16 chunk slice or column array.
pub fn carve_chunk_slice<F, G>(
    chunk_x: i32,
    chunk_z: i32,
    y: i32,
    mut get_block: F,
    mut set_block: G,
    router: &CaveCarverRouter,
) where
    F: FnMut(usize, usize) -> SurfaceBlock,
    G: FnMut(usize, usize, SurfaceBlock),
{
    // Never carve below bedrock ceiling
    if y <= BEDROCK_CEILING_Y {
        return;
    }

    for lx in 0..16 {
        for lz in 0..16 {
            let wx = chunk_x * 16 + lx as i32;
            let wz = chunk_z * 16 + lz as i32;
            let current = get_block(lx, lz);

            let res = carve_block(wx, y, wz, current, router);
            if res.carved {
                set_block(lx, lz, res.new_block);
            }
        }
    }
}
