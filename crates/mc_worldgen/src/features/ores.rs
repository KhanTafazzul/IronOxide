//! Full Ore Generation & Height Trapezoids Engine.
//! REA Reference: net.minecraft.world.level.levelgen.feature.OreFeature (dsp.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.configurations.OreConfiguration (dug.class)
//! REA Reference: net.minecraft.world.level.levelgen.placement.HeightRangePlacement (dyf.class)
//! REA Reference: net.minecraft.world.level.levelgen.heightproviders.TrapezoidHeightProvider (dxp.class)
//! REA Reference: net.minecraft.world.level.levelgen.heightproviders.UniformHeightProvider (dxq.class)
//! REA Reference: net.minecraft.data.worldgen.features.OreFeatures (qm.class)
//! REA Reference: net.minecraft.data.worldgen.placement.OrePlacements (qx.class)

use crate::biomes::{BiomeId, SurfaceBlock};
use crate::structures::LegacyRandom;

/// Trait abstracting coordinate-based block read/write operations for ore placement.
pub trait OreBlockAccess {
    /// Retrieves the block at world coordinates `(x, y, z)`.
    fn get_block(&self, x: i32, y: i32, z: i32) -> SurfaceBlock;
    /// Sets the block at world coordinates `(x, y, z)`.
    fn set_block(&mut self, x: i32, y: i32, z: i32, block: SurfaceBlock);

    /// Checks if any of the 6 cardinal adjacent neighbors is an Air block.
    /// REA Reference: `dsp.a(Function, BlockPos)` -> `isAdjacentToAir`
    fn is_adjacent_to_air(&self, x: i32, y: i32, z: i32) -> bool {
        const OFFSETS: [(i32, i32, i32); 6] = [
            (-1, 0, 0),
            (1, 0, 0),
            (0, -1, 0),
            (0, 1, 0),
            (0, 0, -1),
            (0, 0, 1),
        ];
        for (dx, dy, dz) in OFFSETS {
            if self.get_block(x + dx, y + dy, z + dz) == SurfaceBlock::Air {
                return true;
            }
        }
        false
    }
}

/// In-memory 3D block buffer useful for chunk-scale ore generation and testing.
#[derive(Clone, Debug)]
pub struct ChunkOreBuffer {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub size_x: usize,
    pub size_y: usize,
    pub size_z: usize,
    pub blocks: Vec<SurfaceBlock>,
}

impl ChunkOreBuffer {
    /// Creates a new buffer initialized to the given default block.
    pub fn new(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        size_x: usize,
        size_y: usize,
        size_z: usize,
        default_block: SurfaceBlock,
    ) -> Self {
        let total = size_x * size_y * size_z;
        Self {
            min_x,
            min_y,
            min_z,
            size_x,
            size_y,
            size_z,
            blocks: vec![default_block; total],
        }
    }

    #[inline]
    fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        let lx = x - self.min_x;
        let ly = y - self.min_y;
        let lz = z - self.min_z;
        if lx >= 0
            && (lx as usize) < self.size_x
            && ly >= 0
            && (ly as usize) < self.size_y
            && lz >= 0
            && (lz as usize) < self.size_z
        {
            Some(((ly as usize) * self.size_z + (lz as usize)) * self.size_x + (lx as usize))
        } else {
            None
        }
    }

    #[inline]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> SurfaceBlock {
        if let Some(idx) = self.index(x, y, z) {
            self.blocks[idx]
        } else {
            SurfaceBlock::Air
        }
    }

    #[inline]
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: SurfaceBlock) {
        if let Some(idx) = self.index(x, y, z) {
            self.blocks[idx] = block;
        }
    }
}

impl OreBlockAccess for ChunkOreBuffer {
    fn get_block(&self, x: i32, y: i32, z: i32) -> SurfaceBlock {
        if let Some(idx) = self.index(x, y, z) {
            self.blocks[idx]
        } else {
            SurfaceBlock::Air
        }
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, block: SurfaceBlock) {
        if let Some(idx) = self.index(x, y, z) {
            self.blocks[idx] = block;
        }
    }
}

