// REA Verified: net.minecraft.world.level.block.Blocks (mapped: cws.class)
// REA Verified: net.minecraft.world.level.block.state.BlockBehaviour.Properties (mapped: djg$d.class)
// REA Verified: net.minecraft.world.level.block.StairBlock (mapped: dej.class)
// REA Verified: net.minecraft.world.level.block.SlabBlock (mapped: dds.class)
// REA Verified: net.minecraft.world.level.block.EndRodBlock (mapped: czg.class / dcy.class)
//
// Comprehensive Minecraft 1.20.4 Block Registry & Voxel Shape Engine.
// Supports all 3 dimensions (Overworld, Nether, The End), property metadata flags,
// non-cubic bounding boxes, raycast selection, and entity collision resolution.

use glam::{IVec3, Vec3};
use crate::game::physics::Aabb;

/// Canonical Minecraft Block Identifiers across Overworld, Nether, and The End
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockId {
    // --- Core Utilities & Fluids ---
    Air,
    Dirt,
    GrassBlock,
    Bedrock,
    Sand,
    Gravel,
    Water,
    Lava,
    Vine,
    Torch,
    Chest,

    // --- Overworld: Stone Types & Minerals ---
    Stone,
    Cobblestone,
    Andesite,
    Granite,
    Diorite,
    Deepslate,
    Bricks,
    Glass,
    Obsidian,

    // --- Overworld: Wood Variants ---
    OakLog,
    OakPlanks,
    OakLeaves,
    BirchLog,
    BirchPlanks,
    BirchLeaves,
    SpruceLog,
    SprucePlanks,
    SpruceLeaves,
    DarkOakLog,
    DarkOakPlanks,
    DarkOakLeaves,

    // --- Overworld: Ores ---
    CoalOre,
    IronOre,
    CopperOre,
    GoldOre,
    DiamondOre,
    RedstoneOre,
    LapisOre,
    EmeraldOre,

    // --- Overworld: Non-Cubic Slabs & Stairs ---
    OakSlab,
    StoneSlab,
    CobblestoneSlab,
    OakStairs,
    CobblestoneStairs,

    // --- Nether: Terrain, Fungus & Structures ---
    Netherrack,
    SoulSand,
    SoulSoil,
    Basalt,
    PolishedBasalt,
    Blackstone,
    NetherBricks,
    Glowstone,
    CrimsonNylium,
    WarpedNylium,
    CrimsonStem,
    WarpedStem,
    NetherWartBlock,
    WarpedWartBlock,
    CryingObsidian,
    BoneBlock,
    MagmaBlock,

    // --- The End: Terrain, Structures & Utilities ---
    EndStone,
    EndStoneBricks,
    PurpurBlock,
    PurpurPillar,
    EndRod,
    PurpurStairs,
    PurpurSlab,
    EndPortalFrame,
    ChorusPlant,
    ChorusFlower,
    NetherPortal,
    EndPortal,
    DragonHead,
}

/// Sound type classification matching net.minecraft.world.level.block.SoundType (dec.class)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundFamily {
    Stone,
    Wood,
    Grass,
    Gravel,
    Sand,
    Glass,
    Metal,
    Basalt,
    SoulSand,
    NetherWart,
}

/// Preferred tool classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    None,
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
}

/// Harvest tier classification matching vanilla
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HarvestTier {
    Hand = 0,
    Wood = 1,
    Stone = 2,
    Iron = 3,
    Diamond = 4,
    Netherite = 5,
}

/// Non-cubic shape descriptor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelShapeKind {
    FullCube,
    BottomSlab,
    TopSlab,
    DoubleSlab,
    StairsNorth,
    StairsSouth,
    StairsWest,
    StairsEast,
    EndRodY,
    EndRodX,
    EndRodZ,
    TorchFloor,
    EndPortalFrame,
    ChestBox,
    Empty,
}

/// Complete block property metadata extracted from net.minecraft.world.level.block.state.BlockBehaviour$Properties
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockProperties {
    /// Affects player and entity collision resolution (solid collision boxes)
    pub is_solid: bool,
    /// Dictates face-culling against adjacent blocks (true = neighbor faces remain visible)
    pub is_transparent: bool,
    /// Light emission level (0 to 15)
    pub light_emission: u8,
    /// Hardness value determining break time (-1.0 for unbreakable)
    pub hardness: f32,
    /// Resistance against explosion blast damage
    pub blast_resistance: f32,
    /// Audio feedback category
    pub sound_family: SoundFamily,
    /// Tool category required for efficient harvesting
    pub tool_required: ToolType,
    /// Minimum tool material tier required for drops
    pub min_harvest_tier: HarvestTier,
    /// Geometric voxel shape type
    pub shape_kind: VoxelShapeKind,
}

