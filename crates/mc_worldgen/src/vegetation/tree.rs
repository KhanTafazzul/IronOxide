//! Procedural tree builder combining trunk placers and foliage placers.
//! REA Reference: net.minecraft.world.level.levelgen.feature.TreeFeature

use super::foliage::{BlobFoliagePlacer, SpruceFoliagePlacer};
use super::trunk::{BranchingMegaTrunkPlacer, StraightTrunkPlacer};
use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::LegacyRandom;

/// Procedural tree variant classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TreeType {
    Oak,
    Birch,
    Spruce,
    DarkOak,
    MegaJungle,
}

impl TreeType {
    /// Returns the wood log block used by this tree type.
    pub fn log_block(&self) -> SurfaceBlock {
        match self {
            TreeType::Oak => SurfaceBlock::OakLog,
            TreeType::Birch => SurfaceBlock::BirchLog,
            TreeType::Spruce => SurfaceBlock::SpruceLog,
            TreeType::DarkOak => SurfaceBlock::DarkOakLog,
            TreeType::MegaJungle => SurfaceBlock::JungleLog,
        }
    }

    /// Returns the leaf block used by this tree type.
    pub fn leaf_block(&self) -> SurfaceBlock {
        match self {
            TreeType::Oak => SurfaceBlock::OakLeaves,
            TreeType::Birch => SurfaceBlock::BirchLeaves,
            TreeType::Spruce => SurfaceBlock::SpruceLeaves,
            TreeType::DarkOak => SurfaceBlock::DarkOakLeaves,
            TreeType::MegaJungle => SurfaceBlock::JungleLeaves,
        }
    }
}

/// Tree placement orchestrator coordinating trunk generation and foliage placement.
pub struct TreePlacer;

impl TreePlacer {
    /// Validates if tree base can grow on the given soil.
    pub fn is_valid_soil(block: SurfaceBlock) -> bool {
        matches!(
            block,
            SurfaceBlock::GrassBlock | SurfaceBlock::Dirt | SurfaceBlock::Podzol | SurfaceBlock::CoarseDirt
        )
    }

    /// Generates a complete tree at the given origin.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `origin`: Tree root origin (X, Y, Z). The block at (X, Y - 1, Z) is the soil.
    /// * `tree_type`: Tree species.
    /// * `rng`: Java LCG pseudo-random number generator.
    ///
    /// Returns `(logs_placed, leaves_placed)` or `(0, 0)` if soil was invalid.
    pub fn place_tree(
        world: &mut impl OreBlockAccess,
        origin: (i32, i32, i32),
        tree_type: TreeType,
        rng: &mut LegacyRandom,
    ) -> (usize, usize) {
        let (x, y, z) = origin;
        let soil = world.get_block(x, y - 1, z);
        if !Self::is_valid_soil(soil) {
            return (0, 0);
        }

        let log_block = tree_type.log_block();
        let leaf_block = tree_type.leaf_block();

        match tree_type {
            TreeType::Oak => {
                let trunk = StraightTrunkPlacer::new(4, 2, 0); // 4-6 blocks
                let foliage = BlobFoliagePlacer::new(2, 3);
                let (attachments, logs) = trunk.place_trunk(world, origin, log_block, rng);
                let mut leaves = 0;
                for att in &attachments {
                    leaves += foliage.place_foliage(world, att, leaf_block, rng);
                }
                (logs, leaves)
            }
            TreeType::Birch => {
                let trunk = StraightTrunkPlacer::new(5, 2, 0); // 5-7 blocks
                let foliage = BlobFoliagePlacer::new(2, 3);
                let (attachments, logs) = trunk.place_trunk(world, origin, log_block, rng);
                let mut leaves = 0;
                for att in &attachments {
                    leaves += foliage.place_foliage(world, att, leaf_block, rng);
                }
                (logs, leaves)
            }
            TreeType::Spruce => {
                let trunk = StraightTrunkPlacer::new(6, 3, 0); // 6-9 blocks
                let foliage = SpruceFoliagePlacer::new(2, 6);
                let (attachments, logs) = trunk.place_trunk(world, origin, log_block, rng);
                let mut leaves = 0;
                for att in &attachments {
                    leaves += foliage.place_foliage(world, att, leaf_block, rng);
                }
                (logs, leaves)
            }
            TreeType::DarkOak => {
                let trunk = BranchingMegaTrunkPlacer::new(6, 2, 1); // 6-9 blocks 2x2 with branches
                let foliage = BlobFoliagePlacer::new(2, 3);
                let (attachments, logs) = trunk.place_trunk(world, origin, log_block, rng);
                let mut leaves = 0;
                for att in &attachments {
                    leaves += foliage.place_foliage(world, att, leaf_block, rng);
                }
                (logs, leaves)
            }
            TreeType::MegaJungle => {
                let trunk = BranchingMegaTrunkPlacer::new(12, 6, 2); // 12-20 blocks 2x2 with branches
                let foliage = BlobFoliagePlacer::new(3, 3);
                let (attachments, logs) = trunk.place_trunk(world, origin, log_block, rng);
                let mut leaves = 0;
                for att in &attachments {
                    leaves += foliage.place_foliage(world, att, leaf_block, rng);
                }
                (logs, leaves)
            }
        }
    }
}
