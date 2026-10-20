//! Minecraft 1.21.4 Pale Garden Biome, Climate Definition, Atmospheric Effects, and Pale Oak Trees.
//! REA Reference: net.minecraft.world.level.biome.Biomes (PALE_GARDEN)
//! REA Reference: net.minecraft.world.level.levelgen.feature.TreeFeature (Pale Oak configuration)
//! REA Reference: net.minecraft.world.level.block.CreakingHeartBlock

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::seed_pipeline::climate::{ParameterPoint, ParameterRange, TargetPoint};
use crate::structures::LegacyRandom;

// ----------------------------------------------------------------------------
// 1. CLIMATE DEFINITION (6D PARAMETER POINT)
// ----------------------------------------------------------------------------

/// Vanilla 1.21.4 Pale Garden 6D climate point:
/// - Low vegetation (humidity range -4000..-1000)
/// - Moderate continentalness (1000..3000)
/// - Unique weirdness offset (3000..6000, offset = 500)
pub const PALE_GARDEN_CLIMATE: ParameterPoint = ParameterPoint {
    temperature: ParameterRange::new(-1000, 2000),      // Moderate temperate
    humidity: ParameterRange::new(-4000, -1000),        // Low vegetation
    continentalness: ParameterRange::new(1000, 3000),   // Moderate continentalness
    erosion: ParameterRange::new(-2000, 2000),          // Moderate erosion
    depth: ParameterRange::new(0, 0),                   // Surface layer
    weirdness: ParameterRange::new(3000, 6000),         // Unique weirdness band
    offset: 500,                                        // Explicit weirdness offset
};

/// Evaluates fitness of a target point against the Pale Garden climate point.
#[inline]
pub fn pale_garden_fitness(target: &TargetPoint) -> i64 {
    PALE_GARDEN_CLIMATE.fitness(target)
}

/// Checks whether a given 6D target point falls within the Pale Garden climate volume.
pub fn is_pale_garden_climate(target: &TargetPoint) -> bool {
    target.humidity >= -4000 && target.humidity <= -1000
        && target.continentalness >= 1000 && target.continentalness <= 3000
        && target.weirdness >= 3000 && target.weirdness <= 6000
}

// ----------------------------------------------------------------------------
// 2. ATMOSPHERIC EFFECTS & DESATURATED COLORMAP
// ----------------------------------------------------------------------------

/// Pale gray ambient fog color (#9E9EA0 -> [158, 158, 160])
pub const PALE_GARDEN_FOG_COLOR: [u8; 3] = [0x9E, 0x9E, 0xA0];

/// Desaturated ashen foliage color (#87857E -> [135, 133, 126])
pub const PALE_GARDEN_FOLIAGE_COLOR: [u8; 3] = [0x87, 0x85, 0x7E];

/// Desaturated pale gray grass color (#84877D -> [132, 135, 125])
pub const PALE_GARDEN_GRASS_COLOR: [u8; 3] = [0x84, 0x87, 0x7D];

/// Overcast desaturated sky color (#C0C0C0 -> [192, 192, 192])
pub const PALE_GARDEN_SKY_COLOR: [u8; 3] = [0xC0, 0xC0, 0xC0];

/// Water surface color (#768894 -> [118, 136, 148])
pub const PALE_GARDEN_WATER_COLOR: [u8; 3] = [0x76, 0x88, 0x94];

/// Water fog color (#556874 -> [85, 104, 116])
pub const PALE_GARDEN_WATER_FOG_COLOR: [u8; 3] = [0x55, 0x68, 0x74];

/// Atmospheric color profile container for the Pale Garden biome.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaleGardenAtmosphere {
    pub fog_color: [f32; 3],
    pub foliage_color: [f32; 3],
    pub grass_color: [f32; 3],
    pub sky_color: [f32; 3],
    pub water_color: [f32; 3],
    pub water_fog_color: [f32; 3],
    pub fog_start: f32,
    pub fog_end: f32,
}

