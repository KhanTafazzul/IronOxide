//! Bastion Remnant procedural Jigsaw assembly implementing housing, bridge, stables, and treasure room.
//! REA Reference: data/minecraft/worldgen/template_pool/bastion/starts.json
//! REA Reference: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::LegacyRandom;

/// Bastion Remnant variant matching the 4 start pool entries in bastion/starts.json.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BastionType {
    HousingUnits,
    HoglinStables,
    TreasureRoom,
    Bridge,
}

impl BastionType {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::HousingUnits => "minecraft:bastion/units/air_base",
            Self::HoglinStables => "minecraft:bastion/hoglin_stable/air_base",
            Self::TreasureRoom => "minecraft:bastion/treasure/big_air_full",
            Self::Bridge => "minecraft:bastion/bridge/starting_pieces/entrance_base",
        }
    }
}

/// Bastion Remnant generator.
pub struct BastionRemnantGenerator;

impl BastionRemnantGenerator {
    /// Selects a Bastion Remnant variant from chunk seed with equal 25% weight.
    pub fn pick_variant(rng: &mut LegacyRandom) -> BastionType {
        match rng.next_int_bounded(4) {
            0 => BastionType::HousingUnits,
            1 => BastionType::HoglinStables,
            2 => BastionType::TreasureRoom,
            _ => BastionType::Bridge,
        }
    }

    /// Generates a Bastion Remnant complex at the given origin.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `origin`: Structure center (X, Y, Z). Usually $Y \in [33, 50]$.
    /// * `seed`: Generation seed.
    ///
    /// Returns `(variant, blocks_placed)`.
    pub fn generate(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        seed: i64,
    ) -> (BastionType, usize) {
        let mut rng = LegacyRandom::new(seed);
        let variant = Self::pick_variant(&mut rng);
        let placed = Self::generate_variant(world, origin, variant);
        (variant, placed)
    }

