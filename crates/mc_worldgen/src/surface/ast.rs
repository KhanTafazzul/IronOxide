//! Surface Rule AST definitions: Conditions, SurfaceRules, and Evaluation Context.
//! REA Reference: net.minecraft.world.level.levelgen.SurfaceRules (dph.class)
//! REA Reference: net.minecraft.world.level.levelgen.SurfaceRules$Condition (dph$b.class)
//! REA Reference: net.minecraft.world.level.levelgen.SurfaceRules$SurfaceRule (dph$u.class)

use crate::biomes::{BiomeId, SurfaceBlock, TerracottaBands};

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
    /// Whether the terrain slope is steep (e.g. mountain cliff walls).
    pub is_steep: bool,
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
        }
    }
}

/// AST Condition evaluating boolean predicates on the surface context.
/// REA Reference: net.minecraft.world.level.levelgen.SurfaceRules$ConditionSource
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

    /// Steep terrain cliff condition.
    /// Matches `SurfaceRules.steep()`.
    Steep,

    /// Logical NOT inverting child condition.
    /// Matches `SurfaceRules.not()`.
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

            Condition::Steep => ctx.is_steep,

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
/// REA Reference: net.minecraft.world.level.levelgen.SurfaceRules$RuleSource
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
