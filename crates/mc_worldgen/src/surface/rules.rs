//! Canonical surface rule tree builder implementing vanilla Overworld biome strata.
//! REA Reference: net.minecraft.data.worldgen.SurfaceRuleData (pr.class)
//! REA Reference: net.minecraft.world.level.levelgen.SurfaceSystem (dpi.class)

use super::ast::{Condition, SurfaceContext, SurfaceRule};
use crate::biomes::{BiomeId, SurfaceBlock, TerracottaBands};

/// Surface System coordinating AST surface rule evaluation and strata synthesis.
#[derive(Clone, Debug)]
pub struct SurfaceSystem {
    pub seed: u64,
    pub terracotta_bands: TerracottaBands,
    pub master_rule: SurfaceRule,
}

impl Default for SurfaceSystem {
    fn default() -> Self {
        Self::new(192837465)
    }
}

impl SurfaceSystem {
    /// Constructs a SurfaceSystem with seeded terracotta bands and compiled master rule tree.
    pub fn new(seed: u64) -> Self {
        let terracotta_bands = TerracottaBands::from_seed(seed);
        let master_rule = build_overworld_rules(&terracotta_bands);
        Self {
            seed,
            terracotta_bands,
            master_rule,
        }
    }

    /// Evaluates the surface block for coordinate (x, y, z).
    pub fn get_surface_block(
        &self,
        x: i32,
        y: i32,
        z: i32,
        surface_y: i32,
        biome: BiomeId,
        surface_noise: f64,
    ) -> SurfaceBlock {
        let ctx = SurfaceContext::new(x, y, z, surface_y, biome, surface_noise).with_seed(self.seed);

        // 1. Bedrock bottom floor
        if y == -64 {
            return SurfaceBlock::Bedrock;
        }

        // 2. Above the surface terrain: atmosphere or water ocean filling
        if y > surface_y {
            if y <= ctx.water_height && (biome.is_ocean() || biome == BiomeId::River || biome == BiomeId::FrozenRiver) {
                if (biome == BiomeId::FrozenOcean || biome == BiomeId::DeepFrozenOcean || biome == BiomeId::FrozenRiver) && y == ctx.water_height {
                    return SurfaceBlock::Ice;
                }
                return SurfaceBlock::Water;
            }

            if y == surface_y + 1 && biome.is_snowy() && surface_y >= ctx.water_height {
                return SurfaceBlock::SnowLayer;
            }

            return SurfaceBlock::Air;
        }

        // 3. Evaluate compiled AST surface rule tree
        if let Some(block) = self.master_rule.apply(&ctx) {
            return block;
        }

        // 4. Default base rock
        if y < 0 {
            SurfaceBlock::Deepslate
        } else {
            SurfaceBlock::Stone
        }
    }
}