/// Ore type classification with corresponding stone and deepslate variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OreType {
    Coal,
    Iron,
    Copper,
    Gold,
    Redstone,
    Diamond,
    Emerald,
    Lapis,
}

impl OreType {
    /// Returns the ore variant embedded in standard Stone / rock layers.
    #[inline]
    pub fn stone_variant(&self) -> SurfaceBlock {
        match self {
            OreType::Coal => SurfaceBlock::CoalOre,
            OreType::Iron => SurfaceBlock::IronOre,
            OreType::Copper => SurfaceBlock::CopperOre,
            OreType::Gold => SurfaceBlock::GoldOre,
            OreType::Redstone => SurfaceBlock::RedstoneOre,
            OreType::Diamond => SurfaceBlock::DiamondOre,
            OreType::Emerald => SurfaceBlock::EmeraldOre,
            OreType::Lapis => SurfaceBlock::LapisOre,
        }
    }

    /// Returns the ore variant embedded in Deepslate layers (typically Y <= 0).
    #[inline]
    pub fn deepslate_variant(&self) -> SurfaceBlock {
        match self {
            OreType::Coal => SurfaceBlock::DeepslateCoalOre,
            OreType::Iron => SurfaceBlock::DeepslateIronOre,
            OreType::Copper => SurfaceBlock::DeepslateCopperOre,
            OreType::Gold => SurfaceBlock::DeepslateGoldOre,
            OreType::Redstone => SurfaceBlock::DeepslateRedstoneOre,
            OreType::Diamond => SurfaceBlock::DeepslateDiamondOre,
            OreType::Emerald => SurfaceBlock::DeepslateEmeraldOre,
            OreType::Lapis => SurfaceBlock::DeepslateLapisOre,
        }
    }

    /// Returns the canonical registry name of this ore.
    pub fn name(&self) -> &'static str {
        match self {
            OreType::Coal => "coal",
            OreType::Iron => "iron",
            OreType::Copper => "copper",
            OreType::Gold => "gold",
            OreType::Redstone => "redstone",
            OreType::Diamond => "diamond",
            OreType::Emerald => "emerald",
            OreType::Lapis => "lapis",
        }
    }
}

/// Ellipsoid blob placer matching vanilla `OreFeature` (`dsp.class`).
pub struct OreBlobPlacer;

impl OreBlobPlacer {
    /// Places an ellipsoid ore vein cluster centered near `origin`.
    ///
    /// REA Reference: `dsp.a(FeaturePlaceContext)` & `dsp.a(cuk, auv, dug, ...)`
    /// Returns the number of ore blocks successfully substituted.
    pub fn place_blob(
        world: &mut impl OreBlockAccess,
        origin: (i32, i32, i32),
        size: usize,
        ore_type: OreType,
        rng: &mut LegacyRandom,
    ) -> usize {
        Self::place_blob_with_air_exposure(world, origin, size, ore_type, 0.0, rng)
    }

