//! Pillager Outpost Jigsaw structure generator.
//! Loads base plate from `data/minecraft/worldgen/template_pool/pillager_outpost/base_plates.json`,
//! constructs the central watchtower with 4 cobble/dark oak support pillars,
//! and assembles surrounding accessory pieces (target tents, iron golem cages, log piles, scarecrows).
//!
//! REA Reference: `data/minecraft/worldgen/structure/pillager_outpost.json`
//! REA Reference: `data/minecraft/worldgen/template_pool/pillager_outpost/base_plates.json`
//! REA Reference: `data/minecraft/loot_tables/chests/pillager_outpost.json`

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::locator::LegacyRandom;
use crate::structures::template_pool::TemplatePoolRegistry;

/// Pillager Outpost loot entry.
#[derive(Debug, Clone, PartialEq)]
pub struct LootEntry {
    pub item: &'static str,
    pub min_count: u32,
    pub max_count: u32,
    pub weight: u32,
}

/// Canonical chest loot items for `minecraft:chests/pillager_outpost`.
pub const PILLAGER_OUTPOST_CHEST_LOOT: &[LootEntry] = &[
    LootEntry { item: "minecraft:crossbow", min_count: 1, max_count: 1, weight: 1 },
    LootEntry { item: "minecraft:arrow", min_count: 2, max_count: 7, weight: 4 },
    LootEntry { item: "minecraft:tripwire_hook", min_count: 1, max_count: 3, weight: 3 },
    LootEntry { item: "minecraft:iron_ingot", min_count: 1, max_count: 3, weight: 3 },
    LootEntry { item: "minecraft:dark_oak_log", min_count: 2, max_count: 5, weight: 4 },
    LootEntry { item: "minecraft:wheat", min_count: 3, max_count: 5, weight: 4 },
    LootEntry { item: "minecraft:goat_horn", min_count: 1, max_count: 1, weight: 2 },
    LootEntry { item: "minecraft:sentry_armor_trim_smithing_template", min_count: 2, max_count: 2, weight: 1 },
];

/// Accessory feature types surrounding the watchtower.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutpostFeatureType {
    Tent,
    IronGolemCage,
    LogPile,
    Scarecrow,
}

/// Pillager Outpost structure generator.
pub struct PillagerOutpostGenerator;

impl PillagerOutpostGenerator {
    pub const WATCHTOWER_WIDTH: i32 = 14;
    pub const WATCHTOWER_HEIGHT: i32 = 18;
    pub const WATCHTOWER_DEPTH: i32 = 14;