    /// Generates a specific Bastion Remnant variant at the given origin.
    pub fn generate_variant(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        variant: BastionType,
    ) -> usize {
        let (ox, oy, oz) = origin;

        let blackstone = SurfaceBlock::Blackstone;
        let polished = SurfaceBlock::PolishedBlackstone;
        let bricks = SurfaceBlock::PolishedBlackstoneBricks;
        let gold_block = SurfaceBlock::GoldBlock;
        let spawner = SurfaceBlock::Spawner;
        let chest = SurfaceBlock::Chest;

        let mut blocks_placed = 0;

        match variant {
            BastionType::HousingUnits => {
                // Multi-story residential unit complex (14x12x14)
                for dx in -7..=7 {
                    for dz in -7..=7 {
                        // Foundation
                        world.set_block(ox + dx, oy, oz + dz, bricks);
                        blocks_placed += 1;

                        // Hollow interior
                        for dy in 1..=10 {
                            let is_wall = dx == -7 || dx == 7 || dz == -7 || dz == 7;
                            let px = ox + dx;
                            let py = oy + dy;
                            let pz = oz + dz;

                            if is_wall {
                                // Windows at mid-height
                                if dy == 3 || dy == 7 {
                                    if dx.abs() <= 2 || dz.abs() <= 2 {
                                        world.set_block(px, py, pz, SurfaceBlock::Air);
                                    } else {
                                        world.set_block(px, py, pz, bricks);
                                        blocks_placed += 1;
                                    }
                                } else {
                                    world.set_block(px, py, pz, bricks);
                                    blocks_placed += 1;
                                }
                            } else {
                                // Intermediate floor at dy = 5
                                if dy == 5 {
                                    world.set_block(px, py, pz, polished);
                                    blocks_placed += 1;
                                } else {
                                    world.set_block(px, py, pz, SurfaceBlock::Air);
                                }
                            }
                        }
                    }
                }

                // Decorative gold blocks embedded in housing walls
                world.set_block(ox - 6, oy + 1, oz - 6, gold_block);
                world.set_block(ox + 6, oy + 6, oz + 6, gold_block);
                world.set_block(ox, oy + 1, oz, chest);
                blocks_placed += 3;
            }

            BastionType::HoglinStables => {
                // Sloped feeding ramps & stable pens (16x8x12)
                for dx in -8..=8 {
                    for dz in -6..=6 {
                        // Stable floor
                        world.set_block(ox + dx, oy, oz + dz, blackstone);
                        blocks_placed += 1;

                        // Air clearance
                        for dy in 1..=6 {
                            world.set_block(ox + dx, oy + dy, oz + dz, SurfaceBlock::Air);
                        }
                    }
                }

                // Partition walls dividing the pens
                for pz in [-2, 2] {
                    for dx in -7..=7 {
                        world.set_block(ox + dx, oy + 1, oz + pz, bricks);
                        world.set_block(ox + dx, oy + 2, oz + pz, bricks);
                        blocks_placed += 2;
                    }
                }

                // Feeding troughs with Soul Sand
                for dx in -3..=3 {
                    world.set_block(ox + dx, oy, oz, SurfaceBlock::SoulSand);
                    blocks_placed += 1;
                }
            }

            BastionType::TreasureRoom => {
                // Central Fortified Vault with Lava Moat & Gold Core (16x14x16)
                let vault_radius = 4;
                let moat_radius = 7;

                for dx in -moat_radius..=moat_radius {
                    for dz in -moat_radius..=moat_radius {
                        let dist_sq = dx * dx + dz * dz;

                        // Moat floor
                        world.set_block(ox + dx, oy, oz + dz, blackstone);
                        blocks_placed += 1;

                        if dist_sq > vault_radius * vault_radius && dist_sq <= moat_radius * moat_radius {
                            // Lava moat
                            world.set_block(ox + dx, oy + 1, oz + dz, SurfaceBlock::Lava);
                            blocks_placed += 1;
                        }

                        // Air space above moat
                        for dy in 2..=10 {
                            world.set_block(ox + dx, oy + dy, oz + dz, SurfaceBlock::Air);
                        }
                    }
                }

                // Central elevated vault chamber
                for dx in -vault_radius..=vault_radius {
                    for dz in -vault_radius..=vault_radius {
                        world.set_block(ox + dx, oy + 1, oz + dz, bricks);
                        blocks_placed += 1;
                    }
                }

                // Central Gold Block cube (2x2x2 solid gold treasury!)
                for dx in 0..2 {
                    for dy in 0..2 {
                        for dz in 0..2 {
                            world.set_block(ox + dx, oy + 2 + dy, oz + dz, gold_block);
                            blocks_placed += 1;
                        }
                    }
                }

                // Elevated Magma Cube / Piglin Spawner above gold
                world.set_block(ox, oy + 5, oz, spawner);
                // Treasure chests around the gold
                world.set_block(ox - 2, oy + 2, oz, chest);
                world.set_block(ox + 3, oy + 2, oz, chest);
                blocks_placed += 3;
            }

            BastionType::Bridge => {
                // Massive fortified entrance rampart bridge (8x10x24)
                let bridge_len = 20;
                for step in -bridge_len / 2..=bridge_len / 2 {
                    let cz = oz + step;
                    for dx in -4..=4 {
                        let px = ox + dx;

                        // Thick bridge roadway
                        world.set_block(px, oy, cz, bricks);
                        blocks_placed += 1;

                        // Hollow roadway
                        for dy in 1..=4 {
                            world.set_block(px, oy + dy, cz, SurfaceBlock::Air);
                        }

                        // Defensive ramparts on sides
                        if dx == -4 || dx == 4 {
                            world.set_block(px, oy + 1, cz, polished);
                            if step % 2 == 0 {
                                world.set_block(px, oy + 2, cz, bricks);
                            }
                            blocks_placed += 2;
                        }
                    }
                }

                // Gilded gold blocks along bridge center
                world.set_block(ox, oy, oz - 5, gold_block);
                world.set_block(ox, oy, oz + 5, gold_block);
                blocks_placed += 2;
            }
        }

        blocks_placed
    }
}
