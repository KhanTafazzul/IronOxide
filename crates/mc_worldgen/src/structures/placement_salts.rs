//! Exact vanilla Minecraft 1.20+ RandomSpread structure placement constants,
//! salts, spacing, separation, and valid biomes.
//!
//! REA References:
//! - `net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement` (mapped: `ebc.class`)
//! - `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure` (mapped: `ebb.class`)
//! - `net.minecraft.world.level.levelgen.structure.structures.JungleTempleStructure` (mapped: `eca.class`)
//! - `net.minecraft.world.level.levelgen.structure.structures.SwampHutStructure` (mapped: `eby.class`)
//! - `data/minecraft/worldgen/structure_set/desert_pyramids.json`
//! - `data/minecraft/worldgen/structure_set/jungle_temples.json`
//! - `data/minecraft/worldgen/structure_set/swamp_huts.json`
//! - `data/minecraft/worldgen/structure_set/pillager_outposts.json`
//! - `data/minecraft/worldgen/structure_set/ancient_cities.json`

use crate::biomes::BiomeId;
use crate::structures::locator::{RandomSpreadPlacement, SpreadType};

/// Complete metadata for a vanilla random spread structure placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StructurePlacementDef {
    pub name: &'static str,
    pub spacing: i32,
    pub separation: i32,
    pub salt: i32,
    pub spread_type: SpreadType,
    pub valid_biomes: &'static [BiomeId],
    pub fixed_y: Option<i32>,
}

impl StructurePlacementDef {
    #[inline]
    pub const fn to_placement(&self) -> RandomSpreadPlacement {
        RandomSpreadPlacement::new(self.spacing, self.separation, self.salt, self.spread_type)
    }

    #[inline]
    pub fn is_valid_biome(&self, biome: BiomeId) -> bool {
        self.valid_biomes.contains(&biome)
    }
}

// =========================================================================
// 5 Missing Vanilla Structures (Task 57)
// =========================================================================

/// Desert Pyramid placement parameters:
/// Spacing = 32, Separation = 8, Salt = 14357617, Biomes = [Desert].
pub const DESERT_PYRAMID: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:desert_pyramid",
    spacing: 32,
    separation: 8,
    salt: 14357617,
    spread_type: SpreadType::Linear,
    valid_biomes: &[BiomeId::Desert],
    fixed_y: None,
};

/// Jungle Pyramid / Temple placement parameters:
/// Spacing = 32, Separation = 8, Salt = 14357619, Biomes = [Jungle, Bamboo Jungle].
pub const JUNGLE_PYRAMID: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:jungle_pyramid",
    spacing: 32,
    separation: 8,
    salt: 14357619,
    spread_type: SpreadType::Linear,
    valid_biomes: &[BiomeId::Jungle, BiomeId::BambooJungle],
    fixed_y: None,
};

/// Swamp Hut (Witch Hut) placement parameters:
/// Spacing = 32, Separation = 8, Salt = 14357620, Biomes = [Swamp].
pub const SWAMP_HUT: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:swamp_hut",
    spacing: 32,
    separation: 8,
    salt: 14357620,
    spread_type: SpreadType::Linear,
    valid_biomes: &[BiomeId::Swamp],
    fixed_y: None,
};

/// Pillager Outpost placement parameters:
/// Spacing = 80, Separation = 24, Salt = 165745296,
/// Biomes = [Plains, Desert, Savanna, Taiga, Snowy Plains, Meadow, Grove, Snowy Slopes].
pub const PILLAGER_OUTPOST: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:pillager_outpost",
    spacing: 80,
    separation: 24,
    salt: 165745296,
    spread_type: SpreadType::Linear,
    valid_biomes: &[
        BiomeId::Plains,
        BiomeId::Desert,
        BiomeId::Savanna,
        BiomeId::Taiga,
        BiomeId::SnowyPlains,
        BiomeId::Meadow,
        BiomeId::Grove,
        BiomeId::SnowySlopes,
    ],
    fixed_y: None,
};

/// Ancient City placement parameters:
/// Spacing = 24, Separation = 8, Salt = 20083232, Biomes = [Deep Dark], Y-anchor = fixed at Y = -51.
pub const ANCIENT_CITY: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:ancient_city",
    spacing: 24,
    separation: 8,
    salt: 20083232,
    spread_type: SpreadType::Linear,
    valid_biomes: &[BiomeId::DeepDark],
    fixed_y: Some(-51),
};

// =========================================================================
// Other Standard Structures for Full Coverage
// =========================================================================

pub const VILLAGES: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:village",
    spacing: 34,
    separation: 8,
    salt: 10387312,
    spread_type: SpreadType::Linear,
    valid_biomes: &[
        BiomeId::Plains,
        BiomeId::Desert,
        BiomeId::Savanna,
        BiomeId::Taiga,
        BiomeId::SnowyPlains,
        BiomeId::Meadow,
    ],
    fixed_y: None,
};

pub const TRIAL_CHAMBERS: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:trial_chambers",
    spacing: 34,
    separation: 12,
    salt: 94251327,
    spread_type: SpreadType::Linear,
    valid_biomes: &[],
    fixed_y: Some(-20),
};

pub const WOODLAND_MANSIONS: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:mansion",
    spacing: 80,
    separation: 20,
    salt: 10387313,
    spread_type: SpreadType::Triangular,
    valid_biomes: &[BiomeId::DarkForest],
    fixed_y: None,
};

pub const NETHER_FORTRESSES: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:fortress",
    spacing: 27,
    separation: 4,
    salt: 30084232,
    spread_type: SpreadType::Linear,
    valid_biomes: &[],
    fixed_y: None,
};

pub const BASTION_REMNANTS: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:bastion_remnant",
    spacing: 27,
    separation: 4,
    salt: 30084235,
    spread_type: SpreadType::Linear,
    valid_biomes: &[],
    fixed_y: None,
};

pub const END_CITIES: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:end_city",
    spacing: 20,
    separation: 11,
    salt: 10387313,
    spread_type: SpreadType::Triangular,
    valid_biomes: &[],
    fixed_y: None,
};

pub const OCEAN_MONUMENTS: StructurePlacementDef = StructurePlacementDef {
    name: "minecraft:monument",
    spacing: 32,
    separation: 5,
    salt: 10387313,
    spread_type: SpreadType::Triangular,
    valid_biomes: &[
        BiomeId::DeepOcean,
        BiomeId::DeepColdOcean,
        BiomeId::DeepFrozenOcean,
        BiomeId::DeepLukewarmOcean,
    ],
    fixed_y: None,
};
