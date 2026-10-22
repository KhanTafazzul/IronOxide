//! Deterministic surface rule engine applying vanilla Minecraft 1.20+ layer stratification.
//! REA Reference: net.minecraft.world.level.levelgen.SurfaceRules (dph.class)
//! REA Reference: net.minecraft.data.worldgen.SurfaceRuleData (pr.class)
//! REA Reference: net.minecraft.world.level.levelgen.SurfaceSystem (dpi.class)

use super::registry::BiomeId;
use crate::seed_pipeline::random::{PositionalRandomFactory, Xoroshiro128PlusPlus};

/// Surface block representation for world generation surface layer placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceBlock {
    Air,
    Stone,
    Deepslate,
    Bedrock,
    Dirt,
    GrassBlock,
    Sand,
    Sandstone,
    RedSand,
    RedSandstone,
    Gravel,
    Water,
    Lava,
    SnowBlock,
    SnowLayer,
    PowderSnow,
    Ice,
    PackedIce,
    Podzol,
    CoarseDirt,
    Mud,

    // Terracotta variants for Badlands stratification
    Terracotta,
    WhiteTerracotta,
    OrangeTerracotta,
    YellowTerracotta,
    BrownTerracotta,
    RedTerracotta,
    LightGrayTerracotta,

    // Overworld Ore variants
    CoalOre,
    DeepslateCoalOre,
    IronOre,
    DeepslateIronOre,
    CopperOre,
    DeepslateCopperOre,
    GoldOre,
    DeepslateGoldOre,
    RedstoneOre,
    DeepslateRedstoneOre,
    DiamondOre,
    DeepslateDiamondOre,
    EmeraldOre,
    DeepslateEmeraldOre,
    LapisOre,
    DeepslateLapisOre,

    // Tree Logs
    OakLog,
    BirchLog,
    SpruceLog,
    DarkOakLog,
    JungleLog,
    AcaciaLog,

    // Tree Leaves
    OakLeaves,
    BirchLeaves,
    SpruceLeaves,
    DarkOakLeaves,
    JungleLeaves,
    AcaciaLeaves,

    // Pale Garden & Pale Oak wood set (1.21.4)
    PaleOakLog,
    PaleOakWood,
    PaleOakPlanks,
    PaleOakLeaves,
    PaleMossBlock,
    PaleMossCarpet,
    PaleHangingMoss,
    CreakingHeart,

    // Ground Cover & Flowers
    Poppy,
    Dandelion,
    ShortGrass,
    TallGrass,
    Fern,
    Sunflower,

    // Water & Aquatic Plants
    LilyPad,
    Seagrass,
    Kelp,

    // Structures, Nether & The End
    Cobblestone,
    MossyCobblestone,
    CobblestoneStairs,
    CutSandstone,
    ChiseledSandstone,
    SandstoneStairs,
    OakPlanks,
    SprucePlanks,
    DarkOakPlanks,
    DarkOakStairs,
    OakFence,
    DarkOakFence,
    Cobweb,
    Rail,
    Chest,
    TNT,
    StonePressurePlate,
    SuspiciousSand,
    DecoratedPot,
    BlueTerracotta,
    Lever,
    StickyPiston,
    Piston,
    TripwireHook,
    StringBlock,
    Dispenser,
    RedstoneWire,
    Repeater,
    CraftingTable,
    Cauldron,
    PottedMushroom,
    ReinforcedDeepslate,
    DeepslateBricks,
    PolishedDeepslate,
    Sculk,
    SculkCatalyst,
    SculkSensor,
    SculkShrieker,
    Comparator,
    RedstoneBlock,
    IronBars,
    HayBale,
    TargetBlock,
    CarvedPumpkin,
    Netherrack,
    NetherBricks,
    NetherWart,
    SoulSand,
    Glowstone,
    NetherQuartzOre,
    NetherGoldOre,
    Fire,
    Spawner,
    Blackstone,
    PolishedBlackstone,
    PolishedBlackstoneBricks,
    GoldBlock,
    Prismarine,
    PrismarineBricks,
    DarkPrismarine,
    SeaLantern,
    Sponge,
    Obsidian,
    EndStone,
    EndPortal,
    EndPortalFrame,
}