    /// Places an ellipsoid ore vein cluster with configurable air exposure discard probability.
    ///
    /// REA Reference: `dsp.class` lines 15-38:
    /// If block is adjacent to air, it is discarded with probability `discard_chance_on_air_exposure`.
    pub fn place_blob_with_air_exposure(
        world: &mut impl OreBlockAccess,
        origin: (i32, i32, i32),
        size: usize,
        ore_type: OreType,
        discard_chance_on_air_exposure: f32,
        rng: &mut LegacyRandom,
    ) -> usize {
        if size == 0 {
            return 0;
        }

        let ox = origin.0 as f64;
        let oy = origin.1 as f64;
        let oz = origin.2 as f64;

        // REA Reference: dsp.class lines 25-138:
        // float angle = random.nextFloat() * (float)Math.PI;
        // float sizeDiv8 = (float)config.size / 8.0f;
        let angle = rng.next_float() * std::f32::consts::PI;
        let size_div_8 = (size as f32) / 8.0;

        let x1 = ox + (angle.sin() * size_div_8) as f64;
        let x2 = ox - (angle.sin() * size_div_8) as f64;
        let z1 = oz + (angle.cos() * size_div_8) as f64;
        let z2 = oz - (angle.cos() * size_div_8) as f64;

        let y1 = oy + (rng.next_int_bounded(3) as f64) - 2.0;
        let y2 = oy + (rng.next_int_bounded(3) as f64) - 2.0;

        // Collect ellipsoid spheres along the line segment
        let mut spheres = Vec::with_capacity(size);
        for step in 0..size {
            let t = (step as f64) / (size as f64);
            let cx = x1 + (x2 - x1) * t;
            let cy = y1 + (y2 - y1) * t;
            let cz = z1 + (z2 - z1) * t;

            // REA Reference: dsp.class lines 98-134:
            // double r = random.nextDouble() * (double)config.size / 16.0d;
            // double radius = ((Math.sin(PI * t) + 1.0) * r + 1.0) / 2.0;
            let r = rng.next_double() * (size as f64) / 16.0;
            let radius = (((std::f64::consts::PI * t).sin() + 1.0) * r + 1.0) / 2.0;
            spheres.push((cx, cy, cz, radius));
        }

        let mut placed_count = 0;

        // For each sphere, evaluate the ellipsoid volume in the local bounding box
        for (cx, cy, cz, radius) in spheres {
            if radius <= 0.0 {
                continue;
            }

            let min_bx = (cx - radius).floor() as i32;
            let max_bx = (cx + radius).floor() as i32;
            let min_by = (cy - radius).floor() as i32;
            let max_by = (cy + radius).floor() as i32;
            let min_bz = (cz - radius).floor() as i32;
            let max_bz = (cz + radius).floor() as i32;

            for bx in min_bx..=max_bx {
                let dx = ((bx as f64) + 0.5 - cx) / radius;
                let dx2 = dx * dx;
                if dx2 >= 1.0 {
                    continue;
                }

                for by in min_by..=max_by {
                    let dy = ((by as f64) + 0.5 - cy) / radius;
                    let dy2 = dy * dy;
                    if dx2 + dy2 >= 1.0 {
                        continue;
                    }

                    for bz in min_bz..=max_bz {
                        let dz = ((bz as f64) + 0.5 - cz) / radius;
                        let dist_sq = dx2 + dy2 + dz * dz;

                        // Ellipsoid interior test
                        if dist_sq < 1.0 {
                            // Reduced air exposure rule: if exposed to air, discard with chance
                            if discard_chance_on_air_exposure > 0.0
                                && world.is_adjacent_to_air(bx, by, bz)
                            {
                                if discard_chance_on_air_exposure >= 1.0
                                    || rng.next_float() < discard_chance_on_air_exposure
                                {
                                    continue;
                                }
                            }

                            let current_block = world.get_block(bx, by, bz);

                            // Substitute rock with corresponding ore variant
                            match current_block {
                                SurfaceBlock::Stone
                                | SurfaceBlock::Sandstone
                                | SurfaceBlock::RedSandstone
                                | SurfaceBlock::Terracotta
                                | SurfaceBlock::Gravel => {
                                    if by <= 0 {
                                        world.set_block(bx, by, bz, ore_type.deepslate_variant());
                                    } else {
                                        world.set_block(bx, by, bz, ore_type.stone_variant());
                                    }
                                    placed_count += 1;
                                }
                                SurfaceBlock::Deepslate => {
                                    world.set_block(bx, by, bz, ore_type.deepslate_variant());
                                    placed_count += 1;
                                }
                                // Bedrock, Air, Water, Lava, and existing ores are strictly preserved
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        placed_count
    }
}

/// Height distribution curves controlling ore elevation placement in the world.
#[derive(Clone, Debug, PartialEq)]
pub enum HeightDistribution {
    /// Uniform distribution between `min_inclusive` and `max_inclusive`.
    /// REA Reference: `dxq.class` (`UniformHeightProvider`)
    Uniform {
        min_inclusive: i32,
        max_inclusive: i32,
    },
    /// Trapezoid distribution between `min_inclusive` and `max_inclusive` with plateau width.
    /// When `plateau == 0`, forms an exact symmetric triangle with peak at `(min + max) / 2`.
    /// REA Reference: `dxp.class` (`TrapezoidHeightProvider`)
    Trapezoid {
        min_inclusive: i32,
        max_inclusive: i32,
        plateau: i32,
    },
    /// Biased to bottom distribution with higher probability near `min_inclusive`.
    /// REA Reference: `dxl.class` (`BiasedToBottomHeightProvider`)
    BiasedToBottom {
        min_inclusive: i32,
        max_inclusive: i32,
        bias: i32,
    },
}

impl HeightDistribution {
    /// Creates a uniform height distribution.
    #[inline]
    pub const fn uniform(min: i32, max: i32) -> Self {
        Self::Uniform {
            min_inclusive: min,
            max_inclusive: max,
        }
    }

    /// Creates a symmetric triangular distribution with peak at `(min + max) / 2`.
    #[inline]
    pub const fn triangular(min: i32, max: i32) -> Self {
        Self::Trapezoid {
            min_inclusive: min,
            max_inclusive: max,
            plateau: 0,
        }
    }

    /// Creates a trapezoid distribution with a flat plateau at the peak.
    #[inline]
    pub const fn trapezoid(min: i32, max: i32, plateau: i32) -> Self {
        Self::Trapezoid {
            min_inclusive: min,
            max_inclusive: max,
            plateau,
        }
    }

    /// Samples a Y coordinate from this distribution using the provided Java LCG RNG.
    /// REA Reference: `dxp.a(RandomSource, LevelHeightAccessor)`
    pub fn sample(&self, rng: &mut LegacyRandom) -> i32 {
        match *self {
            HeightDistribution::Uniform {
                min_inclusive,
                max_inclusive,
            } => {
                if min_inclusive >= max_inclusive {
                    return min_inclusive;
                }
                let range = max_inclusive - min_inclusive + 1;
                min_inclusive + rng.next_int_bounded(range)
            }
            HeightDistribution::Trapezoid {
                min_inclusive,
                max_inclusive,
                plateau,
            } => {
                if min_inclusive >= max_inclusive {
                    return min_inclusive;
                }
                let range = max_inclusive - min_inclusive;
                if plateau >= range {
                    return min_inclusive + rng.next_int_bounded(range + 1);
                }
                // REA Reference: dxp.class lines 65-100:
                // int diff = (range - plateau) / 2;
                // int k = range - diff;
                // return min + random.nextInt(k + 1) + random.nextInt(diff + 1);
                let diff = (range - plateau) / 2;
                let k = range - diff;
                min_inclusive + rng.next_int_bounded(k + 1) + rng.next_int_bounded(diff + 1)
            }
            HeightDistribution::BiasedToBottom {
                min_inclusive,
                max_inclusive,
                bias,
            } => {
                if min_inclusive >= max_inclusive {
                    return min_inclusive;
                }
                let range = max_inclusive - min_inclusive;
                let bound = (range + 1 - bias).max(1);
                let r1 = rng.next_int_bounded(bound);
                let r2 = rng.next_int_bounded(r1 + bias + 1);
                min_inclusive + r2
            }
        }
    }

    /// Computes the exact theoretical probability density at elevation `y`.
    /// Useful for verifying distribution peaks, plateaus, and boundaries.
    pub fn probability_density(&self, y: i32) -> f64 {
        match *self {
            HeightDistribution::Uniform {
                min_inclusive,
                max_inclusive,
            } => {
                if y < min_inclusive || y > max_inclusive {
                    0.0
                } else {
                    1.0 / ((max_inclusive - min_inclusive + 1) as f64)
                }
            }
            HeightDistribution::Trapezoid {
                min_inclusive,
                max_inclusive,
                plateau,
            } => {
                if y < min_inclusive || y > max_inclusive {
                    return 0.0;
                }
                let range = (max_inclusive - min_inclusive) as f64;
                if plateau as f64 >= range {
                    return 1.0 / (range + 1.0);
                }
                let diff = ((range - plateau as f64) / 2.0).floor();
                let k = range - diff;
                let offset = (y - min_inclusive) as f64;

                // Discrete convolution of U(0, k) and U(0, diff)
                let num_ways = (0..=(k as i32))
                    .filter(|&i| {
                        let j = (offset as i32) - i;
                        j >= 0 && j <= (diff as i32)
                    })
                    .count();

                let total_outcomes = (k + 1.0) * (diff + 1.0);
                (num_ways as f64) / total_outcomes
            }
            HeightDistribution::BiasedToBottom {
                min_inclusive,
                max_inclusive,
                ..
            } => {
                if y < min_inclusive || y > max_inclusive {
                    0.0
                } else {
                    let range = (max_inclusive - min_inclusive) as f64;
                    let norm = (max_inclusive - y) as f64 / range;
                    norm / (range / 2.0)
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Canonical Vanilla Overworld Ore Distribution Presets
// REA Reference: net.minecraft.data.worldgen.placement.OrePlacements (qx.class)
// ----------------------------------------------------------------------------

/// Coal Upper: Uniform Y = 136 to 256.
/// REA Reference: `qx.r` -> `dyf.a(136, 256)`
#[inline]
pub fn coal_upper_distribution() -> HeightDistribution {
    HeightDistribution::uniform(136, 256)
}

/// Coal Lower: Triangular distribution Y = 0 to 192 with peak at Y = 96.
/// REA Reference: `qx.s` -> `dyf.b(0, 192)`
#[inline]
pub fn coal_lower_distribution() -> HeightDistribution {
    HeightDistribution::triangular(0, 192)
}

/// Iron Upper: Triangular distribution Y = 80 to 384 with peak at Y = 232 / 256 (peaks in high mountains).
/// REA Reference: `qx.t` -> `dyf.b(80, 384)`
#[inline]
pub fn iron_upper_distribution() -> HeightDistribution {
    HeightDistribution::triangular(80, 384)
}

/// Iron Middle: Triangular distribution Y = -24 to 56 with peak at Y = 16.
/// REA Reference: `qx.u` -> `dyf.b(-24, 56)`
#[inline]
pub fn iron_middle_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-24, 56)
}

/// Iron Small: Uniform distribution Y = -64 to 72 (or triangular centered in low terrain).
/// REA Reference: `qx.v` -> `dyf.a(bottom, 72)`
#[inline]
pub fn iron_small_distribution() -> HeightDistribution {
    HeightDistribution::uniform(-64, 72)
}

/// Copper Regular: Triangular distribution Y = -16 to 112 with peak at Y = 48 (size 10, count 16).
/// REA Reference: `qx.L` -> `dyf.b(-16, 112)`
#[inline]
pub fn copper_regular_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-16, 112)
}

/// Copper Dripstone Caves (Large): Triangular distribution Y = -16 to 112 with peak at Y = 48 (doubled cluster size 20).
/// REA Reference: `qx.M` -> `dyf.b(-16, 112)`
#[inline]
pub fn copper_dripstone_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-16, 112)
}

/// Gold Regular: Triangular distribution Y = -64 to 32 with peak at Y = -16 (size 9).
/// REA Reference: `qx.x` / `qx.y` -> `dyf.b(-64, 32)`
#[inline]
pub fn gold_regular_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-64, 32)
}

/// Gold Extra (Badlands Biomes): High-abundance uniform distribution Y = 32 to 256.
/// REA Reference: `qx.w` -> `dyf.a(32, 256)`
#[inline]
pub fn gold_badlands_extra_distribution() -> HeightDistribution {
    HeightDistribution::uniform(32, 256)
}

/// Redstone Regular: Uniform distribution Y = -64 to 15.
/// REA Reference: `qx.z` -> `dyf.a(bottom, 15)`
#[inline]
pub fn redstone_regular_distribution() -> HeightDistribution {
    HeightDistribution::uniform(-64, 15)
}

/// Redstone Lower: Triangular centered at bedrock bottom Y = -64, scaling density down to bedrock.
/// REA Reference: `qx.A` -> `dyf.b(aboveBottom(-32), aboveBottom(32))`
#[inline]
pub fn redstone_lower_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-96, -32)
}

/// Lapis Lazuli Regular: Triangular distribution Y = -32 to 32 with peak at Y = 0 (size 7).
/// REA Reference: `qx.F` -> `dyf.b(-32, 32)`
#[inline]
pub fn lapis_regular_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-32, 32)
}

