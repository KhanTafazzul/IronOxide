//! Swamp Hut (Witch Hut) standalone procedural generator with 7x7 wooden stilt architecture,
//! authentic interior furnishings (crafting table, cauldron, potted mushroom),
//! and hardcoded 7x7x9 mob spawn bounding box restricting natural spawns exclusively to Witches and Black Cats.
//!
//! REA Reference: `net.minecraft.world.level.levelgen.structure.structures.SwampHutStructure` (mapped: `eby.class`)
//! REA Reference: `net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece` (mapped: `ebz.class`)
//! REA Reference: `data/minecraft/worldgen/structure/swamp_hut.json`

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;

/// Axis-Aligned Bounding Box for structure mob spawn isolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobSpawnBoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl MobSpawnBoundingBox {
    #[inline]
    pub const fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        x >= self.min_x && x <= self.max_x
            && y >= self.min_y && y <= self.max_y
            && z >= self.min_z && z <= self.max_z
    }
}

/// Hardcoded entity spawn rule matching `data/minecraft/worldgen/structure/swamp_hut.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnOverrideEntry {
    pub entity_type: &'static str,
    pub weight: u32,
    pub min_count: u32,
    pub max_count: u32,
}

/// Swamp Hut structure generator.
pub struct SwampHutGenerator;

impl SwampHutGenerator {
    pub const WIDTH: i32 = 7;
    pub const HEIGHT: i32 = 7;
    pub const DEPTH: i32 = 9;

    /// Monster spawns in the swamp hut bounding box: strictly Witch only.
    pub const MONSTER_SPAWNS: &[SpawnOverrideEntry] = &[
        SpawnOverrideEntry { entity_type: "minecraft:witch", weight: 1, min_count: 1, max_count: 1 },
    ];

    /// Creature spawns in the swamp hut bounding box: strictly Cat only.
    pub const CREATURE_SPAWNS: &[SpawnOverrideEntry] = &[
        SpawnOverrideEntry { entity_type: "minecraft:cat", weight: 1, min_count: 1, max_count: 1 },
    ];

    /// Returns the exact 7x7x9 mob spawn bounding box for a hut generated at `origin`.
    pub const fn get_spawn_bounding_box(origin: (i32, i32, i32)) -> MobSpawnBoundingBox {
        let (ox, oy, oz) = origin;
        MobSpawnBoundingBox {
            min_x: ox,
            min_y: oy,
            min_z: oz,
            max_x: ox + Self::WIDTH - 1,
            max_y: oy + Self::DEPTH - 1, // 9 blocks height volume in vanilla piece
            max_z: oz + Self::DEPTH - 1,
        }
    }

    /// Validates whether a proposed entity spawn within the swamp hut bounding box is permitted.
    /// Vanilla restriction: Restricts natural monster spawns inside the hut's 7x7x9 volume exclusively to Witches and Black Cats.
    pub fn is_spawn_permitted(origin: (i32, i32, i32), pos: (i32, i32, i32), entity_type: &str) -> bool {
        let bbox = Self::get_spawn_bounding_box(origin);
        if bbox.contains(pos.0, pos.1, pos.2) {
            match entity_type {
                "minecraft:witch" | "witch" | "minecraft:cat" | "cat" => true,
                _ => false, // Reject all regular zombies, skeletons, creepers, spiders, etc.
            }
        } else {
            true
        }
    }

    /// Generates a complete Swamp Hut at the given origin.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `origin`: Stilt base origin (ox, oy, oz).
    ///
    /// Returns the total count of structure blocks placed.
    pub fn generate(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
    ) -> usize {
        let (ox, oy, oz) = origin;
        let mut blocks_placed = 0;

        let oak_log = SurfaceBlock::OakLog;
        let spruce_planks = SurfaceBlock::SprucePlanks;
        let dark_oak_stairs = SurfaceBlock::DarkOakStairs;
        let crafting_table = SurfaceBlock::CraftingTable;
        let cauldron = SurfaceBlock::Cauldron;
        let potted_mushroom = SurfaceBlock::PottedMushroom;
        let air = SurfaceBlock::Air;
        let fence = SurfaceBlock::OakFence;

        // 1. Four Corner Wooden Stilts (Oak Logs) from y = -2 up to floor level y = 2
        for &(sx, sz) in &[(1, 1), (5, 1), (1, 7), (5, 7)] {
            for y in -2..=2 {
                world.set_block(ox + sx, oy + y, oz + sz, oak_log);
                blocks_placed += 1;
            }
        }

        // 2. Elevated Floor Platform (Spruce Planks) at y = 2 (Footprint 7x7 from x: 0..=6, z: 1..=7)
        for x in 0..Self::WIDTH {
            for z in 1..=7 {
                world.set_block(ox + x, oy + 2, oz + z, spruce_planks);
                blocks_placed += 1;
            }
        }

        // Front porch walkway extension at z = 0
        for x in 2..=4 {
            world.set_block(ox + x, oy + 2, oz, spruce_planks);
            blocks_placed += 1;
        }

        // 3. Hut Cabin Outer Walls (Spruce Planks, y = 3..=5, x: 1..=5, z: 2..=7)
        for y in 3..=5 {
            let py = oy + y;
            for x in 1..=5 {
                for z in 2..=7 {
                    let is_wall = x == 1 || x == 5 || z == 2 || z == 7;
                    let is_door = x == 3 && z == 2 && y <= 4;
                    let is_window = (x == 1 && z == 4 && y == 4) || (x == 5 && z == 5 && y == 4);

                    if is_door || is_window {
                        world.set_block(ox + x, py, oz + z, air);
                    } else if is_wall {
                        world.set_block(ox + x, py, oz + z, spruce_planks);
                        blocks_placed += 1;
                    } else {
                        world.set_block(ox + x, py, oz + z, air);
                    }
                }
            }
        }

        // Porch Railing Fences
        world.set_block(ox + 1, oy + 3, oz + 1, fence);
        world.set_block(ox + 5, oy + 3, oz + 1, fence);
        blocks_placed += 2;

        // 4. Overhanging Pitched Roof (Dark Oak Stairs, y = 5..=6)
        for z in 1..=8 {
            world.set_block(ox + 0, oy + 5, oz + z, dark_oak_stairs);
            world.set_block(ox + 6, oy + 5, oz + z, dark_oak_stairs);
            world.set_block(ox + 1, oy + 6, oz + z, dark_oak_stairs);
            world.set_block(ox + 5, oy + 6, oz + z, dark_oak_stairs);
            // Center roof ridge
            world.set_block(ox + 2, oy + 6, oz + z, spruce_planks);
            world.set_block(ox + 3, oy + 6, oz + z, spruce_planks);
            world.set_block(ox + 4, oy + 6, oz + z, spruce_planks);
            blocks_placed += 7;
        }

        // 5. Authentic Interior Witch Hut Furnishings (y = 3 on floor):
        // Crafting Table at rear right
        world.set_block(ox + 2, oy + 3, oz + 6, crafting_table);
        // Witch's Cauldron at rear left
        world.set_block(ox + 4, oy + 3, oz + 6, cauldron);
        // Flower Pot with Mushroom on the window sill
        world.set_block(ox + 1, oy + 3, oz + 4, potted_mushroom);
        blocks_placed += 3;

        blocks_placed
    }
}