/// Standard sea level in Minecraft 1.20+.
pub const SEA_LEVEL: i32 = 63;
/// Minimum Y coordinate in Minecraft 1.20+.
pub const MIN_Y: i32 = -64;
/// Maximum Y coordinate in Minecraft 1.20+.
pub const MAX_Y: i32 = 320;

/// 192-layer Badlands terracotta color bands.
/// REA Reference: net.minecraft.world.level.levelgen.SurfaceSystem.generateBands (dpi.a(auv))
#[derive(Clone, Debug)]
pub struct TerracottaBands {
    pub bands: [SurfaceBlock; 192],
}

impl Default for TerracottaBands {
    fn default() -> Self {
        Self::from_seed(192837465)
    }
}

impl TerracottaBands {
    /// Generates the canonical 192-layer terracotta band pattern matching vanilla Minecraft bytecode (dpi.a).
    pub fn from_seed(seed: u64) -> Self {
        let factory = PositionalRandomFactory::from_world_seed(seed as i64);
        let mut rng = factory.from_hash_of("minecraft:clay_bands");

        let mut bands = [SurfaceBlock::Terracotta; 192];

        // 1. Fill Orange Terracotta intervals (dpi.class lines 14-51)
        let mut y = 0usize;
        while y < bands.len() {
            y += (rng.next_int_bounded(5) + 1) as usize;
            if y < bands.len() {
                bands[y] = SurfaceBlock::OrangeTerracotta;
            }
            y += 1;
        }

        // Helper equivalent to dpi.a(auv, djh[], int, djh)
        fn make_bands(
            rng: &mut Xoroshiro128PlusPlus,
            bands: &mut [SurfaceBlock; 192],
            min_thickness: u32,
            block: SurfaceBlock,
        ) {
            // In vanilla, auv.a(6, 15) is nextInt(15 - 6 + 1) + 6 = nextInt(10) + 6
            let count = rng.next_int_bounded(10) + 6;
            for _ in 0..count {
                let thickness = min_thickness + (rng.next_int_bounded(3) as u32);
                let start_y = rng.next_int_bounded(bands.len() as i32) as usize;
                let mut dy = 0usize;
                while start_y + dy < bands.len() && (dy as u32) < thickness {
                    bands[start_y + dy] = block;
                    dy += 1;
                }
            }
        }

        // 2. Vanilla bands: Yellow, Brown, Red (dpi.class lines 52-78)
        make_bands(&mut rng, &mut bands, 1, SurfaceBlock::YellowTerracotta);
        make_bands(&mut rng, &mut bands, 2, SurfaceBlock::BrownTerracotta);
        make_bands(&mut rng, &mut bands, 1, SurfaceBlock::RedTerracotta);

        // 3. White & Light Gray bands (dpi.class lines 79-186)
        // In vanilla, auv.a(9, 15) is nextInt(15 - 9 + 1) + 9 = nextInt(7) + 9
        let white_count = rng.next_int_bounded(7) + 9;
        let mut cur_y = 0usize;
        for _ in 0..white_count {
            if cur_y >= bands.len() {
                break;
            }
            bands[cur_y] = SurfaceBlock::WhiteTerracotta;
            if cur_y > 1 && (rng.next_long() & 1) != 0 {
                bands[cur_y - 1] = SurfaceBlock::LightGrayTerracotta;
            }
            if cur_y + 1 < bands.len() && (rng.next_long() & 1) != 0 {
                bands[cur_y + 1] = SurfaceBlock::LightGrayTerracotta;
            }
            cur_y += (rng.next_int_bounded(16) + 4) as usize;
        }

        Self { bands }
    }