/// Lapis Lazuli Buried: Uniform distribution Y = -64 to 64 (size 7).
/// REA Reference: `qx.G` -> `dyf.a(bottom, 64)`
#[inline]
pub fn lapis_buried_distribution() -> HeightDistribution {
    HeightDistribution::uniform(-64, 64)
}

/// Diamond: Triangular distribution from Y = 16 down to Y = -64 with peak density at bedrock level (Y = -64).
/// REA Reference: `qx.B` / `qx.C` / `qx.D` -> `dyf.b(aboveBottom(-80), aboveBottom(80))`
#[inline]
pub fn diamond_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-144, 16)
}

/// Configuration defining an ore vein type, its cluster size, frequency, elevation curve, and air exposure discard rules.
#[derive(Clone, Debug, PartialEq)]
pub struct OreVeinConfig {
    /// Target ore material.
    pub ore_type: OreType,
    /// Number of ellipsoid interpolation steps determining cluster size.
    pub size: usize,
    /// Number of vein generation attempts per 16x16 chunk column.
    pub count_per_chunk: usize,
    /// Elevation distribution curve.
    pub distribution: HeightDistribution,
    /// If true, only generates when the chunk column belongs to a Badlands / Mesa biome variant.
    pub badlands_only: bool,
    /// If > 0.0, probability of discarding an ore block when adjacent to Air (reduced air exposure).
    pub discard_chance_on_air_exposure: f32,
}

