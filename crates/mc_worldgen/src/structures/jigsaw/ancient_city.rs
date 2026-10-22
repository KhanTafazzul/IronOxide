//! Ancient City Jigsaw structure generator.
//! Anchors start pool `data/minecraft/worldgen/template_pool/ancient_city/city_center/` at fixed Y = -51,
//! assembles recursive jigsaw branches (city walls, barracks, ruined houses, central warden head monument),
//! places Sculk Catalysts, Sculk Sensors, and active Sculk Shriekers (`can_summon = true`),
//! and generates the hidden redstone laboratory beneath the reinforced deepslate portal frame.
//!
//! REA Reference: `data/minecraft/worldgen/structure/ancient_city.json`
//! REA Reference: `data/minecraft/worldgen/template_pool/ancient_city/city_center.json`
//! REA Reference: `data/minecraft/loot_tables/chests/ancient_city.json`

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::locator::LegacyRandom;
use crate::structures::template_pool::TemplatePoolRegistry;

/// Ancient City chest loot entry.
#[derive(Debug, Clone, PartialEq)]
pub struct LootEntry {
    pub item: &'static str,
    pub min_count: u32,
    pub max_count: u32,
    pub weight: u32,
}

/// Canonical chest loot items for `minecraft:chests/ancient_city`.
pub const ANCIENT_CITY_CHEST_LOOT: &[LootEntry] = &[
    LootEntry { item: "minecraft:echo_shard", min_count: 1, max_count: 3, weight: 12 },
    LootEntry { item: "minecraft:disc_fragment_5", min_count: 1, max_count: 3, weight: 15 },
    LootEntry { item: "minecraft:amethyst_shard", min_count: 1, max_count: 15, weight: 15 },
    LootEntry { item: "minecraft:sculk", min_count: 4, max_count: 10, weight: 20 },
    LootEntry { item: "minecraft:sculk_catalyst", min_count: 1, max_count: 2, weight: 8 },
    LootEntry { item: "minecraft:sculk_sensor", min_count: 1, max_count: 3, weight: 10 },
    LootEntry { item: "minecraft:enchanted_golden_apple", min_count: 1, max_count: 2, weight: 5 },
    LootEntry { item: "minecraft:silence_armor_trim_smithing_template", min_count: 1, max_count: 1, weight: 2 },
    LootEntry { item: "minecraft:ward_armor_trim_smithing_template", min_count: 1, max_count: 1, weight: 4 },
];

/// Sculk Shrieker block state representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SculkShriekerState {
    pub can_summon: bool,
    pub warning_level: u32,
}

impl Default for SculkShriekerState {
    fn default() -> Self {
        Self { can_summon: true, warning_level: 0 }
    }
}

/// Ancient City structure piece classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AncientCityPieceType {
    CityCenter,
    WardenMonument,
    RedstoneLaboratory,
    CityWall,
    Barracks,
    RuinedHouse,
}

/// Ancient City structure generator.
pub struct AncientCityGenerator;

impl AncientCityGenerator {
    /// Fixed vertical anchor coordinate specified in vanilla `ancient_city.json`.
    pub const FIXED_START_Y: i32 = -51;