impl BlockId {
    /// Lookup properties for this block
    pub fn properties(&self) -> BlockProperties {
        match self {
            // --- Core Utilities & Fluids ---
            BlockId::Air => BlockProperties {
                is_solid: false,
                is_transparent: true,
                light_emission: 0,
                hardness: 0.0,
                blast_resistance: 0.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::Empty,
            },
            BlockId::Dirt => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 0.5,
                blast_resistance: 0.5,
                sound_family: SoundFamily::Grass,
                tool_required: ToolType::Shovel,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::GrassBlock => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 0.5,
                blast_resistance: 0.6,
                sound_family: SoundFamily::Grass,
                tool_required: ToolType::Shovel,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Bedrock => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: -1.0, // Unbreakable
                blast_resistance: 3_600_000.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Sand => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 0.5,
                blast_resistance: 0.5,
                sound_family: SoundFamily::Sand,
                tool_required: ToolType::Shovel,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Gravel => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 0.6,
                blast_resistance: 0.6,
                sound_family: SoundFamily::Gravel,
                tool_required: ToolType::Shovel,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Water => BlockProperties {
                is_solid: false,
                is_transparent: true,
                light_emission: 0,
                hardness: 100.0,
                blast_resistance: 100.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::Empty,
            },
            BlockId::Lava => BlockProperties {
                is_solid: false,
                is_transparent: true,
                light_emission: 15,
                hardness: 100.0,
                blast_resistance: 100.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::Empty,
            },
            BlockId::Vine => BlockProperties {
                is_solid: false,
                is_transparent: true,
                light_emission: 0,
                hardness: 0.2,
                blast_resistance: 0.2,
                sound_family: SoundFamily::Grass,
                tool_required: ToolType::Axe,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::Empty,
            },
            BlockId::Torch => BlockProperties {
                is_solid: false,
                is_transparent: true,
                light_emission: 14,
                hardness: 0.0,
                blast_resistance: 0.0,
                sound_family: SoundFamily::Wood,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::TorchFloor,
            },
            BlockId::Chest => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 0,
                hardness: 2.5,
                blast_resistance: 2.5,
                sound_family: SoundFamily::Wood,
                tool_required: ToolType::Axe,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::ChestBox,
            },

            // --- Overworld: Stone Types & Minerals ---
            BlockId::Stone => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 1.5,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Cobblestone => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 2.0,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Andesite => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 1.5,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Granite => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 1.5,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Diorite => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 1.5,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Deepslate => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 3.0,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Bricks => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 2.0,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Glass => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 0,
                hardness: 0.3,
                blast_resistance: 0.3,
                sound_family: SoundFamily::Glass,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Obsidian => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 50.0,
                blast_resistance: 1200.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Diamond,
                shape_kind: VoxelShapeKind::FullCube,
            },

            // --- Overworld: Wood Variants ---
            BlockId::OakLog | BlockId::BirchLog | BlockId::SpruceLog | BlockId::DarkOakLog => {
                BlockProperties {
                    is_solid: true,
                    is_transparent: false,
                    light_emission: 0,
                    hardness: 2.0,
                    blast_resistance: 2.0,
                    sound_family: SoundFamily::Wood,
                    tool_required: ToolType::Axe,
                    min_harvest_tier: HarvestTier::Hand,
                    shape_kind: VoxelShapeKind::FullCube,
                }
            }
            BlockId::OakPlanks | BlockId::BirchPlanks | BlockId::SprucePlanks | BlockId::DarkOakPlanks => {
                BlockProperties {
                    is_solid: true,
                    is_transparent: false,
                    light_emission: 0,
                    hardness: 2.0,
                    blast_resistance: 3.0,
                    sound_family: SoundFamily::Wood,
                    tool_required: ToolType::Axe,
                    min_harvest_tier: HarvestTier::Hand,
                    shape_kind: VoxelShapeKind::FullCube,
                }
            }
            BlockId::OakLeaves | BlockId::BirchLeaves | BlockId::SpruceLeaves | BlockId::DarkOakLeaves => {
                BlockProperties {
                    is_solid: true,
                    is_transparent: true,
                    light_emission: 0,
                    hardness: 0.2,
                    blast_resistance: 0.2,
                    sound_family: SoundFamily::Grass,
                    tool_required: ToolType::Hoe,
                    min_harvest_tier: HarvestTier::Hand,
                    shape_kind: VoxelShapeKind::FullCube,
                }
            }

            // --- Overworld: Ores ---
            BlockId::CoalOre => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 3.0,
                blast_resistance: 3.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::IronOre | BlockId::CopperOre | BlockId::LapisOre => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 3.0,
                blast_resistance: 3.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Stone,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::GoldOre | BlockId::RedstoneOre | BlockId::DiamondOre | BlockId::EmeraldOre => {
                BlockProperties {
                    is_solid: true,
                    is_transparent: false,
                    light_emission: 0,
                    hardness: 3.0,
                    blast_resistance: 3.0,
                    sound_family: SoundFamily::Stone,
                    tool_required: ToolType::Pickaxe,
                    min_harvest_tier: HarvestTier::Iron,
                    shape_kind: VoxelShapeKind::FullCube,
                }
            }