impl OreVeinConfig {
    pub const fn new(
        ore_type: OreType,
        size: usize,
        count_per_chunk: usize,
        distribution: HeightDistribution,
    ) -> Self {
        Self {
            ore_type,
            size,
            count_per_chunk,
            distribution,
            badlands_only: false,
            discard_chance_on_air_exposure: 0.0,
        }
    }

    pub const fn with_air_discard(
        ore_type: OreType,
        size: usize,
        count_per_chunk: usize,
        distribution: HeightDistribution,
        discard_chance_on_air_exposure: f32,
    ) -> Self {
        Self {
            ore_type,
            size,
            count_per_chunk,
            distribution,
            badlands_only: false,
            discard_chance_on_air_exposure,
        }
    }

    pub const fn badlands_extra(
        ore_type: OreType,
        size: usize,
        count_per_chunk: usize,
        distribution: HeightDistribution,
    ) -> Self {
        Self {
            ore_type,
            size,
            count_per_chunk,
            distribution,
            badlands_only: true,
            discard_chance_on_air_exposure: 0.0,
        }
    }
}

/// Overworld ore generator orchestrating all standard vanilla ore placements.
#[derive(Clone, Debug)]
pub struct OreGenerator {
    pub configs: Vec<OreVeinConfig>,
}

