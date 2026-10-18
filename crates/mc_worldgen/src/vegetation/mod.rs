//! Procedural tree, foliage, and ground cover vegetation engine.
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.StraightTrunkPlacer (dxb.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer (dwx.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.foliageplacers.BlobFoliagePlacer (dvf.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.foliageplacers.SpruceFoliagePlacer (dvq.class)

pub mod foliage;
pub mod ground_cover;
pub mod tree;
pub mod trunk;

pub use foliage::{BlobFoliagePlacer, SpruceFoliagePlacer};
pub use ground_cover::{GroundCoverConfig, GroundCoverPlacer};
pub use tree::{TreePlacer, TreeType};
pub use trunk::{BranchingMegaTrunkPlacer, FoliageAttachment, StraightTrunkPlacer};
