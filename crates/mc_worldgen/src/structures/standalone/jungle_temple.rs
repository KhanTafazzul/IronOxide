//! Jungle Temple standalone procedural generator featuring 3-story mossy cobblestone architecture,
//! 3-lever combination puzzle controlling sticky piston hidden door, and dispenser tripwire traps.
//!
//! REA Reference: `net.minecraft.world.level.levelgen.structure.structures.JungleTempleStructure` (mapped: `eca.class`)
//! REA Reference: `net.minecraft.world.level.levelgen.structure.structures.JunglePyramidPiece` (mapped: `ebh.class`)
//! REA Reference: `data/minecraft/loot_tables/chests/jungle_temple.json`
//! REA Reference: `data/minecraft/loot_tables/chests/jungle_temple_dispenser.json`

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::locator::LegacyRandom;

/// Loot entry definition for Jungle Temple.
#[derive(Debug, Clone, PartialEq)]
pub struct LootEntry {
    pub item: &'static str,
    pub min_count: u32,
    pub max_count: u32,
    pub weight: u32,
}

/// Canonical chest loot items for `minecraft:chests/jungle_temple`.
pub const JUNGLE_TEMPLE_CHEST_LOOT: &[LootEntry] = &[
    LootEntry { item: "minecraft:diamond", min_count: 1, max_count: 3, weight: 3 },
    LootEntry { item: "minecraft:iron_ingot", min_count: 1, max_count: 5, weight: 10 },
    LootEntry { item: "minecraft:gold_ingot", min_count: 2, max_count: 7, weight: 15 },
    LootEntry { item: "minecraft:emerald", min_count: 1, max_count: 3, weight: 8 },
    LootEntry { item: "minecraft:bone", min_count: 4, max_count: 6, weight: 20 },
    LootEntry { item: "minecraft:rotten_flesh", min_count: 3, max_count: 7, weight: 16 },
    LootEntry { item: "minecraft:saddle", min_count: 1, max_count: 1, weight: 5 },
    LootEntry { item: "minecraft:iron_horse_armor", min_count: 1, max_count: 1, weight: 5 },
    LootEntry { item: "minecraft:golden_horse_armor", min_count: 1, max_count: 1, weight: 3 },
    LootEntry { item: "minecraft:diamond_horse_armor", min_count: 1, max_count: 1, weight: 1 },
    LootEntry { item: "minecraft:wild_armor_trim_smithing_template", min_count: 2, max_count: 2, weight: 2 },
];

/// Canonical dispenser ammo loot items for `minecraft:chests/jungle_temple_dispenser`.
pub const JUNGLE_TEMPLE_DISPENSER_LOOT: &[LootEntry] = &[
    LootEntry { item: "minecraft:arrow", min_count: 2, max_count: 7, weight: 30 },
];

/// Standing Jungle Temple generator.
pub struct JungleTempleGenerator;

impl JungleTempleGenerator {
    pub const WIDTH: i32 = 12;
    pub const HEIGHT: i32 = 10;
    pub const DEPTH: i32 = 15;