impl Default for OreGenerator {
    fn default() -> Self {
        Self::vanilla_overworld()
    }
}

impl OreGenerator {
    /// Builds the standard vanilla Overworld ore configuration list matching Minecraft 1.20.4 byte-for-byte.
    /// REA Reference: `qm.class` & `qx.class`
    pub fn vanilla_overworld() -> Self {
        let configs = vec![
            // Coal Upper: size 17, count 30, Uniform 136 to 256
            OreVeinConfig::new(OreType::Coal, 17, 30, coal_upper_distribution()),
            // Coal Lower: size 17, count 20, Triangular 0 to 192 (peak at Y=96)
            OreVeinConfig::new(OreType::Coal, 17, 20, coal_lower_distribution()),

            // Iron Upper: size 9, count 90, Triangular 80 to 384 (peak at Y=232 / 256 in high peaks)
            OreVeinConfig::new(OreType::Iron, 9, 90, iron_upper_distribution()),
            // Iron Middle: size 9, count 10, Triangular -24 to 56 (peak at Y=16)
            OreVeinConfig::new(OreType::Iron, 9, 10, iron_middle_distribution()),
            // Iron Small: size 4, count 10, Uniform -64 to 72
            OreVeinConfig::new(OreType::Iron, 4, 10, iron_small_distribution()),

            // Copper Regular: size 10, count 16, Triangular -16 to 112 (peak at Y=48)
            OreVeinConfig::new(OreType::Copper, 10, 16, copper_regular_distribution()),
            // Copper Large (Dripstone Caves): size 20, count 16, Triangular -16 to 112 (peak at Y=48)
            OreVeinConfig::new(OreType::Copper, 20, 16, copper_dripstone_distribution()),

            // Gold Regular: size 9, count 4, Triangular -64 to 32 (peak at Y=-16)
            OreVeinConfig::new(OreType::Gold, 9, 4, gold_regular_distribution()),
            // Gold Extra (Badlands / Mesa biomes): size 9, count 50, Uniform 32 to 256
            OreVeinConfig::badlands_extra(OreType::Gold, 9, 50, gold_badlands_extra_distribution()),

            // Redstone Regular: size 8, count 4, Uniform -64 to 15
            OreVeinConfig::new(OreType::Redstone, 8, 4, redstone_regular_distribution()),
            // Redstone Lower: size 8, count 8, Triangular toward bottom bedrock
            OreVeinConfig::new(OreType::Redstone, 8, 8, redstone_lower_distribution()),

            // Lapis Regular: size 7, count 2, Triangular -32 to 32 (peak at Y=0)
            OreVeinConfig::new(OreType::Lapis, 7, 2, lapis_regular_distribution()),
            // Lapis Buried: size 7, count 4, Uniform -64 to 64
            OreVeinConfig::new(OreType::Lapis, 7, 4, lapis_buried_distribution()),

            // Diamond Small: size 4, count 7, Triangular 16 down to -64, discard air exposure 0.5
            OreVeinConfig::with_air_discard(OreType::Diamond, 4, 7, diamond_distribution(), 0.5),
            // Diamond Medium: size 8, count 7, Triangular 16 down to -64, discard air exposure 0.5
            OreVeinConfig::with_air_discard(OreType::Diamond, 8, 7, diamond_distribution(), 0.5),
            // Diamond Large: size 12, count 1, Triangular 16 down to -64, discard air exposure 0.7
            OreVeinConfig::with_air_discard(OreType::Diamond, 12, 1, diamond_distribution(), 0.7),
            // Diamond Buried: size 8, count 4, Triangular 16 down to -64, discard air exposure 1.0 (never exposed)
            OreVeinConfig::with_air_discard(OreType::Diamond, 8, 4, diamond_distribution(), 1.0),
        ];

        Self { configs }
    }