    /// Evaluates the banded terracotta block at a given coordinate with offset noise.
    /// REA Reference: SurfaceSystem.getBand (dpi.a(int, int, int))
    #[inline]
    pub fn get_band(&self, y: i32, offset_noise: f64) -> SurfaceBlock {
        let offset = (offset_noise * 4.0).round() as i32;
        let idx = (y + offset).rem_euclid(self.bands.len() as i32) as usize;
        self.bands[idx]
    }
}

/// Context information provided during surface rule evaluation for a specific voxel.
/// REA Reference: net.minecraft.world.level.levelgen.SurfaceRules$Context (dph$e.class)
#[derive(Clone, Debug)]
pub struct SurfaceContext {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// Preliminary surface elevation (highest solid block / terrain height).
    pub surface_y: i32,
    /// Distance from preliminary air-contact surface down to current Y coordinate (surface_y - y).
    pub stone_depth_above: i32,
    /// Distance from bottom floor (-64) up to current Y coordinate (y - (-64)).
    pub stone_depth_below: i32,
    /// Water surface level (standard sea level = 63).
    pub water_height: i32,
    /// Biome assigned to the voxel column.
    pub biome: BiomeId,
    /// High-frequency surface noise for terracotta banding or gravel/sand variation.
    pub surface_noise: f64,
    /// Whether the terrain slope is steep (cliff faces with slope > 45 degrees).
    pub is_steep: bool,
    /// Surface hole or underwater depression pocket.
    pub is_hole: bool,
    /// World random seed for deterministic noise / gradients.
    pub seed: u64,
}

