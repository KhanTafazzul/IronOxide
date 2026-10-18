//! Tree trunk placers implementing straight pillars and branching mega trunks.
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.StraightTrunkPlacer (dxb.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer (dwx.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.DarkOakTrunkPlacer (dww.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.MegaJungleTrunkPlacer (dxa.class)

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::LegacyRandom;

/// Foliage attachment point emitted by trunk placers where leaves should cluster.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoliageAttachment {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub radius_offset: i32,
    pub is_giant_trunk: bool,
}

impl FoliageAttachment {
    pub const fn new(x: i32, y: i32, z: i32, radius_offset: i32, is_giant_trunk: bool) -> Self {
        Self {
            x,
            y,
            z,
            radius_offset,
            is_giant_trunk,
        }
    }
}

/// Straight trunk placer generating a 1x1 vertical log pillar.
/// REA Reference: `dxb.class` (`StraightTrunkPlacer`)
#[derive(Clone, Copy, Debug)]
pub struct StraightTrunkPlacer {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
}

impl Default for StraightTrunkPlacer {
    fn default() -> Self {
        Self {
            base_height: 4,
            height_rand_a: 2,
            height_rand_b: 1,
        }
    }
}

impl StraightTrunkPlacer {
    pub const fn new(base_height: i32, height_rand_a: i32, height_rand_b: i32) -> Self {
        Self {
            base_height,
            height_rand_a,
            height_rand_b,
        }
    }

    /// Evaluates trunk height: base_height + rand(0..=a) + rand(0..=b)
    pub fn get_tree_height(&self, rng: &mut LegacyRandom) -> i32 {
        let mut h = self.base_height;
        if self.height_rand_a > 0 {
            h += rng.next_int_bounded(self.height_rand_a + 1);
        }
        if self.height_rand_b > 0 {
            h += rng.next_int_bounded(self.height_rand_b + 1);
        }
        h
    }

    /// Places a 1x1 vertical log column and returns the foliage attachment point at the summit.
    pub fn place_trunk(
        &self,
        world: &mut impl OreBlockAccess,
        origin: (i32, i32, i32),
        log_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> (Vec<FoliageAttachment>, usize) {
        let (x, y, z) = origin;
        let height = self.get_tree_height(rng);

        // Vanilla rule: ensure block beneath trunk is dirt
        world.set_block(x, y - 1, z, SurfaceBlock::Dirt);

        let mut logs_placed = 0;
        for dy in 0..height {
            let py = y + dy;
            let current = world.get_block(x, py, z);
            if current == SurfaceBlock::Air
                || current == SurfaceBlock::SnowLayer
                || current == SurfaceBlock::Water
                || current == SurfaceBlock::ShortGrass
                || current == SurfaceBlock::TallGrass
                || current == SurfaceBlock::Poppy
                || current == SurfaceBlock::Dandelion
                || is_leaves(current)
            {
                world.set_block(x, py, z, log_block);
                logs_placed += 1;
            }
        }

        let attachment = FoliageAttachment::new(x, y + height, z, 0, false);
        (vec![attachment], logs_placed)
    }
}

/// Branching and 2x2 Mega Trunk Placer for Dark Oak, Mega Jungle, and Giant Spruce.
/// REA Reference: `dww.class` (`DarkOakTrunkPlacer`) & `dxa.class` (`MegaJungleTrunkPlacer`)
#[derive(Clone, Copy, Debug)]
pub struct BranchingMegaTrunkPlacer {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
}

impl Default for BranchingMegaTrunkPlacer {
    fn default() -> Self {
        Self {
            base_height: 6,
            height_rand_a: 3,
            height_rand_b: 1,
        }
    }
}

impl BranchingMegaTrunkPlacer {
    pub const fn new(base_height: i32, height_rand_a: i32, height_rand_b: i32) -> Self {
        Self {
            base_height,
            height_rand_a,
            height_rand_b,
        }
    }

    pub fn get_tree_height(&self, rng: &mut LegacyRandom) -> i32 {
        let mut h = self.base_height;
        if self.height_rand_a > 0 {
            h += rng.next_int_bounded(self.height_rand_a + 1);
        }
        if self.height_rand_b > 0 {
            h += rng.next_int_bounded(self.height_rand_b + 1);
        }
        h
    }

    /// Places a 2x2 thick trunk pillar with branching arms extending outward from the canopy.
    pub fn place_trunk(
        &self,
        world: &mut impl OreBlockAccess,
        origin: (i32, i32, i32),
        log_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> (Vec<FoliageAttachment>, usize) {
        let (ox, oy, oz) = origin;
        let height = self.get_tree_height(rng);

        // Ensure dirt under all 4 base trunk positions
        for dx in 0..2 {
            for dz in 0..2 {
                world.set_block(ox + dx, oy - 1, oz + dz, SurfaceBlock::Dirt);
            }
        }

        let mut logs_placed = 0;

        // Place main 2x2 vertical pillar
        for dy in 0..height {
            let py = oy + dy;
            for dx in 0..2 {
                for dz in 0..2 {
                    let bx = ox + dx;
                    let bz = oz + dz;
                    let current = world.get_block(bx, py, bz);
                    if current == SurfaceBlock::Air
                        || current == SurfaceBlock::SnowLayer
                        || current == SurfaceBlock::Water
                        || current == SurfaceBlock::ShortGrass
                        || current == SurfaceBlock::TallGrass
                        || is_leaves(current)
                    {
                        world.set_block(bx, py, bz, log_block);
                        logs_placed += 1;
                    }
                }
            }
        }

        let mut attachments = Vec::new();
        // Top 2x2 center attachment
        attachments.push(FoliageAttachment::new(ox, oy + height, oz, 1, true));

        // Generate angled branches in the upper 40% of the trunk
        let branch_start_y = oy + (height * 3) / 5;
        let branch_directions = [
            (1, 0),  // East
            (-1, 0), // West
            (0, 1),  // South
            (0, -1), // North
            (1, 1),  // South-East
            (-1, 1), // South-West
        ];

        // Choose 2 to 4 branches
        let num_branches = 2 + rng.next_int_bounded(3);
        for i in 0..num_branches {
            let (dir_x, dir_z) = branch_directions[(i as usize) % branch_directions.len()];
            let branch_y_offset = rng.next_int_bounded((height - (branch_start_y - oy)).max(1));
            let start_y = branch_start_y + branch_y_offset;
            let branch_length = 2 + rng.next_int_bounded(3); // 2-4 blocks long

            let mut cur_x = ox + if dir_x > 0 { 1 } else { 0 };
            let mut cur_y = start_y;
            let mut cur_z = oz + if dir_z > 0 { 1 } else { 0 };

            for step in 1..=branch_length {
                cur_x += dir_x;
                cur_z += dir_z;
                if step % 2 == 0 {
                    cur_y += 1; // 45-degree upward branch trajectory
                }

                let current = world.get_block(cur_x, cur_y, cur_z);
                if current == SurfaceBlock::Air || is_leaves(current) {
                    world.set_block(cur_x, cur_y, cur_z, log_block);
                    logs_placed += 1;
                }
            }

            // Foliage attachment at branch tip
            attachments.push(FoliageAttachment::new(cur_x, cur_y, cur_z, 0, false));
        }

        (attachments, logs_placed)
    }
}

#[inline]
pub fn is_leaves(block: SurfaceBlock) -> bool {
    matches!(
        block,
        SurfaceBlock::OakLeaves
            | SurfaceBlock::BirchLeaves
            | SurfaceBlock::SpruceLeaves
            | SurfaceBlock::DarkOakLeaves
            | SurfaceBlock::JungleLeaves
    )
}