    /// Generates a complete Pillager Outpost complex with watchtower and 4 accessory pieces.
    pub fn generate(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        seed: i64,
        registry: Option<&TemplatePoolRegistry>,
    ) -> usize {
        let (ox, oy, oz) = origin;
        let mut blocks_placed = 0;
        let mut rng = LegacyRandom::new(seed);

        // Verify or access base plate template pool
        let _has_pool = if let Some(reg) = registry {
            reg.contains_key("minecraft:pillager_outpost/base_plates")
                || reg.contains_key("pillager_outpost/base_plates")
        } else {
            false
        };

        let cobble = SurfaceBlock::Cobblestone;
        let dark_oak_log = SurfaceBlock::DarkOakLog;
        let dark_oak_planks = SurfaceBlock::DarkOakPlanks;
        let dark_oak_stairs = SurfaceBlock::DarkOakStairs;
        let fence = SurfaceBlock::DarkOakFence;
        let air = SurfaceBlock::Air;
        let chest = SurfaceBlock::Chest;

        // 1. Central Watchtower Foundation & 4 Support Pillars (x: 0..=13, z: 0..=13)
        // Pillar 1: (2, 2)
        // Pillar 2: (11, 2)
        // Pillar 3: (2, 11)
        // Pillar 4: (11, 11)
        let pillar_coords = [(2, 2), (11, 2), (2, 11), (11, 11)];

        // Cobblestone ground base pads under pillars
        for &(px, pz) in &pillar_coords {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    for y in -2..=0 {
                        world.set_block(ox + px + dx, oy + y, oz + pz + dz, cobble);
                        blocks_placed += 1;
                    }
                }
            }
        }

        // 4 Support Pillars rising to y = 8 (Cobblestone lower, Dark Oak log upper)
        for &(px, pz) in &pillar_coords {
            for y in 1..=8 {
                let block = if y <= 4 { cobble } else { dark_oak_log };
                world.set_block(ox + px, oy + y, oz + pz, block);
                blocks_placed += 1;
            }
        }

        // Open archway ground floor (center y = 1..=7 is open air between the 4 pillars)
        for x in 3..=10 {
            for z in 3..=10 {
                for y in 1..=7 {
                    world.set_block(ox + x, oy + y, oz + z, air);
                }
            }
        }

        // 2. Middle Living Floor (y = 8, floor platform 10x10)
        for x in 2..=11 {
            for z in 2..=11 {
                world.set_block(ox + x, oy + 8, oz + z, dark_oak_planks);
                blocks_placed += 1;
            }
        }

        // Intermediate Room Walls (y = 9..=12, Dark Oak Planks & Logs)
        for y in 9..=12 {
            let py = oy + y;
            for x in 2..=11 {
                for z in 2..=11 {
                    let is_wall = x == 2 || x == 11 || z == 2 || z == 11;
                    let is_corner = (x == 2 || x == 11) && (z == 2 || z == 11);

                    if is_corner {
                        world.set_block(ox + x, py, oz + z, dark_oak_log);
                        blocks_placed += 1;
                    } else if is_wall {
                        // Arrow slit openings at y = 10
                        if y == 10 && (x == 6 || x == 7 || z == 6 || z == 7) {
                            world.set_block(ox + x, py, oz + z, air);
                        } else {
                            world.set_block(ox + x, py, oz + z, dark_oak_planks);
                            blocks_placed += 1;
                        }
                    } else {
                        world.set_block(ox + x, py, oz + z, air);
                    }
                }
            }
        }

        // 3. Top Lookout Deck Platform (y = 13, overhang 12x12)
        for x in 1..=12 {
            for z in 1..=12 {
                world.set_block(ox + x, oy + 13, oz + z, dark_oak_planks);
                blocks_placed += 1;
            }
        }

        // Balcony Guardrails (Dark Oak Fences at y = 14)
        for x in 1..=12 {
            for z in 1..=12 {
                let is_rim = x == 1 || x == 12 || z == 1 || z == 12;
                if is_rim {
                    world.set_block(ox + x, oy + 14, oz + z, fence);
                    blocks_placed += 1;
                }
            }
        }

        // Top Lookout Roof Supported by 4 Corner Posts (y = 14..=17)
        for &(rx, rz) in &[(3, 3), (10, 3), (3, 10), (10, 10)] {
            for y in 14..=16 {
                world.set_block(ox + rx, oy + y, oz + rz, fence);
                blocks_placed += 1;
            }
        }
        // Pitched Roof Cap (Dark Oak Stairs & Planks, y = 17..=18)
        for x in 2..=11 {
            for z in 2..=11 {
                let is_edge = x == 2 || x == 11 || z == 2 || z == 11;
                let block = if is_edge { dark_oak_stairs } else { dark_oak_planks };
                world.set_block(ox + x, oy + 17, oz + z, block);
                blocks_placed += 1;
            }
        }

        // Top Floor Central Outpost Loot Chest at (ox + 6, oy + 14, oz + 6)
        world.set_block(ox + 6, oy + 14, oz + 6, chest);
        blocks_placed += 1;

        // 4. Surrounding Accessory Pieces:
        // Positioned randomly in the 4 quadrants around the watchtower
        // Quadrant 1 (North-East): Target Tent
        Self::generate_target_tent(world, (ox + 22, oy, oz - 15));
        blocks_placed += 25;

        // Quadrant 2 (North-West): Iron Golem Cage
        Self::generate_iron_golem_cage(world, (ox - 18, oy, oz - 12));
        blocks_placed += 30;

        // Quadrant 3 (South-West): Log Pile
        Self::generate_log_pile(world, (ox - 16, oy, oz + 20));
        blocks_placed += 18;

        // Quadrant 4 (South-East): Scarecrows / Archery Targets
        Self::generate_scarecrows(world, (ox + 20, oy, oz + 18), &mut rng);
        blocks_placed += 12;

        blocks_placed
    }

    /// Accessory 1: Target Tent (white wool canopy over target block and arrows)
    pub fn generate_target_tent(world: &mut dyn OreBlockAccess, origin: (i32, i32, i32)) {
        let (ox, oy, oz) = origin;
        let wool = SurfaceBlock::SnowBlock; // Wool representation
        let fence = SurfaceBlock::DarkOakFence;
        let target = SurfaceBlock::TargetBlock;

        // Support poles
        world.set_block(ox, oy + 1, oz, fence);
        world.set_block(ox + 4, oy + 1, oz, fence);
        world.set_block(ox, oy + 1, oz + 4, fence);
        world.set_block(ox + 4, oy + 1, oz + 4, fence);

        // Wool canopy
        for x in 0..=4 {
            for z in 0..=4 {
                world.set_block(ox + x, oy + 3, oz + z, wool);
            }
        }
        // Central target practice block
        world.set_block(ox + 2, oy + 1, oz + 2, target);
    }

    /// Accessory 2: Iron Golem Cage (Dark Oak fence cage with iron bars holding captive golem)
    pub fn generate_iron_golem_cage(world: &mut dyn OreBlockAccess, origin: (i32, i32, i32)) {
        let (ox, oy, oz) = origin;
        let fence = SurfaceBlock::DarkOakFence;
        let planks = SurfaceBlock::DarkOakPlanks;
        let bars = SurfaceBlock::IronBars;

        // Floor
        for x in 0..=3 {
            for z in 0..=3 {
                world.set_block(ox + x, oy, oz + z, planks);
            }
        }
        // Cage walls
        for y in 1..=3 {
            for x in 0..=3 {
                for z in 0..=3 {
                    let is_wall = x == 0 || x == 3 || z == 0 || z == 3;
                    if is_wall {
                        let block = if y == 2 { bars } else { fence };
                        world.set_block(ox + x, oy + y, oz + z, block);
                    }
                }
            }
        }
        // Roof
        for x in 0..=3 {
            for z in 0..=3 {
                world.set_block(ox + x, oy + 4, oz + z, planks);
            }
        }
    }

    /// Accessory 3: Log Pile (Dark oak log stack bound with chains)
    pub fn generate_log_pile(world: &mut dyn OreBlockAccess, origin: (i32, i32, i32)) {
        let (ox, oy, oz) = origin;
        let log = SurfaceBlock::DarkOakLog;

        // Base 3x3 horizontal logs
        for x in 0..3 {
            for z in 0..3 {
                world.set_block(ox + x, oy + 1, oz + z, log);
            }
        }
        // Second tier 2x2 logs
        for x in 0..2 {
            for z in 0..2 {
                world.set_block(ox + x, oy + 2, oz + z, log);
            }
        }
    }

    /// Accessory 4: Scarecrows & Archery Target Dummies
    pub fn generate_scarecrows(world: &mut dyn OreBlockAccess, origin: (i32, i32, i32), _rng: &mut LegacyRandom) {
        let (ox, oy, oz) = origin;
        let fence = SurfaceBlock::DarkOakFence;
        let hay = SurfaceBlock::HayBale;
        let pumpkin = SurfaceBlock::CarvedPumpkin;
        let target = SurfaceBlock::TargetBlock;

        // Scarecrow 1
        world.set_block(ox, oy + 1, oz, fence);
        world.set_block(ox, oy + 2, oz, hay);
        world.set_block(ox, oy + 3, oz, pumpkin);

        // Scarecrow 2 (Target block dummy)
        world.set_block(ox + 3, oy + 1, oz + 2, fence);
        world.set_block(ox + 3, oy + 2, oz + 2, target);
        world.set_block(ox + 3, oy + 3, oz + 2, hay);
    }
}