impl Default for PaleGardenAtmosphere {
    fn default() -> Self {
        Self {
            fog_color: [0x9E as f32 / 255.0, 0x9E as f32 / 255.0, 0xA0 as f32 / 255.0],
            foliage_color: [0x87 as f32 / 255.0, 0x85 as f32 / 255.0, 0x7E as f32 / 255.0],
            grass_color: [0x84 as f32 / 255.0, 0x87 as f32 / 255.0, 0x7D as f32 / 255.0],
            sky_color: [0xC0 as f32 / 255.0, 0xC0 as f32 / 255.0, 0xC0 as f32 / 255.0],
            water_color: [0x76 as f32 / 255.0, 0x88 as f32 / 255.0, 0x94 as f32 / 255.0],
            water_fog_color: [0x55 as f32 / 255.0, 0x68 as f32 / 255.0, 0x74 as f32 / 255.0],
            fog_start: 0.85,
            fog_end: 1.0,
        }
    }
}

/// Forces desaturation onto standard RGB color values converting them to Pale Garden's
/// eerie grayscale atmospheric palette using Rec. 709 luminance coefficients.
#[inline]
pub fn desaturate_to_pale_garden(rgb: [u8; 3], factor: f32) -> [u8; 3] {
    let r = rgb[0] as f32;
    let g = rgb[1] as f32;
    let b = rgb[2] as f32;
    // Standard Rec. 709 grayscale luminance
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;

    let nr = r + (luma - r) * factor;
    let ng = g + (luma - g) * factor;
    let nb = b + (luma - b) * factor;

    [
        nr.clamp(0.0, 255.0).round() as u8,
        ng.clamp(0.0, 255.0).round() as u8,
        nb.clamp(0.0, 255.0).round() as u8,
    ]
}

// ----------------------------------------------------------------------------
// 3. PALE OAK PROCEDURAL TREE GENERATOR
// ----------------------------------------------------------------------------

/// Output statistics from generating a Pale Oak tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaleOakTreeResult {
    pub logs_placed: usize,
    pub leaves_placed: usize,
    pub moss_vines_placed: usize,
    pub creaking_heart_placed: bool,
}

/// Checks if a block is valid soil for a Pale Oak tree to grow on.
#[inline]
pub fn is_valid_pale_oak_soil(block: SurfaceBlock) -> bool {
    matches!(
        block,
        SurfaceBlock::GrassBlock
            | SurfaceBlock::Dirt
            | SurfaceBlock::Podzol
            | SurfaceBlock::CoarseDirt
            | SurfaceBlock::Mud
            | SurfaceBlock::PaleMossBlock
    )
}

