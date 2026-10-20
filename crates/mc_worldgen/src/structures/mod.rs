//! Structure grid placement, concentric rings, and procedural structure generators.

pub mod bastion;
pub mod buried_treasure;
pub mod locator;
pub mod mineshaft;
pub mod nether_fortress;
pub mod ocean_monument;
pub mod template_pool;

pub use bastion::{BastionRemnantGenerator, BastionType};
pub use buried_treasure::BuriedTreasureGenerator;
pub use locator::{
    ChunkPos, ConcentricRingsPlacement, LegacyRandom, RandomSpreadPlacement, SpreadType,
    StructureLocator, StructureRoot, StructureType,
};
pub use mineshaft::{MineshaftGenerator, MineshaftType};
pub use nether_fortress::NetherFortressGenerator;
pub use ocean_monument::OceanMonumentGenerator;
pub use template_pool::{
    JigsawStructureAssembler, PlacedJigsawPiece, PoolElement, PoolProjection,
    StructureTemplatePool, TemplatePoolEntry, TemplatePoolRegistry, WeightedAliasTable,
};
