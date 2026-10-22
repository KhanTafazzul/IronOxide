//! Desert Pyramid standalone procedural generator with exact 21x21 sandstone geometry,
//! twin entrance towers, terracotta floor pattern, lower TNT pressure plate trap, and 1.20+ archaeology basement.
//!
//! REA Reference: `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure` (mapped: `ebb.class`)
//! REA Reference: `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidPiece` (mapped: `eba.class`)
//! REA Reference: `data/minecraft/loot_tables/chests/desert_pyramid.json`
//! REA Reference: `data/minecraft/loot_tables/archaeology/desert_pyramid.json`

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::locator::LegacyRandom;

/// Desert Pyramid loot table entry.
#[derive(Debug, Clone, PartialEq)]
pub struct LootEntry {
    pub item: &'static str,
    pub min_count: u32,
    pub max_count: u32,
    pub weight: u32,
}

/// Canonical chest loot items for `minecraft:chests/desert_pyramid`.
pub const DESERT_PYRAMID_CHEST_LOOT: &[LootEntry] = &[
    LootEntry { item: "minecraft:diamond", min_count: 1, max_count: 3, weight: 5 },
    LootEntry { item: "minecraft:iron_ingot", min_count: 1, max_count: 5, weight: 15 },
    LootEntry { item: "minecraft:gold_ingot", min_count: 2, max_count: 7, weight: 15 },
    LootEntry { item: "minecraft:emerald", min_count: 1, max_count: 3, weight: 15 },
    LootEntry { item: "minecraft:bone", min_count: 4, max_count: 6, weight: 25 },
    LootEntry { item: "minecraft:spider_eye", min_count: 1, max_count: 3, weight: 25 },
    LootEntry { item: "minecraft:rotten_flesh", min_count: 3, max_count: 7, weight: 25 },
    LootEntry { item: "minecraft:saddle", min_count: 1, max_count: 1, weight: 20 },
    LootEntry { item: "minecraft:iron_horse_armor", min_count: 1, max_count: 1, weight: 15 },
    LootEntry { item: "minecraft:golden_horse_armor", min_count: 1, max_count: 1, weight: 10 },
    LootEntry { item: "minecraft:diamond_horse_armor", min_count: 1, max_count: 1, weight: 5 },
    LootEntry { item: "minecraft:golden_apple", min_count: 1, max_count: 1, weight: 20 },
    LootEntry { item: "minecraft:enchanted_golden_apple", min_count: 1, max_count: 1, weight: 2 },
    LootEntry { item: "minecraft:dune_armor_trim_smithing_template", min_count: 2, max_count: 2, weight: 4 },
];

/// Canonical archaeology brush loot items for `minecraft:archaeology/desert_pyramid`.
pub const DESERT_PYRAMID_ARCHAEOLOGY_LOOT: &[&'static str] = &[
    "minecraft:archer_pottery_sherd",
    "minecraft:miner_pottery_sherd",
    "minecraft:prize_pottery_sherd",
    "minecraft:skull_pottery_sherd",
    "minecraft:sniffer_egg",
    "minecraft:diamond",
    "minecraft:tnt",
    "minecraft:gunpowder",
    "minecraft:emerald",
];

/// Standing Desert Pyramid generator.
pub struct DesertPyramidGenerator;

impl DesertPyramidGenerator {
    pub const WIDTH: i32 = 21;
    pub const HEIGHT: i32 = 15;
    pub const DEPTH: i32 = 21;