            // --- Overworld: Slabs & Stairs ---
            BlockId::OakSlab => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 0,
                hardness: 2.0,
                blast_resistance: 3.0,
                sound_family: SoundFamily::Wood,
                tool_required: ToolType::Axe,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::BottomSlab,
            },
            BlockId::StoneSlab | BlockId::CobblestoneSlab => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 0,
                hardness: 2.0,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::BottomSlab,
            },
            BlockId::OakStairs => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 0,
                hardness: 2.0,
                blast_resistance: 3.0,
                sound_family: SoundFamily::Wood,
                tool_required: ToolType::Axe,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::StairsNorth,
            },
            BlockId::CobblestoneStairs => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 0,
                hardness: 2.0,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::StairsNorth,
            },

            // --- Nether ---
            BlockId::Netherrack => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 0.4,
                blast_resistance: 0.4,
                sound_family: SoundFamily::NetherWart,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::SoulSand | BlockId::SoulSoil => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 0.5,
                blast_resistance: 0.5,
                sound_family: SoundFamily::SoulSand,
                tool_required: ToolType::Shovel,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Basalt | BlockId::PolishedBasalt => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 1.25,
                blast_resistance: 4.2,
                sound_family: SoundFamily::Basalt,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Blackstone => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 1.5,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::NetherBricks => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 2.0,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::Glowstone => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 15,
                hardness: 0.3,
                blast_resistance: 0.3,
                sound_family: SoundFamily::Glass,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::CrimsonNylium | BlockId::WarpedNylium => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 0.4,
                blast_resistance: 0.4,
                sound_family: SoundFamily::NetherWart,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::CrimsonStem | BlockId::WarpedStem => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 2.0,
                blast_resistance: 2.0,
                sound_family: SoundFamily::Wood,
                tool_required: ToolType::Axe,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::NetherWartBlock | BlockId::WarpedWartBlock => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 1.0,
                blast_resistance: 1.0,
                sound_family: SoundFamily::NetherWart,
                tool_required: ToolType::Hoe,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::CryingObsidian => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 10,
                hardness: 50.0,
                blast_resistance: 1200.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Diamond,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::BoneBlock => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 2.0,
                blast_resistance: 2.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::MagmaBlock => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 3,
                hardness: 0.5,
                blast_resistance: 0.5,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },

            // --- The End ---
            BlockId::EndStone => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 3.0,
                blast_resistance: 9.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::EndStoneBricks => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 3.0,
                blast_resistance: 9.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::PurpurBlock | BlockId::PurpurPillar => BlockProperties {
                is_solid: true,
                is_transparent: false,
                light_emission: 0,
                hardness: 1.5,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::EndRod => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 14,
                hardness: 0.0,
                blast_resistance: 0.0,
                sound_family: SoundFamily::Wood,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::EndRodY,
            },
            BlockId::PurpurStairs => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 0,
                hardness: 1.5,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::StairsNorth,
            },
            BlockId::PurpurSlab => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 0,
                hardness: 1.5,
                blast_resistance: 6.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::Pickaxe,
                min_harvest_tier: HarvestTier::Wood,
                shape_kind: VoxelShapeKind::BottomSlab,
            },
            BlockId::EndPortalFrame => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 1,
                hardness: -1.0, // Unbreakable
                blast_resistance: 3_600_000.0,
                sound_family: SoundFamily::Glass,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::EndPortalFrame,
            },
            BlockId::ChorusPlant | BlockId::ChorusFlower => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 0,
                hardness: 0.4,
                blast_resistance: 0.4,
                sound_family: SoundFamily::Wood,
                tool_required: ToolType::Axe,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
            BlockId::NetherPortal => BlockProperties {
                is_solid: false,
                is_transparent: true,
                light_emission: 11,
                hardness: -1.0,
                blast_resistance: 0.0,
                sound_family: SoundFamily::Glass,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::Empty,
            },
            BlockId::EndPortal => BlockProperties {
                is_solid: false,
                is_transparent: true,
                light_emission: 15,
                hardness: -1.0,
                blast_resistance: 3_600_000.0,
                sound_family: SoundFamily::Glass,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::Empty,
            },
            BlockId::DragonHead => BlockProperties {
                is_solid: true,
                is_transparent: true,
                light_emission: 0,
                hardness: 1.0,
                blast_resistance: 1.0,
                sound_family: SoundFamily::Stone,
                tool_required: ToolType::None,
                min_harvest_tier: HarvestTier::Hand,
                shape_kind: VoxelShapeKind::FullCube,
            },
        }
    }

    /// Whether this block affects player and entity physics movement
    #[inline]
    pub fn is_solid(&self) -> bool {
        self.properties().is_solid
    }

    /// Whether neighbor blocks can see through this block face
    #[inline]
    pub fn is_transparent(&self) -> bool {
        self.properties().is_transparent
    }

    /// Fully opaque cube (solid and non-transparent)
    #[inline]
    pub fn is_opaque(&self) -> bool {
        let p = self.properties();
        p.is_solid && !p.is_transparent && p.shape_kind == VoxelShapeKind::FullCube
    }

    /// Light level emitted by this block (0 to 15)
    #[inline]
    pub fn light_emission(&self) -> u8 {
        self.properties().light_emission
    }

    /// Block hardness for mining speed calculation (-1.0 for unbreakable)
    #[inline]
    pub fn hardness(&self) -> f32 {
        self.properties().hardness
    }

    /// Blast resistance against explosions
    #[inline]
    pub fn blast_resistance(&self) -> f32 {
        self.properties().blast_resistance
    }

    /// Whether this block is air
    #[inline]
    pub fn is_air(&self) -> bool {
        matches!(self, BlockId::Air)
    }

    /// Whether this block is a dynamic fluid (Water or Lava)
    #[inline]
    pub fn is_fluid(&self) -> bool {
        matches!(self, BlockId::Water | BlockId::Lava)
    }

    /// Whether light rays can propagate through this block without complete occlusion
    #[inline]
    pub fn is_transparent_for_light(&self) -> bool {
        match self {
            BlockId::Air
            | BlockId::Glass
            | BlockId::Water
            | BlockId::Torch
            | BlockId::Vine
            | BlockId::EndRod
            | BlockId::OakLeaves
            | BlockId::BirchLeaves
            | BlockId::SpruceLeaves
            | BlockId::DarkOakLeaves
            | BlockId::OakSlab
            | BlockId::StoneSlab
            | BlockId::CobblestoneSlab
            | BlockId::PurpurSlab
            | BlockId::OakStairs
            | BlockId::CobblestoneStairs
            | BlockId::PurpurStairs
            | BlockId::EndPortalFrame => true,
            _ => false,
        }
    }

    /// Returns non-cubic compound collision AABBs in world coordinates for entity physics
    pub fn collision_boxes(&self, pos: IVec3) -> Vec<Aabb> {
        let p = self.properties();
        if !p.is_solid {
            return Vec::new();
        }

        let origin = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
        match p.shape_kind {
            VoxelShapeKind::FullCube | VoxelShapeKind::DoubleSlab => {
                vec![Aabb::new(origin, origin + Vec3::ONE)]
            }
            VoxelShapeKind::BottomSlab => {
                // REA Verified: SlabBlock.java dds.d = AABB(0, 0, 0, 16, 8, 16)
                vec![Aabb::new(origin, origin + Vec3::new(1.0, 0.5, 1.0))]
            }
            VoxelShapeKind::TopSlab => {
                // REA Verified: SlabBlock.java dds.e = AABB(0, 8, 0, 16, 16, 16)
                vec![Aabb::new(
                    origin + Vec3::new(0.0, 0.5, 0.0),
                    origin + Vec3::ONE,
                )]
            }
            VoxelShapeKind::StairsNorth => {
                // REA Verified: StairBlock.java dej.class
                // Base slab [0, 0, 0] -> [1, 0.5, 1] + Upper step (North half) [0, 0.5, 0] -> [1, 1, 0.5]
                vec![
                    Aabb::new(origin, origin + Vec3::new(1.0, 0.5, 1.0)),
                    Aabb::new(
                        origin + Vec3::new(0.0, 0.5, 0.0),
                        origin + Vec3::new(1.0, 1.0, 0.5),
                    ),
                ]
            }
            VoxelShapeKind::StairsSouth => {
                // Base slab + Upper step (South half) [0, 0.5, 0.5] -> [1, 1, 1]
                vec![
                    Aabb::new(origin, origin + Vec3::new(1.0, 0.5, 1.0)),
                    Aabb::new(
                        origin + Vec3::new(0.0, 0.5, 0.5),
                        origin + Vec3::new(1.0, 1.0, 1.0),
                    ),
                ]
            }
            VoxelShapeKind::StairsWest => {
                // Base slab + Upper step (West half) [0, 0.5, 0] -> [0.5, 1, 1]
                vec![
                    Aabb::new(origin, origin + Vec3::new(1.0, 0.5, 1.0)),
                    Aabb::new(
                        origin + Vec3::new(0.0, 0.5, 0.0),
                        origin + Vec3::new(0.5, 1.0, 1.0),
                    ),
                ]
            }
            VoxelShapeKind::StairsEast => {
                // Base slab + Upper step (East half) [0.5, 0.5, 0] -> [1, 1, 1]
                vec![
                    Aabb::new(origin, origin + Vec3::new(1.0, 0.5, 1.0)),
                    Aabb::new(
                        origin + Vec3::new(0.5, 0.5, 0.0),
                        origin + Vec3::new(1.0, 1.0, 1.0),
                    ),
                ]
            }
            VoxelShapeKind::EndRodY => {
                // REA Verified: RodBlock.java dcy.h = AABB(6, 0, 6, 10, 16, 10) -> [0.375, 0, 0.375] to [0.625, 1, 0.625]
                vec![Aabb::new(
                    origin + Vec3::new(0.375, 0.0, 0.375),
                    origin + Vec3::new(0.625, 1.0, 0.625),
                )]
            }
            VoxelShapeKind::EndRodX => {
                // REA Verified: RodBlock.java dcy.j = AABB(0, 6, 6, 16, 10, 10)
                vec![Aabb::new(
                    origin + Vec3::new(0.0, 0.375, 0.375),
                    origin + Vec3::new(1.0, 0.625, 0.625),
                )]
            }
            VoxelShapeKind::EndRodZ => {
                // REA Verified: RodBlock.java dcy.i = AABB(6, 6, 0, 10, 10, 16)
                vec![Aabb::new(
                    origin + Vec3::new(0.375, 0.375, 0.0),
                    origin + Vec3::new(0.625, 0.625, 1.0),
                )]
            }
            VoxelShapeKind::EndPortalFrame => {
                // REA Verified: EndPortalFrameBlock.java = AABB(0, 0, 0, 16, 13, 16) -> [0, 0, 0] to [1, 0.8125, 1]
                vec![Aabb::new(origin, origin + Vec3::new(1.0, 0.8125, 1.0))]
            }
            VoxelShapeKind::ChestBox => {
                // Chest AABB(1, 0, 1, 15, 14, 15) -> [0.0625, 0, 0.0625] to [0.9375, 0.875, 0.9375]
                vec![Aabb::new(
                    origin + Vec3::new(0.0625, 0.0, 0.0625),
                    origin + Vec3::new(0.9375, 0.875, 0.9375),
                )]
            }
            VoxelShapeKind::TorchFloor | VoxelShapeKind::Empty => Vec::new(),
        }
    }

    /// Returns selection bounding boxes in world coordinates for raycast crosshair targeting
    pub fn selection_boxes(&self, pos: IVec3) -> Vec<Aabb> {
        let p = self.properties();
        let origin = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);

        match p.shape_kind {
            VoxelShapeKind::TorchFloor => {
                // Centered thin pillar [0.375, 0.0, 0.375] -> [0.625, 0.625, 0.625]
                vec![Aabb::new(
                    origin + Vec3::new(0.375, 0.0, 0.375),
                    origin + Vec3::new(0.625, 0.625, 0.625),
                )]
            }
            VoxelShapeKind::Empty => Vec::new(),
            _ => self.collision_boxes(pos),
        }
    }

    /// Converts BlockId to vanilla namespaced identifier for Anvil storage
    pub fn vanilla_name(&self) -> &'static str {
        match self {
            BlockId::Air => "minecraft:air",
            BlockId::Dirt => "minecraft:dirt",
            BlockId::GrassBlock => "minecraft:grass_block",
            BlockId::Bedrock => "minecraft:bedrock",
            BlockId::Sand => "minecraft:sand",
            BlockId::Gravel => "minecraft:gravel",
            BlockId::Water => "minecraft:water",
            BlockId::Lava => "minecraft:lava",
            BlockId::Vine => "minecraft:vine",
            BlockId::Torch => "minecraft:torch",
            BlockId::Chest => "minecraft:chest",

            BlockId::Stone => "minecraft:stone",
            BlockId::Cobblestone => "minecraft:cobblestone",
            BlockId::Andesite => "minecraft:andesite",
            BlockId::Granite => "minecraft:granite",
            BlockId::Diorite => "minecraft:diorite",
            BlockId::Deepslate => "minecraft:deepslate",
            BlockId::Bricks => "minecraft:bricks",
            BlockId::Glass => "minecraft:glass",
            BlockId::Obsidian => "minecraft:obsidian",

            BlockId::OakLog => "minecraft:oak_log",
            BlockId::OakPlanks => "minecraft:oak_planks",
            BlockId::OakLeaves => "minecraft:oak_leaves",
            BlockId::BirchLog => "minecraft:birch_log",
            BlockId::BirchPlanks => "minecraft:birch_planks",
            BlockId::BirchLeaves => "minecraft:birch_leaves",
            BlockId::SpruceLog => "minecraft:spruce_log",
            BlockId::SprucePlanks => "minecraft:spruce_planks",
            BlockId::SpruceLeaves => "minecraft:spruce_leaves",
            BlockId::DarkOakLog => "minecraft:dark_oak_log",
            BlockId::DarkOakPlanks => "minecraft:dark_oak_planks",
            BlockId::DarkOakLeaves => "minecraft:dark_oak_leaves",

            BlockId::CoalOre => "minecraft:coal_ore",
            BlockId::IronOre => "minecraft:iron_ore",
            BlockId::CopperOre => "minecraft:copper_ore",
            BlockId::GoldOre => "minecraft:gold_ore",
            BlockId::DiamondOre => "minecraft:diamond_ore",
            BlockId::RedstoneOre => "minecraft:redstone_ore",
            BlockId::LapisOre => "minecraft:lapis_ore",
            BlockId::EmeraldOre => "minecraft:emerald_ore",

            BlockId::OakSlab => "minecraft:oak_slab",
            BlockId::StoneSlab => "minecraft:stone_slab",
            BlockId::CobblestoneSlab => "minecraft:cobblestone_slab",
            BlockId::OakStairs => "minecraft:oak_stairs",
            BlockId::CobblestoneStairs => "minecraft:cobblestone_stairs",

            BlockId::Netherrack => "minecraft:netherrack",
            BlockId::SoulSand => "minecraft:soul_sand",
            BlockId::SoulSoil => "minecraft:soul_soil",
            BlockId::Basalt => "minecraft:basalt",
            BlockId::PolishedBasalt => "minecraft:polished_basalt",
            BlockId::Blackstone => "minecraft:blackstone",
            BlockId::NetherBricks => "minecraft:nether_bricks",
            BlockId::Glowstone => "minecraft:glowstone",
            BlockId::CrimsonNylium => "minecraft:crimson_nylium",
            BlockId::WarpedNylium => "minecraft:warped_nylium",
            BlockId::CrimsonStem => "minecraft:crimson_stem",
            BlockId::WarpedStem => "minecraft:warped_stem",
            BlockId::NetherWartBlock => "minecraft:nether_wart_block",
            BlockId::WarpedWartBlock => "minecraft:warped_wart_block",
            BlockId::CryingObsidian => "minecraft:crying_obsidian",
            BlockId::BoneBlock => "minecraft:bone_block",
            BlockId::MagmaBlock => "minecraft:magma_block",

            BlockId::EndStone => "minecraft:end_stone",
            BlockId::EndStoneBricks => "minecraft:end_stone_bricks",
            BlockId::PurpurBlock => "minecraft:purpur_block",
            BlockId::PurpurPillar => "minecraft:purpur_pillar",
            BlockId::EndRod => "minecraft:end_rod",
            BlockId::PurpurStairs => "minecraft:purpur_stairs",
            BlockId::PurpurSlab => "minecraft:purpur_slab",
            BlockId::EndPortalFrame => "minecraft:end_portal_frame",
            BlockId::ChorusPlant => "minecraft:chorus_plant",
            BlockId::ChorusFlower => "minecraft:chorus_flower",
            BlockId::NetherPortal => "minecraft:nether_portal",
            BlockId::EndPortal => "minecraft:end_portal",
            BlockId::DragonHead => "minecraft:dragon_head",
        }
    }

    /// Parse vanilla namespaced identifier into BlockId
    pub fn from_vanilla_name(name: &str) -> Option<BlockId> {
        let clean = name.strip_prefix("minecraft:").unwrap_or(name);
        match clean {
            "air" => Some(BlockId::Air),
            "dirt" => Some(BlockId::Dirt),
            "grass_block" => Some(BlockId::GrassBlock),
            "bedrock" => Some(BlockId::Bedrock),
            "sand" => Some(BlockId::Sand),
            "gravel" => Some(BlockId::Gravel),
            "water" => Some(BlockId::Water),
            "lava" => Some(BlockId::Lava),
            "vine" => Some(BlockId::Vine),
            "torch" => Some(BlockId::Torch),
            "chest" => Some(BlockId::Chest),

            "stone" => Some(BlockId::Stone),
            "cobblestone" => Some(BlockId::Cobblestone),
            "andesite" => Some(BlockId::Andesite),
            "granite" => Some(BlockId::Granite),
            "diorite" => Some(BlockId::Diorite),
            "deepslate" => Some(BlockId::Deepslate),
            "bricks" => Some(BlockId::Bricks),
            "glass" => Some(BlockId::Glass),
            "obsidian" => Some(BlockId::Obsidian),

            "oak_log" => Some(BlockId::OakLog),
            "oak_planks" => Some(BlockId::OakPlanks),
            "oak_leaves" => Some(BlockId::OakLeaves),
            "birch_log" => Some(BlockId::BirchLog),
            "birch_planks" => Some(BlockId::BirchPlanks),
            "birch_leaves" => Some(BlockId::BirchLeaves),
            "spruce_log" => Some(BlockId::SpruceLog),
            "spruce_planks" => Some(BlockId::SprucePlanks),
            "spruce_leaves" => Some(BlockId::SpruceLeaves),
            "dark_oak_log" => Some(BlockId::DarkOakLog),
            "dark_oak_planks" => Some(BlockId::DarkOakPlanks),
            "dark_oak_leaves" => Some(BlockId::DarkOakLeaves),

            "coal_ore" => Some(BlockId::CoalOre),
            "iron_ore" => Some(BlockId::IronOre),
            "copper_ore" => Some(BlockId::CopperOre),
            "gold_ore" => Some(BlockId::GoldOre),
            "diamond_ore" => Some(BlockId::DiamondOre),
            "redstone_ore" => Some(BlockId::RedstoneOre),
            "lapis_ore" => Some(BlockId::LapisOre),
            "emerald_ore" => Some(BlockId::EmeraldOre),

            "oak_slab" => Some(BlockId::OakSlab),
            "stone_slab" => Some(BlockId::StoneSlab),
            "cobblestone_slab" => Some(BlockId::CobblestoneSlab),
            "oak_stairs" => Some(BlockId::OakStairs),
            "cobblestone_stairs" => Some(BlockId::CobblestoneStairs),

            "netherrack" => Some(BlockId::Netherrack),
            "soul_sand" => Some(BlockId::SoulSand),
            "soul_soil" => Some(BlockId::SoulSoil),
            "basalt" => Some(BlockId::Basalt),
            "polished_basalt" => Some(BlockId::PolishedBasalt),
            "blackstone" => Some(BlockId::Blackstone),
            "nether_bricks" => Some(BlockId::NetherBricks),
            "glowstone" => Some(BlockId::Glowstone),
            "crimson_nylium" => Some(BlockId::CrimsonNylium),
            "warped_nylium" => Some(BlockId::WarpedNylium),
            "crimson_stem" => Some(BlockId::CrimsonStem),
            "warped_stem" => Some(BlockId::WarpedStem),
            "nether_wart_block" => Some(BlockId::NetherWartBlock),
            "warped_wart_block" => Some(BlockId::WarpedWartBlock),
            "crying_obsidian" => Some(BlockId::CryingObsidian),
            "bone_block" => Some(BlockId::BoneBlock),
            "magma_block" => Some(BlockId::MagmaBlock),

            "end_stone" => Some(BlockId::EndStone),
            "end_stone_bricks" => Some(BlockId::EndStoneBricks),
            "purpur_block" => Some(BlockId::PurpurBlock),
            "purpur_pillar" => Some(BlockId::PurpurPillar),
            "end_rod" => Some(BlockId::EndRod),
            "purpur_stairs" => Some(BlockId::PurpurStairs),
            "purpur_slab" => Some(BlockId::PurpurSlab),
            "end_portal_frame" => Some(BlockId::EndPortalFrame),
            "chorus_plant" => Some(BlockId::ChorusPlant),
            "chorus_flower" => Some(BlockId::ChorusFlower),
            "nether_portal" => Some(BlockId::NetherPortal),
            "end_portal" => Some(BlockId::EndPortal),
            "dragon_head" => Some(BlockId::DragonHead),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expanded_block_id_properties() {
        // Overworld tests
        assert_eq!(BlockId::Air.is_solid(), false);
        assert_eq!(BlockId::Air.is_transparent(), true);
        assert_eq!(BlockId::Stone.is_solid(), true);
        assert_eq!(BlockId::Stone.is_opaque(), true);
        assert_eq!(BlockId::Glass.is_solid(), true);
        assert_eq!(BlockId::Glass.is_transparent(), true);
        assert_eq!(BlockId::Glass.is_opaque(), false);
        assert_eq!(BlockId::Torch.light_emission(), 14);
        assert_eq!(BlockId::Obsidian.hardness(), 50.0);
        assert_eq!(BlockId::Obsidian.blast_resistance(), 1200.0);
        assert_eq!(BlockId::Bedrock.hardness(), -1.0);

        // Wood variants
        assert_eq!(BlockId::OakLog.properties().sound_family, SoundFamily::Wood);
        assert_eq!(BlockId::BirchPlanks.properties().sound_family, SoundFamily::Wood);
        assert_eq!(BlockId::SpruceLeaves.is_transparent(), true);
        assert_eq!(BlockId::DarkOakLog.hardness(), 2.0);

        // Ores
        assert_eq!(BlockId::DiamondOre.properties().min_harvest_tier, HarvestTier::Iron);
        assert_eq!(BlockId::CoalOre.properties().min_harvest_tier, HarvestTier::Wood);

        // Nether tests
        assert_eq!(BlockId::Netherrack.hardness(), 0.4);
        assert_eq!(BlockId::Glowstone.light_emission(), 15);
        assert_eq!(BlockId::CryingObsidian.light_emission(), 10);
        assert_eq!(BlockId::CryingObsidian.hardness(), 50.0);
        assert_eq!(BlockId::SoulSand.properties().sound_family, SoundFamily::SoulSand);
        assert_eq!(BlockId::Basalt.properties().sound_family, SoundFamily::Basalt);

        // The End tests
        assert_eq!(BlockId::EndStone.hardness(), 3.0);
        assert_eq!(BlockId::EndStone.blast_resistance(), 9.0);
        assert_eq!(BlockId::EndRod.light_emission(), 14);
        assert_eq!(BlockId::EndRod.is_transparent(), true);
        assert_eq!(BlockId::EndPortalFrame.hardness(), -1.0);
        assert_eq!(BlockId::EndPortalFrame.light_emission(), 1);
    }

    #[test]
    fn test_non_cubic_bounding_boxes() {
        let pos = IVec3::new(10, 5, 20);

        // Slab: half-height 0.5m
        let slab_boxes = BlockId::OakSlab.collision_boxes(pos);
        assert_eq!(slab_boxes.len(), 1);
        assert_eq!(slab_boxes[0].min, Vec3::new(10.0, 5.0, 20.0));
        assert_eq!(slab_boxes[0].max, Vec3::new(11.0, 5.5, 21.0));

        // Stairs: 2 compound AABBs (base slab + upper step)
        let stair_boxes = BlockId::OakStairs.collision_boxes(pos);
        assert_eq!(stair_boxes.len(), 2);
        // Base slab
        assert_eq!(stair_boxes[0].min, Vec3::new(10.0, 5.0, 20.0));
        assert_eq!(stair_boxes[0].max, Vec3::new(11.0, 5.5, 21.0));
        // Step
        assert_eq!(stair_boxes[1].min, Vec3::new(10.0, 5.5, 20.0));
        assert_eq!(stair_boxes[1].max, Vec3::new(11.0, 6.0, 20.5));

        // End Rod: thin 0.25m x 0.25m column centered
        let rod_boxes = BlockId::EndRod.collision_boxes(pos);
        assert_eq!(rod_boxes.len(), 1);
        assert_eq!(rod_boxes[0].min, Vec3::new(10.375, 5.0, 20.375));
        assert_eq!(rod_boxes[0].max, Vec3::new(10.625, 6.0, 20.625));

        // Torch: no solid collision, but selection box exists
        assert!(BlockId::Torch.collision_boxes(pos).is_empty());
        let torch_sel = BlockId::Torch.selection_boxes(pos);
        assert_eq!(torch_sel.len(), 1);
        assert_eq!(torch_sel[0].min, Vec3::new(10.375, 5.0, 20.375));
        assert_eq!(torch_sel[0].max, Vec3::new(10.625, 5.625, 20.625));
    }

    #[test]
    fn test_vanilla_name_roundtrip() {
        let blocks = [
            BlockId::Stone,
            BlockId::OakLog,
            BlockId::Glass,
            BlockId::Obsidian,
            BlockId::DiamondOre,
            BlockId::OakSlab,
            BlockId::OakStairs,
            BlockId::Netherrack,
            BlockId::Glowstone,
            BlockId::SoulSand,
            BlockId::EndStone,
            BlockId::EndRod,
            BlockId::PurpurStairs,
            BlockId::EndPortalFrame,
            BlockId::NetherPortal,
            BlockId::EndPortal,
            BlockId::DragonHead,
        ];

        for b in blocks {
            let name = b.vanilla_name();
            let parsed = BlockId::from_vanilla_name(name);
            assert_eq!(parsed, Some(b), "Roundtrip failed for {:?}", b);
        }
    }

    #[test]
    fn test_slab_raycast_hit_and_miss() {
        let pos = IVec3::new(5, 0, 5);
        let slab_boxes = BlockId::OakSlab.selection_boxes(pos);
        assert_eq!(slab_boxes.len(), 1);
        let slab_bb = slab_boxes[0];

        // Ray 1 aimed at lower half (y = 0.25): should hit
        let ray1_origin = Vec3::new(2.0, 0.25, 5.5);
        let ray1_dir = Vec3::new(1.0, 0.0, 0.0);
        let hit1 = slab_bb.ray_intersection(ray1_origin, ray1_dir, 10.0);
        assert!(hit1.is_some());
        let (dist1, face1, pt1) = hit1.unwrap();
        assert!((dist1 - 3.0).abs() < 1e-4);
        assert_eq!(face1, crate::render::atlas::Direction::West);
        assert_eq!(pt1, Vec3::new(5.0, 0.25, 5.5));

        // Ray 2 aimed at upper half (y = 0.75): passes over bottom slab without hitting
        let ray2_origin = Vec3::new(2.0, 0.75, 5.5);
        let ray2_dir = Vec3::new(1.0, 0.0, 0.0);
        let hit2 = slab_bb.ray_intersection(ray2_origin, ray2_dir, 10.0);
        assert!(hit2.is_none(), "Ray through empty upper half should not hit bottom slab");
    }

    #[test]
    fn test_end_rod_raycast_hit_and_miss() {
        let pos = IVec3::new(0, 0, 0);
        let rod_boxes = BlockId::EndRod.selection_boxes(pos);
        let rod_bb = rod_boxes[0];

        // Ray directly into center column (x = 0.5, z = 0.5): hits
        let hit_center = rod_bb.ray_intersection(
            Vec3::new(0.5, 0.5, -2.0),
            Vec3::new(0.0, 0.0, 1.0),
            10.0,
        );
        assert!(hit_center.is_some());
        let (dist, face, _) = hit_center.unwrap();
        assert!((dist - 2.375).abs() < 1e-4);
        assert_eq!(face, crate::render::atlas::Direction::North);

        // Ray passing to the side (x = 0.1, z = 0.5): misses rod
        let hit_side = rod_bb.ray_intersection(
            Vec3::new(0.1, 0.5, -2.0),
            Vec3::new(0.0, 0.0, 1.0),
            10.0,
        );
        assert!(hit_side.is_none());
    }
}
