//! Carver configuration and replaceable block rules.
//! REA Reference: net.minecraft.world.level.levelgen.carver.CarverConfiguration (dqo.class)
//! REA Reference: data/minecraft/tags/blocks/overworld_carver_replaceables.json

use crate::biomes::SurfaceBlock;

pub const SEA_LEVEL: i32 = 63;
pub const BEDROCK_CEILING_Y: i32 = -60;
pub const LAVA_LEVEL_Y: i32 = -55; // above_bottom: 8 from MIN_Y (-64)

/// Predicate evaluating if a surface/underworld block is replaceable by cave carvers.
/// REA Reference: #minecraft:overworld_carver_replaceables
#[inline]
pub fn is_carver_replaceable(block: SurfaceBlock) -> bool {
    match block {
        // Bedrock is strictly unbreakable and NEVER carved
        SurfaceBlock::Bedrock => false,
        SurfaceBlock::Air => false,

        // Solid overworld stone, dirt, and sedimentary layers
        SurfaceBlock::Stone
        | SurfaceBlock::Deepslate
        | SurfaceBlock::Dirt
        | SurfaceBlock::GrassBlock
        | SurfaceBlock::Sand
        | SurfaceBlock::Sandstone
        | SurfaceBlock::RedSand
        | SurfaceBlock::RedSandstone
        | SurfaceBlock::Gravel
        | SurfaceBlock::SnowBlock
        | SurfaceBlock::SnowLayer
        | SurfaceBlock::PowderSnow
        | SurfaceBlock::Ice
        | SurfaceBlock::PackedIce
        | SurfaceBlock::Podzol
        | SurfaceBlock::CoarseDirt
        | SurfaceBlock::Mud
        | SurfaceBlock::Water => true,

        // Badlands terracotta strata
        SurfaceBlock::Terracotta
        | SurfaceBlock::WhiteTerracotta
        | SurfaceBlock::OrangeTerracotta
        | SurfaceBlock::YellowTerracotta
        | SurfaceBlock::BrownTerracotta
        | SurfaceBlock::RedTerracotta
        | SurfaceBlock::LightGrayTerracotta => true,

        // Ore blocks embedded in rock
        SurfaceBlock::CoalOre
        | SurfaceBlock::DeepslateCoalOre
        | SurfaceBlock::IronOre
        | SurfaceBlock::DeepslateIronOre
        | SurfaceBlock::CopperOre
        | SurfaceBlock::DeepslateCopperOre
        | SurfaceBlock::GoldOre
        | SurfaceBlock::DeepslateGoldOre
        | SurfaceBlock::RedstoneOre
        | SurfaceBlock::DeepslateRedstoneOre
        | SurfaceBlock::DiamondOre
        | SurfaceBlock::DeepslateDiamondOre
        | SurfaceBlock::EmeraldOre
        | SurfaceBlock::DeepslateEmeraldOre
        | SurfaceBlock::LapisOre
        | SurfaceBlock::DeepslateLapisOre => true,
    }
}

/// Carved block state determined by elevation relative to sea level and lava pools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarvedState {
    /// Carved pocket exposed to atmosphere
    Air,
    /// Subterranean aquifer below sea level Y=63
    Water,
    /// Deep cavern lava pool below Y=-55
    Lava,
}

/// Determines what fluid/air block should fill a carved voxel at height Y.
#[inline]
pub fn carved_fluid_or_air(y: i32) -> CarvedState {
    if y <= LAVA_LEVEL_Y {
        CarvedState::Lava
    } else if y <= SEA_LEVEL {
        CarvedState::Water
    } else {
        CarvedState::Air
    }
}

/// Base configuration settings matching net.minecraft.world.level.levelgen.carver.CarverConfiguration.
#[derive(Clone, Debug)]
pub struct CarverConfiguration {
    pub probability: f32,
    pub min_y: i32,
    pub max_y: i32,
    pub lava_level: i32,
}

impl Default for CarverConfiguration {
    fn default() -> Self {
        Self {
            probability: 0.15,
            min_y: -56,
            max_y: 180,
            lava_level: LAVA_LEVEL_Y,
        }
    }
}

/// Specific configuration for Canyon / Ravine carving.
/// REA Reference: data/minecraft/worldgen/configured_carver/canyon.json
#[derive(Clone, Debug)]
pub struct CanyonCarverConfiguration {
    pub base: CarverConfiguration,
    pub distance_factor_min: f32,
    pub distance_factor_max: f32,
    pub thickness_min: f32,
    pub thickness_max: f32,
    pub y_scale: f32,
}

impl Default for CanyonCarverConfiguration {
    fn default() -> Self {
        Self {
            base: CarverConfiguration {
                probability: 0.01,
                min_y: 10,
                max_y: 67,
                lava_level: LAVA_LEVEL_Y,
            },
            distance_factor_min: 0.75,
            distance_factor_max: 1.0,
            thickness_min: 0.0,
            thickness_max: 6.0,
            y_scale: 3.0,
        }
    }
}
