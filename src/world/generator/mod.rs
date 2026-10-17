pub mod nether;
pub mod the_end;

pub use nether::{
    hash_coords, NetherBiome, NetherGenerator, NetherGeneratorConfig, NETHER_LAVA_LEVEL,
    NETHER_MAX_Y, NETHER_MIN_Y, NETHER_SECTIONS_COUNT,
};

pub use the_end::{
    EndBiome, EndGeneratorConfig, TheEndGenerator, CENTRAL_ISLAND_RADIUS, THE_END_MAX_Y,
    THE_END_MIN_Y, THE_END_SECTIONS_COUNT, VOID_GAP_MAX_RADIUS, VOID_GAP_MIN_RADIUS,
};