impl SurfaceContext {
    pub fn new(
        x: i32,
        y: i32,
        z: i32,
        surface_y: i32,
        biome: BiomeId,
        surface_noise: f64,
    ) -> Self {
        let stone_depth_above = surface_y - y;
        let stone_depth_below = y - (-64);
        let water_height = 63;
        let is_steep = false;
        let is_hole = false;
        let seed = 192837465;

        Self {
            x,
            y,
            z,
            surface_y,
            stone_depth_above,
            stone_depth_below,
            water_height,
            biome,
            surface_noise,
            is_steep,
            is_hole,
            seed,
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_steep(mut self, is_steep: bool) -> Self {
        self.is_steep = is_steep;
        self
    }

    pub fn with_hole(mut self, is_hole: bool) -> Self {
        self.is_hole = is_hole;
        self
    }
}

/// AST Condition evaluating boolean predicates on the surface context.
/// REA Reference: net.minecraft.world.level.levelgen.SurfaceRules$ConditionSource (dph$f.class)
#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
    /// Checks distance from the air-contact surface.
    /// Matches `SurfaceRules.abovePreliminarySurface()` or `SurfaceRules.stoneDepthCheck()`.
    AbovePreliminarySurface {
        /// Distance from the surface boundary (e.g. 0 for top face, 4 for soil depth).
        offset: i32,
        /// Whether surface noise modulates depth.
        add_surface_depth: bool,
    },

    /// Checks if block is under or at sea level / water table.
    /// Matches `SurfaceRules.waterBlockCheck()`.
    WaterCheck {
        /// Offset relative to sea level (0 = at or below sea level 63).
        offset: i32,
    },

    /// Applies biome-specific strata.
    /// Matches `SurfaceRules.isBiome()`.
    Biome(Vec<BiomeId>),

    /// Elevation boundary check (Y between min_y and max_y).
    /// Matches `SurfaceRules.yBlockCheck()`.
    YCheck {
        min_y: i32,
        max_y: i32,
    },

    /// Linear probability gradient between true_at_and_below and false_at_and_above.
    /// Matches `SurfaceRules.verticalGradient("minecraft:bedrock_floor", bottom(), aboveBottom(5))`.
    VerticalGradient {
        random_name: String,
        true_at_and_below: i32,
        false_at_and_above: i32,
    },

    /// Steep cliff faces (slope > 45 degrees). Replaces surface grass/dirt with bare stone or gravel.
    /// Matches `SurfaceRules.steep()`.
    Steep,

    /// Surface hole or underwater depression pocket. Generates gravel/sand patches on ocean and river beds.
    /// Matches `SurfaceRules.hole()`.
    Hole,

    /// Temperature check (e.g. for mountain freezing).
    TemperatureCheck {
        max_temp: f64,
    },

    /// Logical NOT inverting child condition.
    Not(Box<Condition>),

    /// Logical AND combining multiple conditions.
    All(Vec<Condition>),

    /// Logical OR combining multiple conditions.
    Any(Vec<Condition>),
}

impl Condition {
    /// Evaluates this condition against the surface context.
    pub fn test(&self, ctx: &SurfaceContext) -> bool {
        match self {
            Condition::AbovePreliminarySurface {
                offset,
                add_surface_depth,
            } => {
                let surface_delta = if *add_surface_depth {
                    (ctx.surface_noise * 2.0).round() as i32
                } else {
                    0
                };
                let threshold = offset + surface_delta;
                ctx.stone_depth_above <= threshold
            }

            Condition::WaterCheck { offset } => {
                ctx.y <= ctx.water_height + offset
            }

            Condition::Biome(biomes) => {
                biomes.contains(&ctx.biome)
            }

            Condition::YCheck { min_y, max_y } => {
                ctx.y >= *min_y && ctx.y <= *max_y
            }

            Condition::VerticalGradient {
                random_name,
                true_at_and_below,
                false_at_and_above,
            } => {
                if ctx.y <= *true_at_and_below {
                    return true;
                }
                if ctx.y >= *false_at_and_above {
                    return false;
                }
                let progress = (*false_at_and_above - ctx.y) as f64
                    / (*false_at_and_above - *true_at_and_below) as f64;
                let factory = PositionalRandomFactory::from_world_seed(ctx.seed as i64);
                let named_factory = factory.from_hash_of(random_name).fork_positional();
                let mut rng = named_factory.at(ctx.x, ctx.y, ctx.z);
                (rng.next_float() as f64) < progress
            }

            Condition::Steep => ctx.is_steep,

            Condition::Hole => ctx.is_hole || ctx.surface_noise < -0.15,

            Condition::TemperatureCheck { max_temp } => {
                let temp = (ctx.biome.temperature() as f64) - (ctx.y as f64 - 64.0) * 0.00166667;
                temp <= *max_temp
            }

            Condition::Not(inner) => !inner.test(ctx),

            Condition::All(conditions) => {
                conditions.iter().all(|c| c.test(ctx))
            }

            Condition::Any(conditions) => {
                conditions.iter().any(|c| c.test(ctx))
            }
        }
    }
}

/// AST Surface Rule yielding blocks based on context.
/// REA Reference: net.minecraft.world.level.levelgen.SurfaceRules$RuleSource (dph$o.class)
#[derive(Clone, Debug)]
pub enum SurfaceRule {
    /// Direct block state placement.
    Block(SurfaceBlock),

    /// Conditional rule: executes `then_rule` if `condition` tests true.
    If {
        condition: Condition,
        then_rule: Box<SurfaceRule>,
    },

    /// Sequential rule list evaluated in priority order until one yields a block.
    Sequence(Vec<SurfaceRule>),

    /// Dynamic Badlands banded terracotta block placement based on Y and noise.
    BandTerracotta(TerracottaBands),
}

impl SurfaceRule {
    /// Evaluates this rule against the surface context.
    pub fn apply(&self, ctx: &SurfaceContext) -> Option<SurfaceBlock> {
        match self {
            SurfaceRule::Block(block) => Some(*block),

            SurfaceRule::If { condition, then_rule } => {
                if condition.test(ctx) {
                    then_rule.apply(ctx)
                } else {
                    None
                }
            }

            SurfaceRule::Sequence(rules) => {
                for rule in rules {
                    if let Some(block) = rule.apply(ctx) {
                        return Some(block);
                    }
                }
                None
            }

            SurfaceRule::BandTerracotta(bands) => {
                Some(bands.get_band(ctx.y, ctx.surface_noise))
            }
        }
    }
}

/// Surface rule evaluation engine for chunk surface generation.
#[derive(Clone, Debug)]
pub struct SurfaceRuleEngine {
    pub seed: u64,
    pub terracotta_bands: TerracottaBands,
    pub master_rule: SurfaceRule,
}

impl Default for SurfaceRuleEngine {
    fn default() -> Self {
        Self::new(192837465)
    }
}

impl SurfaceRuleEngine {
    pub fn new(seed: u64) -> Self {
        let terracotta_bands = TerracottaBands::from_seed(seed);
        let master_rule = build_overworld_rules(&terracotta_bands);
        Self {
            seed,
            terracotta_bands,
            master_rule,
        }
    }

