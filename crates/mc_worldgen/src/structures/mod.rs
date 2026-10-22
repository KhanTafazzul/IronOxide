//! Structure grid placement, concentric rings, and procedural structure generators.

pub mod bastion;
pub mod buried_treasure;
pub mod jigsaw;
pub mod locator;
pub mod mineshaft;
pub mod nether_fortress;
pub mod ocean_monument;
pub mod placement_salts;
pub mod standalone;
pub mod template_pool;

pub use bastion::{BastionRemnantGenerator, BastionType};
pub use buried_treasure::BuriedTreasureGenerator;
pub use jigsaw::{
    AncientCityGenerator, PillagerOutpostGenerator, SculkShriekerState,
    ANCIENT_CITY_CHEST_LOOT, PILLAGER_OUTPOST_CHEST_LOOT,
};
pub use locator::{
    ChunkPos, ConcentricRingsPlacement, LegacyRandom, RandomSpreadPlacement, SpreadType,
    StructureLocator, StructureRoot, StructureType,
};
pub use mineshaft::{MineshaftGenerator, MineshaftType};
pub use nether_fortress::NetherFortressGenerator;
pub use ocean_monument::OceanMonumentGenerator;
pub use placement_salts::{
    ANCIENT_CITY, DESERT_PYRAMID, JUNGLE_PYRAMID, PILLAGER_OUTPOST, SWAMP_HUT,
    StructurePlacementDef,
};
pub use standalone::{
    DesertPyramidGenerator, JungleTempleGenerator, MobSpawnBoundingBox, SwampHutGenerator,
    DESERT_PYRAMID_ARCHAEOLOGY_LOOT, DESERT_PYRAMID_CHEST_LOOT, JUNGLE_TEMPLE_CHEST_LOOT,
    JUNGLE_TEMPLE_DISPENSER_LOOT,
};
pub use template_pool::{
    JigsawStructureAssembler, PlacedJigsawPiece, PoolElement, PoolProjection,
    StructureTemplatePool, TemplatePoolEntry, TemplatePoolRegistry, WeightedAliasTable,
};
