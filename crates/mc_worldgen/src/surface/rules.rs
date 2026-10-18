//! Canonical surface rule tree builder implementing vanilla Overworld biome strata.
//! REA Reference: net.minecraft.data.worldgen.SurfaceRuleData (pr.class)
//! REA Reference: net.minecraft.world.level.levelgen.SurfaceSystem (dpi.class)

use super::ast::{Condition, SurfaceContext, SurfaceRule};
use crate::biomes::{BiomeId, SurfaceBlock, TerracottaBands};

/// Surface System coordinating AST surface rule evaluation and strata synthesis.
#[derive(Clone, Debug)]
pub struct SurfaceSystem {
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
        let ctx = SurfaceContext::new(x, y, z, surface_y, biome, surface_noise);

        // 1. Bedrock bottom floor
        if y <= -64 {
            return SurfaceBlock::Bedrock;
        }
        if y < -60 {
            let threshold = -60 - y;
            let hash = (x.wrapping_mul(3129871) ^ z.wrapping_mul(116129781) ^ y.wrapping_mul(9182371)).abs() % 4;
            if hash < threshold {
                return SurfaceBlock::Bedrock;
            }
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
    // 1. BEDROCK RULES
    // =========================================================================
    rules.push(SurfaceRule::If {
        condition: Condition::YCheck {
            min_y: i32::MIN,
            max_y: -64,
        },
        then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Bedrock)),
    });

    // =========================================================================
    // 2. BADLANDS / MESA STRATA
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
                condition: Condition::AbovePreliminarySurface {
                    offset: 20,
                    add_surface_depth: true,
                },
                then_rule: Box::new(SurfaceRule::BandTerracotta(terracotta_bands.clone())),
            },
        ])),
    };
    rules.push(badlands_rule);

    // =========================================================================
    // 3. DESERT STRATA
    // =========================================================================
    let desert_rule = SurfaceRule::If {
        condition: Condition::Biome(vec![BiomeId::Desert]),
        then_rule: Box::new(SurfaceRule::Sequence(vec![
            // Top 3-4 layers Sand
            SurfaceRule::If {
                condition: Condition::AbovePreliminarySurface {
                    offset: 3,
                    add_surface_depth: true,
                },
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Sand)),
            },
            // Subsurface Sandstone (up to 8 layers)
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
    // 4. UNDERWATER / BEACH STRATA
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
            // Sand top layer (depth <= 3)
            SurfaceRule::If {
                condition: Condition::AbovePreliminarySurface {
                    offset: 3,
                    add_surface_depth: true,
                },
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Sand)),
            },
            // Subsurface Sandstone (depth <= 5)
            SurfaceRule::If {
                condition: Condition::AbovePreliminarySurface {
                    offset: 5,
                    add_surface_depth: false,
                },
                then_rule: Box::new(SurfaceRule::Block(SurfaceBlock::Sandstone)),
            },
        ])),
    };
    rules.push(beach_rule);

    // =========================================================================
    // 5. MOUNTAIN PEAKS (FROZEN PEAKS, JAGGED PEAKS, STONY PEAKS)
    // =========================================================================
    let mountain_peaks_rule = SurfaceRule::If {
        condition: Condition::Biome(vec![
            BiomeId::FrozenPeaks,
            BiomeId::JaggedPeaks,
            BiomeId::StonyPeaks,
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
            // Frozen & Jagged Peaks: SnowBlock surface over PackedIce over Stone
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
    };
    rules.push(mountain_peaks_rule);

    // =========================================================================
    // 6. STANDARD OVERWORLD STRATA (Plains, Forests, etc.)
    // =========================================================================
    let standard_rule = SurfaceRule::Sequence(vec![
        // Top block: GrassBlock when above water level (y >= 63), or Dirt/Sand when underwater
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
