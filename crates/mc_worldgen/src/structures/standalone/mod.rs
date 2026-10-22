//! Standalone non-jigsaw structure generators.

pub mod desert_pyramid;
pub mod jungle_temple;
pub mod swamp_hut;

pub use desert_pyramid::{DesertPyramidGenerator, DESERT_PYRAMID_ARCHAEOLOGY_LOOT, DESERT_PYRAMID_CHEST_LOOT};
pub use jungle_temple::{JungleTempleGenerator, JUNGLE_TEMPLE_CHEST_LOOT, JUNGLE_TEMPLE_DISPENSER_LOOT};
pub use swamp_hut::{MobSpawnBoundingBox, SwampHutGenerator};
