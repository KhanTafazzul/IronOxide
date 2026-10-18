//! Structure grid placement, concentric rings, and structure locator.

pub mod locator;

pub use locator::{
    ChunkPos, ConcentricRingsPlacement, LegacyRandom, RandomSpreadPlacement, SpreadType,
    StructureLocator, StructureRoot, StructureType,
};