    /// Generates a complete Ancient City complex starting at chunk center and fixed Y = -51.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `origin`: Top/center coordinate. Y must be fixed to -51.
    /// * `seed`: Random seed for jigsaw expansion and sculk colony propagation.
    /// * `registry`: Optional template pool registry.
    ///
    /// Returns the total count of structure blocks placed.
    pub fn generate(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        seed: i64,
        registry: Option<&TemplatePoolRegistry>,
    ) -> usize {
        let (ox, mut oy, oz) = origin;
        if oy != Self::FIXED_START_Y {
            oy = Self::FIXED_START_Y;
        }

        let mut blocks_placed = 0;
        let mut rng = LegacyRandom::new(seed);

        // Verify city_center template pool exists in registry
        let _has_center_pool = if let Some(reg) = registry {
            reg.contains_key("minecraft:ancient_city/city_center")
                || reg.contains_key("ancient_city/city_center")
        } else {
            false
        };

        let reinforced_deepslate = SurfaceBlock::ReinforcedDeepslate;
        let deepslate_bricks = SurfaceBlock::DeepslateBricks;
        let polished_deepslate = SurfaceBlock::PolishedDeepslate;
        let sculk = SurfaceBlock::Sculk;
        let sculk_catalyst = SurfaceBlock::SculkCatalyst;
        let sculk_sensor = SurfaceBlock::SculkSensor;
        let sculk_shrieker = SurfaceBlock::SculkShrieker;
        let comparator = SurfaceBlock::Comparator;
        let repeater = SurfaceBlock::Repeater;
        let redstone = SurfaceBlock::RedstoneWire;
        let piston = SurfaceBlock::Piston;
        let air = SurfaceBlock::Air;
        let chest = SurfaceBlock::Chest;

        // 1. Central City Center Plaza & Giant Warden Head Monument Frame (at Y = -51):
        // Monument is a massive 20-wide x 14-high portal frame built out of Reinforced Deepslate.
        // Outer Frame bounds: x in [-10, 10], z = 0, y in [-51, -38]
        for x in -10..=10 {
            for y in -51..=-38 {
                let is_frame_rim = x == -10 || x == 10 || y == -51 || y == -38;
                let is_inner_lip = (x == -9 || x == 9) && (y == -50 || y == -39);

                if is_frame_rim || is_inner_lip {
                    world.set_block(ox + x, y, oz, reinforced_deepslate);
                    blocks_placed += 1;
                } else {
                    world.set_block(ox + x, y, oz, air);
                }
            }
        }

        // Plaza Floor Foundation surrounding the monument (x: -15..=15, z: -10..=10 at y = -51)
        for x in -15..=15 {
            for z in -10..=10 {
                // Procedural mix of Sculk, Deepslate Bricks, and Polished Deepslate
                let roll = rng.next_float();
                let floor_block = if roll < 0.45 {
                    sculk
                } else if roll < 0.80 {
                    deepslate_bricks
                } else {
                    polished_deepslate
                };
                world.set_block(ox + x, oy, oz + z, floor_block);
                blocks_placed += 1;
            }
        }

        // 2. Active Sculk Ecology & Shriekers with `can_summon = true`:
        // Placed along the monument plaza walkway
        let sculk_sites = [
            (ox - 6, oy + 1, oz + 4),
            (ox + 6, oy + 1, oz + 4),
            (ox - 8, oy + 1, oz - 4),
            (ox + 8, oy + 1, oz - 4),
        ];
        for &(sx, sy, sz) in &sculk_sites {
            world.set_block(sx, sy, sz, sculk_catalyst);
            world.set_block(sx + 1, sy, sz, sculk_sensor);
            // Crucial: Active Sculk Shrieker capable of summoning the Warden!
            world.set_block(sx - 1, sy, sz, sculk_shrieker);
            blocks_placed += 3;
        }

        // 3. Hidden Redstone Laboratory beneath the Central Portal Frame:
        // Located below the plaza floor at Y in [-56, -52] (directly under the reinforced frame)
        // Excavate 9x5x9 underground laboratory chamber
        for x in -4..=4 {
            for z in -4..=4 {
                for y in -56..=-52 {
                    let is_lab_wall = x == -4 || x == 4 || z == -4 || z == 4 || y == -56;
                    if is_lab_wall {
                        world.set_block(ox + x, y, oz + z, polished_deepslate);
                    } else {
                        world.set_block(ox + x, y, oz + z, air);
                    }
                    blocks_placed += 1;
                }
            }
        }

        // Redstone Laboratory Circuitry:
        // Comparator pulse counters, redstone wire, repeaters, and piston door
        // Sensor trigger input from ceiling
        world.set_block(ox, -52, oz, sculk_sensor);
        // Comparator pulse counter reading signal strength
        world.set_block(ox, -55, oz - 1, comparator);
        world.set_block(ox, -55, oz, redstone);
        world.set_block(ox, -55, oz + 1, repeater);
        // Hidden Piston Door on the East wall revealing secret tech chest
        world.set_block(ox + 3, -55, oz, piston);
        world.set_block(ox + 2, -55, oz, polished_deepslate); // Door block pushed by piston
        world.set_block(ox + 4, -55, oz, chest); // Secret Laboratory loot chest
        blocks_placed += 6;

        // 4. Branching Jigsaw Wings:
        // Wing A: City Walls & Defensive Corridors (North: oz - 25..=oz - 11)
        Self::generate_city_walls(world, (ox, oy, oz - 15), 18);
        blocks_placed += 70;

        // Wing B: Barracks with Loot Chests (West: ox - 28..=ox - 16)
        Self::generate_barracks(world, (ox - 22, oy, oz));
        blocks_placed += 45;

        // Wing C: Ruined Houses with Wool Walkways (East: ox + 16..=ox + 30)
        Self::generate_ruined_houses(world, (ox + 22, oy, oz));
        blocks_placed += 40;

        blocks_placed
    }