    /// Evaluates the exact block for a given (x, y, z) voxel position given the local surface height,
    /// the biome ID, and the local offset noise value.
    pub fn get_block(
        &self,
        x: i32,
        y: i32,
        z: i32,
        surface_y: i32,
        biome: BiomeId,
        offset_noise: f64,
    ) -> SurfaceBlock {
        let ctx = SurfaceContext::new(x, y, z, surface_y, biome, offset_noise).with_seed(self.seed);
        self.get_block_with_context(&ctx)
    }

    /// Evaluates surface block using the full SurfaceContext (including steep and hole flags).
    pub fn get_block_with_context(&self, ctx: &SurfaceContext) -> SurfaceBlock {
        // 1. Below world bounds
        if ctx.y < MIN_Y {
            return SurfaceBlock::Air;
        }

        // 2. Bedrock bottom floor: evaluated via master_rule (Condition::VerticalGradient)
        // Solid bedrock floor at MIN_Y
        if ctx.y == MIN_Y {
            return SurfaceBlock::Bedrock;
        }

        // 3. Above the surface terrain: atmosphere or water ocean filling
        if ctx.y > ctx.surface_y {
            // Water ocean filling up to sea level (Y = 63)
            if ctx.y <= ctx.water_height
                && (ctx.biome.is_ocean() || ctx.biome == BiomeId::River || ctx.biome == BiomeId::FrozenRiver)
            {
                if ctx.biome == BiomeId::FrozenOcean
                    || ctx.biome == BiomeId::DeepFrozenOcean
                    || ctx.biome == BiomeId::FrozenRiver
                {
                    if ctx.y == ctx.water_height {
                        return SurfaceBlock::Ice;
                    }
                }
                return SurfaceBlock::Water;
            }

            // Single thin snow layer resting on top of solid snowy terrain
            if ctx.y == ctx.surface_y + 1 && ctx.biome.is_snowy() && ctx.surface_y >= ctx.water_height {
                return SurfaceBlock::SnowLayer;
            }

            return SurfaceBlock::Air;
        }

        // 4. Apply compiled AST master surface rule
        if let Some(block) = self.master_rule.apply(ctx) {
            return block;
        }

        // 5. Default base rock
        if ctx.y < 0 {
            SurfaceBlock::Deepslate
        } else {
            SurfaceBlock::Stone
        }
    }
}

/// Builds the canonical Minecraft Overworld surface rule AST tree.
/// REA Reference: net.minecraft.data.worldgen.SurfaceRuleData.overworld (pr.class)
pub fn build_overworld_rules(terracotta_bands: &TerracottaBands) -> SurfaceRule {
    let mut rules = Vec::new();

    // =========================================================================
    // 1. BEDROCK RULES (VerticalGradient)
    // =========================================================================
    rules.push(SurfaceRule::If {
        condition: Condition::VerticalGradient {
            random_name: "minecraft:bedrock_floor".to_string(),
            true_at_and_below: MIN_Y,
            false_at_and_above: MIN_Y + 5,
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
                    max_y: MAX_Y,
                },
                then_rule: Box::new(SurfaceRule::BandTerracotta(terracotta_bands.clone())),
            },
        ])),
    };
    rules.push(badlands_rule);

    // =========================================================================
    // 4. MOUNTAIN TOPS (Frozen Peaks, Jagged Peaks, Stony Peaks, Snowy Slopes)
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
