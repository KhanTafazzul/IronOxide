//! Foliage placers implementing spherical blob clusters and stepped spruce cones.
//! REA Reference: net.minecraft.world.level.levelgen.feature.foliageplacers.BlobFoliagePlacer (dvf.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.foliageplacers.SpruceFoliagePlacer (dvq.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer (dvk.class)

use super::trunk::FoliageAttachment;
use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::LegacyRandom;

/// Blob foliage placer generating spherical clusters of leaves with corner decay.
/// REA Reference: `dvf.class` (`BlobFoliagePlacer`)
#[derive(Clone, Copy, Debug)]
pub struct BlobFoliagePlacer {
    pub radius: i32,
    pub height: i32,
}

impl Default for BlobFoliagePlacer {
    fn default() -> Self {
        Self { radius: 2, height: 3 }
    }
}

impl BlobFoliagePlacer {
    pub const fn new(radius: i32, height: i32) -> Self {
        Self { radius, height }
    }

    /// Places blob foliage around the provided attachment point.
    pub fn place_foliage(
        &self,
        world: &mut impl OreBlockAccess,
        attachment: &FoliageAttachment,
        leaf_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> usize {
        let center_x = attachment.x;
        let center_y = attachment.y;
        let center_z = attachment.z;
        let effective_radius = self.radius + attachment.radius_offset;

        let mut leaves_placed = 0;

        // Iterate vertically from top down
        // dy = 0 is canopy top, -1 is middle, -2 is bottom
        for dy in (-(self.height - 1)..=0).rev() {
            // Top layer has smaller radius (radius - 1, min 1)
            let layer_radius = if dy == 0 {
                (effective_radius - 1).max(1)
            } else if dy == -(self.height - 1) {
                effective_radius.max(1)
            } else {
                effective_radius
            };

            for dx in -layer_radius..=layer_radius {
                for dz in -layer_radius..=layer_radius {
                    let is_corner = dx.abs() == layer_radius && dz.abs() == layer_radius;

                    // REA Reference: dvf.class lines 130-155:
                    // Corner decay logic: 50% chance to drop corner blocks, or always on top layer
                    if is_corner {
                        if dy == 0 || rng.next_int_bounded(2) == 0 {
                            continue;
                        }
                    }

                    let px = center_x + dx;
                    let py = center_y + dy;
                    let pz = center_z + dz;

                    let current = world.get_block(px, py, pz);
                    if current == SurfaceBlock::Air || current == SurfaceBlock::SnowLayer {
                        world.set_block(px, py, pz, leaf_block);
                        leaves_placed += 1;
                    }
                }
            }
        }

        leaves_placed
    }
}

/// Spruce / Pine foliage placer generating stepped conical evergreen canopies.
/// REA Reference: `dvq.class` (`SpruceFoliagePlacer`)
#[derive(Clone, Copy, Debug)]
pub struct SpruceFoliagePlacer {
    pub base_radius: i32,
    pub height: i32,
}

impl Default for SpruceFoliagePlacer {
    fn default() -> Self {
        Self { base_radius: 2, height: 6 }
    }
}

impl SpruceFoliagePlacer {
    pub const fn new(base_radius: i32, height: i32) -> Self {
        Self { base_radius, height }
    }

    /// Places stepped cone foliage layers tapering toward the top tip.
    /// REA Reference: `dvq.a(ctv, dvk$b, auv, duu, ...)`
    pub fn place_foliage(
        &self,
        world: &mut impl OreBlockAccess,
        attachment: &FoliageAttachment,
        leaf_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> usize {
        let center_x = attachment.x;
        let center_y = attachment.y;
        let center_z = attachment.z;

        let mut leaves_placed = 0;

        // Stepped cone state machine matching dvq.class:
        // int radius = random.nextInt(2);
        // int maxRadius = 1;
        // int minRadius = 0;
        let mut radius = rng.next_int_bounded(2);
        let mut max_radius = 1;
        let mut min_radius = 0;

        for dy in (-(self.height)..=0).rev() {
            let cur_radius = radius + attachment.radius_offset;

            for dx in -cur_radius..=cur_radius {
                for dz in -cur_radius..=cur_radius {
                    // Outer corner decay for larger circles
                    if cur_radius > 0 && dx.abs() == cur_radius && dz.abs() == cur_radius {
                        if rng.next_int_bounded(2) == 0 {
                            continue;
                        }
                    }

                    let px = center_x + dx;
                    let py = center_y + dy;
                    let pz = center_z + dz;

                    let current = world.get_block(px, py, pz);
                    if current == SurfaceBlock::Air || current == SurfaceBlock::SnowLayer {
                        world.set_block(px, py, pz, leaf_block);
                        leaves_placed += 1;
                    }
                }
            }

            // Stepped radius progression matching dvq.class:
            if radius >= max_radius {
                radius = min_radius;
                min_radius = 1;
                max_radius = (max_radius + 1).min(self.base_radius + attachment.radius_offset);
            } else {
                radius += 1;
            }
        }

        leaves_placed
    }
}