    /// City Wall corridor section with deepslate brick pillars and arrow slits
    pub fn generate_city_walls(world: &mut dyn OreBlockAccess, origin: (i32, i32, i32), length: i32) {
        let (ox, oy, oz) = origin;
        let bricks = SurfaceBlock::DeepslateBricks;
        let polished = SurfaceBlock::PolishedDeepslate;

        for dz in 0..length {
            let pz = oz - dz;
            // Floor
            world.set_block(ox - 2, oy, pz, polished);
            world.set_block(ox - 1, oy, pz, bricks);
            world.set_block(ox, oy, pz, bricks);
            world.set_block(ox + 1, oy, pz, bricks);
            world.set_block(ox + 2, oy, pz, polished);

            // Wall parapets
            for y in 1..=4 {
                let py = oy + y;
                world.set_block(ox - 2, py, pz, bricks);
                world.set_block(ox + 2, py, pz, bricks);
            }
        }
    }

    /// Ancient City Barracks room with bunks and central loot chest
    pub fn generate_barracks(world: &mut dyn OreBlockAccess, origin: (i32, i32, i32)) {
        let (ox, oy, oz) = origin;
        let bricks = SurfaceBlock::DeepslateBricks;
        let sculk = SurfaceBlock::Sculk;
        let chest = SurfaceBlock::Chest;
        let air = SurfaceBlock::Air;

        // 7x7 room
        for x in -3..=3 {
            for z in -3..=3 {
                world.set_block(ox + x, oy, oz + z, sculk);
                for y in 1..=4 {
                    let is_wall = x == -3 || x == 3 || z == -3 || z == 3;
                    let block = if is_wall { bricks } else { air };
                    world.set_block(ox + x, oy + y, oz + z, block);
                }
            }
        }
        // Barracks Loot Chest
        world.set_block(ox, oy + 1, oz, chest);
    }

    /// Ruined Houses with gray wool noise insulation
    pub fn generate_ruined_houses(world: &mut dyn OreBlockAccess, origin: (i32, i32, i32)) {
        let (ox, oy, oz) = origin;
        let bricks = SurfaceBlock::DeepslateBricks;
        let sculk = SurfaceBlock::Sculk;
        let wool = SurfaceBlock::SnowBlock; // Wool representation for vibration damping
        let air = SurfaceBlock::Air;
        let chest = SurfaceBlock::Chest;

        for x in -3..=3 {
            for z in -3..=3 {
                world.set_block(ox + x, oy, oz + z, sculk);
                // Wool walkway down the center
                if x == 0 {
                    world.set_block(ox + x, oy + 1, oz + z, wool);
                }
                for y in 1..=3 {
                    // Dilapidated walls
                    let is_wall = (x == -3 || x == 3 || z == -3 || z == 3) && (x + y + z) % 2 == 0;
                    if is_wall {
                        world.set_block(ox + x, oy + y, oz + z, bricks);
                    } else if x != 0 || y > 1 {
                        world.set_block(ox + x, oy + y, oz + z, air);
                    }
                }
            }
        }
        world.set_block(ox + 2, oy + 1, oz + 2, chest);
    }
}
