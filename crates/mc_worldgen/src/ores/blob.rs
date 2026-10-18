//! Ellipsoid ore blob placer generating connected clusters.
//! REA Reference: net.minecraft.world.level.levelgen.feature.OreFeature (dsp.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.configurations.OreConfiguration (dug.class)

use crate::biomes::SurfaceBlock;
use crate::structures::LegacyRandom;

/// Trait abstracting coordinate-based block read/write operations for ore placement.
pub trait OreBlockAccess {
    /// Retrieves the block at world coordinates `(x, y, z)`.
    fn get_block(&self, x: i32, y: i32, z: i32) -> SurfaceBlock;
    /// Sets the block at world coordinates `(x, y, z)`.
    fn set_block(&mut self, x: i32, y: i32, z: i32, block: SurfaceBlock);
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
    pub fn new(min_x: i32, min_y: i32, min_z: i32, size_x: usize, size_y: usize, size_z: usize, default_block: SurfaceBlock) -> Self {
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
        if lx >= 0 && (lx as usize) < self.size_x
            && ly >= 0 && (ly as usize) < self.size_y
            && lz >= 0 && (lz as usize) < self.size_z
        {
            Some(((ly as usize) * self.size_z + (lz as usize)) * self.size_x + (lx as usize))
        } else {
            None
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
        // Each entry: (cx, cy, cz, radius)
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
                            let current_block = world.get_block(bx, by, bz);

                            // Substitute rock with corresponding ore variant
                            match current_block {
                                SurfaceBlock::Stone
                                | SurfaceBlock::Sandstone
                                | SurfaceBlock::RedSandstone
                                | SurfaceBlock::Terracotta
                                | SurfaceBlock::Gravel => {
                                    world.set_block(bx, by, bz, ore_type.stone_variant());
                                    placed_count += 1;
                                }
                                SurfaceBlock::Deepslate => {
                                    world.set_block(bx, by, bz, ore_type.deepslate_variant());
                                    placed_count += 1;
                                }
                                // Bedrock, Air, Water, and other non-rock blocks are strictly preserved
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