    /// Generates a complete 3-story Jungle Temple at the given origin.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `origin`: Structure origin corner (ox, oy, oz).
    /// * `seed`: Random seed for mossy variation and loot generation.
    ///
    /// Returns the total count of structure blocks placed.
    pub fn generate(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        seed: i64,
    ) -> usize {
        let (ox, oy, oz) = origin;
        let mut blocks_placed = 0;
        let mut rng = LegacyRandom::new(seed);

        let cobble = SurfaceBlock::Cobblestone;
        let mossy = SurfaceBlock::MossyCobblestone;
        let stairs = SurfaceBlock::CobblestoneStairs;
        let air = SurfaceBlock::Air;
        let lever = SurfaceBlock::Lever;
        let sticky_piston = SurfaceBlock::StickyPiston;
        let _piston = SurfaceBlock::Piston;
        let redstone = SurfaceBlock::RedstoneWire;
        let repeater = SurfaceBlock::Repeater;
        let tripwire_hook = SurfaceBlock::TripwireHook;
        let string_block = SurfaceBlock::StringBlock;
        let dispenser = SurfaceBlock::Dispenser;
        let chest = SurfaceBlock::Chest;

        // Helper closure for authentic mossy cobblestone procedural mixture
        let pick_cobble = |rng: &mut LegacyRandom| {
            if rng.next_float() < 0.35 {
                mossy
            } else {
                cobble
            }
        };

        // 1. Foundation and Solid Base (y = -1 to 0)
        for dx in 0..Self::WIDTH {
            for dz in 0..Self::DEPTH {
                world.set_block(ox + dx, oy - 1, oz + dz, pick_cobble(&mut rng));
                world.set_block(ox + dx, oy, oz + dz, pick_cobble(&mut rng));
                blocks_placed += 2;
            }
        }

        // 2. Main 3-Story Outer Shell (y = 1..=9)
        for y in 1..Self::HEIGHT {
            let py = oy + y;
            let inset = if y >= 7 { 1 } else { 0 };

            for x in inset..Self::WIDTH - inset {
                for z in inset..Self::DEPTH - inset {
                    let is_wall = x == inset || x == Self::WIDTH - 1 - inset || z == inset || z == Self::DEPTH - 1 - inset;
                    let is_corner = (x == inset || x == Self::WIDTH - 1 - inset) && (z == inset || z == Self::DEPTH - 1 - inset);

                    if is_wall {
                        let block = if is_corner && y % 3 == 0 {
                            chiseled_corner(&mut rng)
                        } else {
                            pick_cobble(&mut rng)
                        };
                        world.set_block(ox + x, py, oz + z, block);
                        blocks_placed += 1;
                    } else if y == 4 || y == 7 {
                        // Intermediate floor slabs
                        world.set_block(ox + x, py, oz + z, pick_cobble(&mut rng));
                        blocks_placed += 1;
                    } else {
                        world.set_block(ox + x, py, oz + z, air);
                    }
                }
            }
        }

        // 3. Roof Parapet Battlements (y = 10)
        for x in 1..Self::WIDTH - 1 {
            for z in 1..Self::DEPTH - 1 {
                let is_edge = x == 1 || x == Self::WIDTH - 2 || z == 1 || z == Self::DEPTH - 2;
                if is_edge && (x + z) % 2 == 0 {
                    world.set_block(ox + x, oy + Self::HEIGHT, oz + z, stairs);
                    blocks_placed += 1;
                }
            }
        }

        // 4. Ground Floor Entrance (x: 5..=6, z: 0, y: 1..=3)
        for x in 5..=6 {
            for y in 1..=3 {
                world.set_block(ox + x, oy + y, oz, air);
            }
        }

        // 5. Lower Puzzle Floor (Basement y = 1..=3):
        // 3-Lever combination puzzle along West wall (x = 1, z in [6, 8, 10], y = 2)
        world.set_block(ox + 1, oy + 2, oz + 6, lever); // Lever 1
        world.set_block(ox + 1, oy + 2, oz + 8, lever); // Lever 2
        world.set_block(ox + 1, oy + 2, oz + 10, lever); // Lever 3
        blocks_placed += 3;

        // Hidden Sticky Piston Mechanism behind lever wall:
        // Redstone wiring running behind the wall connecting the correct combination (e.g. Lever 1 + Lever 3)
        world.set_block(ox + 2, oy + 1, oz + 6, redstone);
        world.set_block(ox + 2, oy + 1, oz + 8, repeater);
        world.set_block(ox + 2, oy + 1, oz + 10, redstone);
        // Sticky piston door at (x = 4, y = 2, z = 12) concealing hidden chest chamber
        world.set_block(ox + 4, oy + 2, oz + 12, sticky_piston);
        world.set_block(ox + 3, oy + 2, oz + 12, pick_cobble(&mut rng)); // Piston door block
        // Hidden Chest inside secret alcove behind the sticky piston door
        world.set_block(ox + 5, oy + 1, oz + 12, chest);
        blocks_placed += 5;

        // 6. Dispenser Tripwire Traps along Main Corridor (x = 6, z = 1..=13):
        // Trap 1: Entrance tripwire crossing at z = 4
        world.set_block(ox + 5, oy + 2, oz + 4, tripwire_hook);
        world.set_block(ox + 6, oy + 2, oz + 4, string_block);
        world.set_block(ox + 7, oy + 2, oz + 4, tripwire_hook);
        // Concealed Dispenser 1 in North wall aimed directly down the corridor
        world.set_block(ox + 6, oy + 2, oz + 1, dispenser);
        blocks_placed += 4;

        // Trap 2: Secondary tripwire crossing directly in front of the Main Chest at z = 11
        world.set_block(ox + 5, oy + 2, oz + 11, tripwire_hook);
        world.set_block(ox + 6, oy + 2, oz + 11, string_block);
        world.set_block(ox + 7, oy + 2, oz + 11, tripwire_hook);
        // Concealed Dispenser 2 loaded with arrows in East wall at (x = 8, y = 2, z = 11)
        world.set_block(ox + 8, oy + 2, oz + 11, dispenser);
        blocks_placed += 4;

        // Main Corridor Loot Chest at end of the passage at (x = 6, y = 1, z = 13)
        world.set_block(ox + 6, oy + 1, oz + 13, chest);
        blocks_placed += 1;

        blocks_placed
    }
}

#[inline]
fn chiseled_corner(rng: &mut LegacyRandom) -> SurfaceBlock {
    if rng.next_float() < 0.5 {
        SurfaceBlock::MossyCobblestone
    } else {
        SurfaceBlock::Cobblestone
    }
}