/// Generates a vanilla 1.21.4 Pale Oak tree:
/// - 2x2 thick trunk of `PaleOakLog` (10 to 16 blocks tall).
/// - Wide flat sprawling canopies of `PaleOakLeaves` radiating horizontally.
/// - Hanging Pale Moss vines (`PaleHangingMoss`) underneath leaves hanging down 1 to 4 blocks.
/// - Optional placement of `CreakingHeart` embedded inside the trunk (~15% natural chance).
pub fn generate_pale_oak_tree(
    world: &mut dyn OreBlockAccess,
    origin: (i32, i32, i32),
    rng: &mut LegacyRandom,
    place_creaking_heart: bool,
) -> PaleOakTreeResult {
    let (ox, oy, oz) = origin;

    // Verify soil beneath all 4 trunk blocks
    for dx in 0..=1 {
        for dz in 0..=1 {
            let soil = world.get_block(ox + dx, oy - 1, oz + dz);
            if !is_valid_pale_oak_soil(soil) {
                return PaleOakTreeResult {
                    logs_placed: 0,
                    leaves_placed: 0,
                    moss_vines_placed: 0,
                    creaking_heart_placed: false,
                };
            }
        }
    }

    // Convert soil to dirt beneath the 2x2 trunk
    for dx in 0..=1 {
        for dz in 0..=1 {
            let soil = world.get_block(ox + dx, oy - 1, oz + dz);
            if soil == SurfaceBlock::GrassBlock || soil == SurfaceBlock::PaleMossBlock {
                world.set_block(ox + dx, oy - 1, oz + dz, SurfaceBlock::Dirt);
            }
        }
    }

    // Height of 2x2 trunk: 10 to 16 blocks (base 10 + rng(7))
    let height = 10 + rng.next_int_bounded(7);
    let mut logs_placed = 0;
    let mut creaking_heart_placed = false;

    // Determine Creaking Heart placement coordinate inside the 2x2 trunk
    // Creaking heart spawns naturally between Y+3 and Y+6 inside one of the 4 trunk columns,
    // encased with logs above and below.
    let creaking_y = if place_creaking_heart && height >= 8 {
        Some(oy + 3 + rng.next_int_bounded(3.min(height - 5).max(1)))
    } else {
        None
    };
    let (creaking_dx, creaking_dz) = if creaking_y.is_some() {
        (rng.next_int_bounded(2), rng.next_int_bounded(2))
    } else {
        (0, 0)
    };

    // 1. Place 2x2 trunk
    for dy in 0..height {
        let y = oy + dy;
        for dx in 0..=1 {
            for dz in 0..=1 {
                let x = ox + dx;
                let z = oz + dz;

                if Some(y) == creaking_y && dx == creaking_dx && dz == creaking_dz {
                    world.set_block(x, y, z, SurfaceBlock::CreakingHeart);
                    creaking_heart_placed = true;
                } else {
                    world.set_block(x, y, z, SurfaceBlock::PaleOakLog);
                    logs_placed += 1;
                }
            }
        }
    }

    // 2. Place sprawling horizontal branches near trunk top (Y = height - 3 to height)
    let branch_y = oy + height - 2;
    let branch_offsets = [
        (-1, 0), (2, 0), (0, -1), (0, 2),
        (-1, 1), (2, 1), (1, -1), (1, 2),
    ];
    for &(bx, bz) in &branch_offsets {
        if rng.next_int_bounded(4) != 0 {
            world.set_block(ox + bx, branch_y, oz + bz, SurfaceBlock::PaleOakLog);
            logs_placed += 1;
        }
    }

    // 3. Place flat sprawling canopy of PaleOakLeaves
    // Pale Oak canopies are characterized by broad flat horizontal disks with layered tiers.
    let mut leaves_placed = 0;
    let mut leaf_coords = Vec::new();

    let top_y = oy + height;
    // Tiers: -3 (broad lower disk, radius 4), -2 (mid disk, radius 3-4), -1 (upper mid, radius 2-3), 0 (top cap, radius 1-2)
    for dy in -3..=1 {
        let y = top_y + dy;
        let radius: i32 = match dy {
            -3 => 4,
            -2 => 4,
            -1 => 3,
            0 => 2,
            1 => 1,
            _ => 1,
        };

        let radius_sq = (radius * radius) as f64 + 0.5;

        for dx in -radius..=(radius + 1) {
            for dz in -radius..=(radius + 1) {
                // Distance from center of 2x2 trunk (ox + 0.5, oz + 0.5)
                let dist_x = (dx as f64) - 0.5;
                let dist_z = (dz as f64) - 0.5;
                let dist_sq = dist_x * dist_x + dist_z * dist_z;

                if dist_sq <= radius_sq {
                    let lx = ox + dx;
                    let lz = oz + dz;
                    let cur = world.get_block(lx, y, lz);
                    if cur == SurfaceBlock::Air || cur == SurfaceBlock::PaleHangingMoss {
                        world.set_block(lx, y, lz, SurfaceBlock::PaleOakLeaves);
                        leaves_placed += 1;
                        leaf_coords.push((lx, y, lz));
                    }
                }
            }
        }
    }

    // 4. Place hanging pale moss underneath leaves
    // Under ~25% of leaves with open air beneath, hang 1 to 4 blocks of PaleHangingMoss
    let mut moss_vines_placed = 0;
    for (lx, ly, lz) in leaf_coords {
        if rng.next_int_bounded(4) == 0 {
            let mut vine_y = ly - 1;
            let vine_length = 1 + rng.next_int_bounded(4);
            let mut len = 0;
            while len < vine_length && vine_y > oy {
                if world.get_block(lx, vine_y, lz) == SurfaceBlock::Air {
                    world.set_block(lx, vine_y, lz, SurfaceBlock::PaleHangingMoss);
                    moss_vines_placed += 1;
                    vine_y -= 1;
                    len += 1;
                } else {
                    break;
                }
            }
        }
    }

    PaleOakTreeResult {
        logs_placed,
        leaves_placed,
        moss_vines_placed,
        creaking_heart_placed,
    }
}