/// Builds the canonical Minecraft Overworld surface rule AST tree.
pub fn build_overworld_rules(terracotta_bands: &TerracottaBands) -> SurfaceRule {
    let mut rules = Vec::new();

    // =========================================================================
    // 1. BEDROCK RULES (VerticalGradient)
    // =========================================================================
    rules.push(SurfaceRule::If {
        condition: Condition::VerticalGradient {
            random_name: "minecraft:bedrock_floor".to_string(),
            true_at_and_below: -64,
            false_at_and_above: -59,
        },
        then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Bedrock)),
    });

    // =========================================================================
    // 2. STEEP CLIFF FACES (Bare Stone & Gravel)
    // =========================================================================
    rules.push(SurfaceRule::If {
        condition: Condition::All(vec![
            Condition::Steep,
            Condition::AbovePreliminarySurface {
                offset: 3,
                add_surface_depth: true,
            },
        ]),
        then_rule: Box::new(SurfaceRule::Sequence(vec![
            SurfaceRule::If {
                condition: Condition::Biome(vec![BiomeId::StonyPeaks]),
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Stone)),
            },
            SurfaceRule::If {
                condition: Condition::Hole,
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Gravel)),
            },
            SurfaceRule::Block(SurfaceBlock::Stone),
        ])),
    });

    // =========================================================================
    // 3. BADLANDS / MESA STRATA (192-layer Terracotta Bands)
    // =========================================================================
    let badlands_biomes = vec![
        BiomeId::Badlands,
        BiomeId::ErodedBadlands,
        BiomeId::WoodedBadlands,
    ];

    let badlands_rule = SurfaceRule::If {
        condition: Condition::Biome(badlands_biomes),
        then_rule: Box::new(SurfaceRule::Sequence(vec![
            // Valleys below sea level: Red Sand over Red Sandstone
            SurfaceRule::If {
                condition: Condition::All(vec![
                    Condition::YCheck { min_y: -64, max_y: 65 },
                    Condition::AbovePreliminarySurface {
                        offset: 0,
                        add_surface_depth: false,
                    },
                ]),
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::RedSand)),
            },
            SurfaceRule::If {
                condition: Condition::All(vec![
                    Condition::YCheck { min_y: -64, max_y: 65 },
                    Condition::AbovePreliminarySurface {
                        offset: 3,
                        add_surface_depth: false,
                    },
                ]),
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::RedSandstone)),
            },
            // High plateau surface: Terracotta
            SurfaceRule::If {
                condition: Condition::AbovePreliminarySurface {
                    offset: 0,
                    add_surface_depth: false,
                },
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Terracotta)),
            },
            // Subsurface strata: multi-colored terracotta banding determined by deterministic Y-level noise
            SurfaceRule::If {
                condition: Condition::YCheck {
                    min_y: 50,
                    max_y: 320,
                },
                then_rule: Box::new(SurfaceRule::BandTerracotta(terracotta_bands.clone())),
            },
        ])),
    };
    rules.push(badlands_rule);

    // =========================================================================
    // 4. MOUNTAIN TOPS (Frozen Peaks, Jagged Peaks, Stony Peaks, Snowy Slopes, Grove)
    // =========================================================================
    let mountain_peaks_rule = SurfaceRule::If {
        condition: Condition::Biome(vec![
            BiomeId::FrozenPeaks,
            BiomeId::JaggedPeaks,
            BiomeId::StonyPeaks,
            BiomeId::SnowySlopes,
            BiomeId::Grove,
        ]),
        then_rule: Box::new(SurfaceRule::Sequence(vec![
            // Stony Peaks: Exposed Stone and Gravel
            SurfaceRule::If {
                condition: Condition::Biome(vec![BiomeId::StonyPeaks]),
                then_rule: Box::new(SurfaceRule::Sequence(vec![
                    SurfaceRule::If {
                        condition: Condition::AbovePreliminarySurface {
                            offset: 2,
                            add_surface_depth: true,
                        },
                        then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Gravel)),
                    },
                    SurfaceRule::Block(SurfaceBlock::Stone),
                ])),
            },
            // Frozen & Jagged Peaks: PowderSnow pockets in holes, SnowBlock over PackedIce over Stone
            SurfaceRule::If {
                condition: Condition::Biome(vec![BiomeId::FrozenPeaks, BiomeId::JaggedPeaks]),
                then_rule: Box::new(SurfaceRule::Sequence(vec![
                    SurfaceRule::If {
                        condition: Condition::All(vec![
                            Condition::Hole,
                            Condition::AbovePreliminarySurface {
                                offset: 1,
                                add_surface_depth: false,
                            },
                        ]),
                        then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::PowderSnow)),
                    },
                    SurfaceRule::If {
                        condition: Condition::AbovePreliminarySurface {
                            offset: 0,
                            add_surface_depth: false,
                        },
                        then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::SnowBlock)),
                    },
                    SurfaceRule::If {
                        condition: Condition::AbovePreliminarySurface {
                            offset: 3,
                            add_surface_depth: true,
                        },
                        then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::PackedIce)),
                    },
                    SurfaceRule::Block(SurfaceBlock::Stone),
                ])),
            },
            // Snowy Slopes & Grove: Deep snow coverage
            SurfaceRule::If {
                condition: Condition::AbovePreliminarySurface {
                    offset: 2,
                    add_surface_depth: true,
                },
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::SnowBlock)),
            },
            SurfaceRule::If {
                condition: Condition::AbovePreliminarySurface {
                    offset: 5,
                    add_surface_depth: true,
                },
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Dirt)),
            },
            SurfaceRule::Block(SurfaceBlock::Stone),
        ])),
    };
    rules.push(mountain_peaks_rule);

    // =========================================================================
    // 5. DESERT STRATA
    // =========================================================================
    let desert_rule = SurfaceRule::If {
        condition: Condition::Biome(vec![BiomeId::Desert]),
        then_rule: Box::new(SurfaceRule::Sequence(vec![
            // Below water table / sea level (Y <= 63): Sandstone shelf
            SurfaceRule::If {
                condition: Condition::WaterCheck { offset: 0 },
                then_rule: Box::new(SurfaceRule::Sequence(vec![
                    SurfaceRule::If {
                        condition: Condition::AbovePreliminarySurface {
                            offset: 4,
                            add_surface_depth: true,
                        },
                        then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Sandstone)),
                    },
                    SurfaceRule::Block(SurfaceBlock::Stone),
                ])),
            },
            // Above sea level: Top 3-4 layers Sand (depth 0..=3)
            SurfaceRule::If {
                condition: Condition::AbovePreliminarySurface {
                    offset: 3,
                    add_surface_depth: true,
                },
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Sand)),
            },
            // Subsurface Sandstone (depth 4..=7)
            SurfaceRule::If {
                condition: Condition::AbovePreliminarySurface {
                    offset: 7,
                    add_surface_depth: true,
                },
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Sandstone)),
            },
        ])),
    };
    rules.push(desert_rule);

    // =========================================================================
    // 6. UNDERWATER / COASTLINE / BEACH STRATA
    // =========================================================================
    let beach_biomes = vec![
        BiomeId::Beach,
        BiomeId::Ocean,
        BiomeId::DeepOcean,
        BiomeId::WarmOcean,
        BiomeId::LukewarmOcean,
        BiomeId::DeepLukewarmOcean,
        BiomeId::ColdOcean,
        BiomeId::DeepColdOcean,
    ];

    let beach_rule = SurfaceRule::If {
        condition: Condition::Biome(beach_biomes),
        then_rule: Box::new(SurfaceRule::Sequence(vec![
            // Gravel hole / depression patches on ocean floor
            SurfaceRule::If {
                condition: Condition::All(vec![
                    Condition::Hole,
                    Condition::WaterCheck { offset: 0 },
                    Condition::AbovePreliminarySurface {
                        offset: 2,
                        add_surface_depth: true,
                    },
                ]),
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Gravel)),
            },
            // Cold ocean floor: Gravel
            SurfaceRule::If {
                condition: Condition::All(vec![
                    Condition::Biome(vec![BiomeId::ColdOcean, BiomeId::DeepColdOcean]),
                    Condition::AbovePreliminarySurface {
                        offset: 2,
                        add_surface_depth: true,
                    },
                ]),
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Gravel)),
            },
            // Sand top layer (depth <= 2)
            SurfaceRule::If {
                condition: Condition::AbovePreliminarySurface {
                    offset: 2,
                    add_surface_depth: true,
                },
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Sand)),
            },
            // Subsurface Sandstone (depth <= 4)
            SurfaceRule::If {
                condition: Condition::AbovePreliminarySurface {
                    offset: 4,
                    add_surface_depth: false,
                },
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Sandstone)),
            },
        ])),
    };
    rules.push(beach_rule);

    // =========================================================================
    // 7. STANDARD OVERWORLD STRATA (Plains, Forests, etc.)
    // =========================================================================
    let standard_rule = SurfaceRule::Sequence(vec![
        // River / ocean bed gravel & sand patches via Hole condition
        SurfaceRule::If {
            condition: Condition::All(vec![
                Condition::Hole,
                Condition::WaterCheck { offset: -1 },
                Condition::AbovePreliminarySurface {
                    offset: 1,
                    add_surface_depth: false,
                },
            ]),
            then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Gravel)),
        },
        // Top block: GrassBlock when above water level (y >= 63), or Dirt when underwater
        SurfaceRule::If {
            condition: Condition::AbovePreliminarySurface {
                offset: 0,
                add_surface_depth: false,
            },
            then_rule: Box::new(SurfaceRule::Sequence(vec![
                SurfaceRule::If {
                    condition: Condition::WaterCheck { offset: -1 },
                    then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Dirt)),
                },
                SurfaceRule::Block(SurfaceBlock::GrassBlock),
            ])),
        },
        // Subsurface: 3-4 layers of Dirt
        SurfaceRule::If {
            condition: Condition::AbovePreliminarySurface {
                offset: 3,
                add_surface_depth: true,
            },
            then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Dirt)),
        },
        // Deep rock foundation: Deepslate below Y=0, Stone above Y=0
        SurfaceRule::If {
            condition: Condition::YCheck {
                min_y: i32::MIN,
                max_y: -1,
            },
            then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Deepslate)),
        },
        SurfaceRule::Block(SurfaceBlock::Stone),
    ]);
    rules.push(standard_rule);

    SurfaceRule::Sequence(rules)
}