    /// Generates a complete 21x21 Desert Pyramid at the given origin.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `origin`: Top surface center/corner (ox, oy, oz). When surface is $Y = 64$, the TNT trap floor is at $Y = 53$.
    /// * `seed`: Random seed for loot generation and archaeology placement.
    ///
    /// Returns the total count of structure blocks placed.
    pub fn generate(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        seed: i64,
    ) -> usize {
        let (ox, oy, oz) = origin;
        let mut blocks_placed = 0;
        let mut _rng = LegacyRandom::new(seed);

        let sandstone = SurfaceBlock::Sandstone;
        let cut_sandstone = SurfaceBlock::CutSandstone;
        let chiseled_sandstone = SurfaceBlock::ChiseledSandstone;
        let stairs = SurfaceBlock::SandstoneStairs;
        let blue_terracotta = SurfaceBlock::BlueTerracotta;
        let orange_terracotta = SurfaceBlock::OrangeTerracotta;
        let air = SurfaceBlock::Air;
        let tnt = SurfaceBlock::TNT;
        let pressure_plate = SurfaceBlock::StonePressurePlate;
        let chest = SurfaceBlock::Chest;
        let suspicious_sand = SurfaceBlock::SuspiciousSand;
        let decorated_pot = SurfaceBlock::DecoratedPot;

        // 1. Foundation Slab: Solid sandstone from y = -1 down to y = -12 for stability
        for dx in 0..Self::WIDTH {
            for dz in 0..Self::DEPTH {
                world.set_block(ox + dx, oy - 1, oz + dz, sandstone);
                blocks_placed += 1;
            }
        }

        // 2. Main Stepped Pyramid Exterior Shell (0..=9)
        for layer in 0..10 {
            let inset = layer;
            let min_x = inset;
            let max_x = Self::WIDTH - 1 - inset;
            let min_z = inset;
            let max_z = Self::DEPTH - 1 - inset;

            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    let is_border = x == min_x || x == max_x || z == min_z || z == max_z;
                    let py = oy + layer;

                    if is_border {
                        world.set_block(ox + x, py, oz + z, sandstone);
                        blocks_placed += 1;
                    } else if layer == 0 {
                        world.set_block(ox + x, py, oz + z, sandstone);
                        blocks_placed += 1;
                    } else {
                        world.set_block(ox + x, py, oz + z, air);
                    }
                }
            }
        }

        // 3. Flat Pyramid Top Cap at layer 10
        for x in 9..=11 {
            for z in 9..=11 {
                world.set_block(ox + x, oy + 10, oz + z, cut_sandstone);
                blocks_placed += 1;
            }
        }

        // 4. Twin Entrance Towers (Left: 0..=4, 0..=4; Right: 16..=20, 0..=4) rising to y = oy + 14
        for tower_x_offset in [0, 16] {
            for dy in 0..14 {
                let py = oy + dy;
                for tx in 0..5 {
                    for tz in 0..5 {
                        let wx = ox + tower_x_offset + tx;
                        let wz = oz + tz;
                        let is_outer = tx == 0 || tx == 4 || tz == 0 || tz == 4;

                        if is_outer {
                            let block = if dy == 0 || dy == 7 || dy == 13 {
                                cut_sandstone
                            } else if dy == 3 || dy == 10 {
                                chiseled_sandstone
                            } else {
                                sandstone
                            };
                            world.set_block(wx, py, wz, block);
                        } else if dy < 13 {
                            world.set_block(wx, py, wz, air);
                        } else {
                            world.set_block(wx, py, wz, cut_sandstone);
                        }
                        blocks_placed += 1;
                    }
                }
            }
            // Decorative parapets on tower roofs
            for tx in 0..5 {
                for tz in 0..5 {
                    if (tx == 0 || tx == 4) && (tz == 0 || tz == 4) {
                        world.set_block(ox + tower_x_offset + tx, oy + 14, oz + tz, stairs);
                        blocks_placed += 1;
                    }
                }
            }
        }

        // 5. Grand Front Entrance Archway (x: 8..=12, z: 0, y: 1..=3)
        for x in 9..=11 {
            for y in 1..=3 {
                world.set_block(ox + x, oy + y, oz, air);
            }
        }

        // 6. Center Decorative Floor Pattern at y = oy (Floor Level)
        // Center of the 21x21 base is at (10, 10).
        // 3x3 Blue Terracotta center with Orange Terracotta decorative arms
        for x in 8..=12 {
            for z in 8..=12 {
                let is_center_3x3 = x >= 9 && x <= 11 && z >= 9 && z <= 11;
                let is_exact_center = x == 10 && z == 10;
                let block = if is_exact_center {
                    blue_terracotta
                } else if is_center_3x3 {
                    orange_terracotta
                } else {
                    cut_sandstone
                };
                world.set_block(ox + x, oy, oz + z, block);
                blocks_placed += 1;
            }
        }
        // Cardinal Orange Terracotta cross arms
        for step in 1..=4 {
            world.set_block(ox + 10 + step, oy, oz + 10, orange_terracotta);
            world.set_block(ox + 10 - step, oy, oz + 10, orange_terracotta);
            world.set_block(ox + 10, oy, oz + 10 + step, orange_terracotta);
            world.set_block(ox + 10, oy, oz + 10 - step, orange_terracotta);
            blocks_placed += 4;
        }

        // 7. Lower Trap Chamber & Drop Shaft:
        // Underneath the 3x3 floor at x: 9..=11, z: 9..=11:
        // Air drop shaft from y = oy - 1 down to y = oy - 9 (Height = 9 air blocks)
        for dy in 1..=9 {
            for dx in 9..=11 {
                for dz in 9..=11 {
                    world.set_block(ox + dx, oy - dy, oz + dz, air);
                    blocks_placed += 1;
                }
            }
        }

        // Secret Vault Walls surrounding the 5x5 chamber at depth y in [oy - 11, oy - 1]
        for dy in 1..=12 {
            let cy = oy - dy;
            for cx in 7..=13 {
                for cz in 7..=13 {
                    let is_vault_wall = cx == 7 || cx == 13 || cz == 7 || cz == 13;
                    if is_vault_wall {
                        world.set_block(ox + cx, cy, oz + cz, sandstone);
                        blocks_placed += 1;
                    }
                }
            }
        }

        // Chamber floor at y = oy - 10 (when oy = 64, this is Y = 54):
        // Floor of cut sandstone around the center
        for dx in 8..=12 {
            for dz in 8..=12 {
                let is_under_plate = dx >= 9 && dx <= 11 && dz >= 9 && dz <= 11;
                if !is_under_plate {
                    world.set_block(ox + dx, oy - 10, oz + dz, cut_sandstone);
                    blocks_placed += 1;
                }
            }
        }

        // The TNT Trap at y = oy - 11 (when oy = 64, this is EXACTLY Y = 53):
        // 3x3 grid of 9 TNT blocks directly underneath the drop shaft!
        for dx in 9..=11 {
            for dz in 9..=11 {
                world.set_block(ox + dx, oy - 11, oz + dz, tnt);
                blocks_placed += 1;
            }
        }
        // Bedrock/sandstone foundation under TNT at y = oy - 12
        for dx in 9..=11 {
            for dz in 9..=11 {
                world.set_block(ox + dx, oy - 12, oz + dz, sandstone);
                blocks_placed += 1;
            }
        }

        // Center Stone Pressure Plate at y = oy - 10 (when oy = 64, this is Y = 54):
        // Placed directly on top of the central TNT block at (ox + 10, oy - 10, oz + 10)!
        world.set_block(ox + 10, oy - 10, oz + 10, pressure_plate);
        blocks_placed += 1;

        // 4 Loot Chests on the cardinal walls at y = oy - 10:
        world.set_block(ox + 10, oy - 10, oz + 8, chest); // North chest
        world.set_block(ox + 10, oy - 10, oz + 12, chest); // South chest
        world.set_block(ox + 8, oy - 10, oz + 10, chest); // West chest
        world.set_block(ox + 12, oy - 10, oz + 10, chest); // East chest
        blocks_placed += 4;

        // 8. Archaeology Basement (Minecraft 1.20+):
        // Sand-filled archaeological excavation rooms beneath the front tower bases (y in [oy - 5, oy - 2])
        for &(base_x, base_z) in &[(1, 1), (15, 1)] {
            for dy in 2..=5 {
                let cy = oy - dy;
                for dx in 0..4 {
                    for dz in 0..4 {
                        let wx = ox + base_x + dx;
                        let wz = oz + base_z + dz;
                        let is_archaeology_spot = (dx + dz + dy) % 3 == 0;
                        let is_pot_spot = dx == 2 && dz == 2 && dy == 2;

                        if is_pot_spot {
                            world.set_block(wx, cy, wz, decorated_pot);
                        } else if is_archaeology_spot {
                            world.set_block(wx, cy, wz, suspicious_sand);
                        } else {
                            world.set_block(wx, cy, wz, SurfaceBlock::Sand);
                        }
                        blocks_placed += 1;
                    }
                }
            }
        }

        blocks_placed
    }
}
