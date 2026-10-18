//! Buried Treasure procedural generator placing buried loot chests on beaches.
//! REA Reference: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasureStructure (ejk.class)

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;

/// Buried Treasure generator.
pub struct BuriedTreasureGenerator;

impl BuriedTreasureGenerator {
    /// Generates a Buried Treasure chest at the target coordinates buried beneath sand or gravel.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `origin`: Coordinate (X, Y, Z) where Y is the top beach surface block.
    ///
    /// Returns count of blocks placed (1 for chest, plus sand cover).
    pub fn generate(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
    ) -> usize {
        let (x, surface_y, z) = origin;
        // Bury chest 1-2 blocks below surface
        let chest_y = (surface_y - 2).max(-60);

        // Place Chest
        world.set_block(x, chest_y, z, SurfaceBlock::Chest);

        // Cover with Sand above
        world.set_block(x, chest_y + 1, z, SurfaceBlock::Sand);
        world.set_block(x, surface_y, z, SurfaceBlock::Sand);

        3
    }
}