    /// Evaluates whether the given biome is a Badlands / Mesa variant.
    #[inline]
    pub fn is_badlands(biome: BiomeId) -> bool {
        matches!(
            biome,
            BiomeId::Badlands | BiomeId::ErodedBadlands | BiomeId::WoodedBadlands
        )
    }

    /// Generates all configured ores within a 16x16 chunk column.
    pub fn generate_chunk(
        &self,
        world: &mut impl OreBlockAccess,
        chunk_x: i32,
        chunk_z: i32,
        biome: BiomeId,
        world_seed: i64,
    ) -> usize {
        let is_badlands = Self::is_badlands(biome);
        let mut total_ores = 0;

        for (feature_idx, config) in self.configs.iter().enumerate() {
            // Badlands extra gold generates strictly in Badlands biomes
            if config.badlands_only && !is_badlands {
                continue;
            }

            // Derive deterministic chunk feature seed matching vanilla decoration step
            let salt = (feature_idx as i32).wrapping_mul(314159).wrapping_add(10387313);
            let mut rng = LegacyRandom::from_chunk_seed(world_seed, chunk_x, chunk_z, salt);

            for _ in 0..config.count_per_chunk {
                // Sample random (X, Z) inside chunk bounds [0, 15]
                let local_x = rng.next_int_bounded(16);
                let local_z = rng.next_int_bounded(16);
                let world_x = chunk_x * 16 + local_x;
                let world_z = chunk_z * 16 + local_z;

                // Sample Y coordinate from ore distribution curve
                let world_y = config.distribution.sample(&mut rng);

                // Place ellipsoid blob cluster
                let placed = OreBlobPlacer::place_blob_with_air_exposure(
                    world,
                    (world_x, world_y, world_z),
                    config.size,
                    config.ore_type,
                    config.discard_chance_on_air_exposure,
                    &mut rng,
                );

                total_ores += placed;
            }
        }

        total_ores
    }
}
