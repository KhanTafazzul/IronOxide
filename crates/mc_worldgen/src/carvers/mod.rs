//! 3D Noise Cave Carvers and Canyons for Minecraft 1.20.4 Overworld generation.
//!
//! Features:
//! - **Cheese Caves**: Large open cavern bubbles using 3D cheese noise.
//! - **Spaghetti Caves**: Twisting tunnels formed by intersecting 3D noise channels.
//! - **Noodle Caves**: Narrow, claustrophobic passages using high-frequency noise.
//! - **Canyons / Ravines**: Steep gorges cutting through surface terrain.
//! - **Block Replacement Rules**: Solid terrain is carved into Air or Water/Lava, with Bedrock protected.

pub mod canyons;
pub mod config;
pub mod noise_caves;
pub mod world_carver;

pub use canyons::{CanyonNode, CanyonWorldCarver};
pub use config::{
    carved_fluid_or_air, is_carver_replaceable, CanyonCarverConfiguration, CarverConfiguration,
    CarvedState, BEDROCK_CEILING_Y, LAVA_LEVEL_Y, SEA_LEVEL,
};
pub use noise_caves::{
    CaveCarverRouter, CaveType, CheeseCaveSampler, NoodleCaveSampler, SpaghettiCaveSampler,
};
pub use world_carver::{carve_block, carve_chunk_slice, CarveResult};
