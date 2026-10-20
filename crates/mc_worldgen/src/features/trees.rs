//! Procedural tree generation engine implementing vanilla Minecraft 1.20+ trunk & foliage placers.
//! REA Reference: net.minecraft.world.level.levelgen.feature.TreeFeature (dxj.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.StraightTrunkPlacer (dxb.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer (dwx.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.MegaJungleTrunkPlacer (dxa.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.ForkingTrunkPlacer (dwy.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.DarkOakTrunkPlacer (dww.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.foliageplacers.BlobFoliagePlacer (dvf.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.foliageplacers.SpruceFoliagePlacer (dvq.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.foliageplacers.MegaPineFoliagePlacer (dvn.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.foliageplacers.AcaciaFoliagePlacer (dve.class)

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::LegacyRandom;
use std::f64::consts::PI;

/// Foliage attachment point emitted by trunk placers where leaf clusters generate.
/// REA Reference: `dvk$a.class`
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

/// Validates whether a block is considered tree leaves for placement replacement.
#[inline]
pub fn is_replaceable_for_tree(block: SurfaceBlock) -> bool {
    matches!(
        block,
        SurfaceBlock::Air
            | SurfaceBlock::SnowLayer
            | SurfaceBlock::Water
            | SurfaceBlock::ShortGrass
            | SurfaceBlock::TallGrass
            | SurfaceBlock::Fern
            | SurfaceBlock::Poppy
            | SurfaceBlock::Dandelion
            | SurfaceBlock::Sunflower
            | SurfaceBlock::OakLeaves
            | SurfaceBlock::BirchLeaves
            | SurfaceBlock::SpruceLeaves
            | SurfaceBlock::DarkOakLeaves
            | SurfaceBlock::JungleLeaves
            | SurfaceBlock::AcaciaLeaves
            | SurfaceBlock::PaleOakLeaves
            | SurfaceBlock::PaleMossCarpet
            | SurfaceBlock::PaleHangingMoss
    )
}

/// Validates if tree base can grow on the given soil block.
#[inline]
pub fn is_valid_tree_soil(block: SurfaceBlock) -> bool {
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

// ============================================================================
// TRUNK PLACERS
// ============================================================================

/// Trait for procedural trunk placers.
pub trait TrunkPlacer: Send + Sync {
    fn place_trunk(
        &self,
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        log_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> (Vec<FoliageAttachment>, usize);
}

/// Straight trunk placer generating a 1x1 vertical log pillar (Oak, Birch).
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
}

impl TrunkPlacer for StraightTrunkPlacer {
    fn place_trunk(
        &self,
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        log_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> (Vec<FoliageAttachment>, usize) {
        let (x, y, z) = origin;
        let height = self.get_tree_height(rng);

        // Vanilla rule: set soil beneath trunk to dirt
        world.set_block(x, y - 1, z, SurfaceBlock::Dirt);

        let mut logs_placed = 0;
        for dy in 0..height {
            let py = y + dy;
            let current = world.get_block(x, py, z);
            if is_replaceable_for_tree(current) {
                world.set_block(x, py, z, log_block);
                logs_placed += 1;
            }
        }

        let attachment = FoliageAttachment::new(x, y + height, z, 0, false);
        (vec![attachment], logs_placed)
    }
}

/// Fancy trunk placer generating branching angled limbs with log branch clusters (Large Oak).
/// REA Reference: `dwx.class` (`FancyTrunkPlacer`)
#[derive(Clone, Copy, Debug)]
pub struct FancyTrunkPlacer {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
}

impl Default for FancyTrunkPlacer {
    fn default() -> Self {
        Self {
            base_height: 8,
            height_rand_a: 4,
            height_rand_b: 2,
        }
    }
}

impl FancyTrunkPlacer {
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

    /// Draws a discrete Bresenham 3D log line between two coordinates.
    fn draw_limb(
        world: &mut dyn OreBlockAccess,
        from: (i32, i32, i32),
        to: (i32, i32, i32),
        log_block: SurfaceBlock,
    ) -> usize {
        let (x0, y0, z0) = from;
        let (x1, y1, z1) = to;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let dz = (z1 - z0).abs();
        let max_d = dx.max(dy).max(dz);
        if max_d == 0 {
            if is_replaceable_for_tree(world.get_block(x0, y0, z0)) {
                world.set_block(x0, y0, z0, log_block);
                return 1;
            }
            return 0;
        }

        let mut logs = 0;
        for step in 0..=max_d {
            let t = step as f64 / max_d as f64;
            let px = (x0 as f64 + (x1 - x0) as f64 * t).round() as i32;
            let py = (y0 as f64 + (y1 - y0) as f64 * t).round() as i32;
            let pz = (z0 as f64 + (z1 - z0) as f64 * t).round() as i32;

            if is_replaceable_for_tree(world.get_block(px, py, pz)) {
                world.set_block(px, py, pz, log_block);
                logs += 1;
            }
        }
        logs
    }
}

impl TrunkPlacer for FancyTrunkPlacer {
    fn place_trunk(
        &self,
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        log_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> (Vec<FoliageAttachment>, usize) {
        let (ox, oy, oz) = origin;
        let height = self.get_tree_height(rng);

        world.set_block(ox, oy - 1, oz, SurfaceBlock::Dirt);

        let mut logs_placed = 0;
        // Central vertical trunk up to main branch node
        let trunk_top_y = oy + (height as f64 * 0.618) as i32;
        for y in oy..=trunk_top_y {
            if is_replaceable_for_tree(world.get_block(ox, y, oz)) {
                world.set_block(ox, y, oz, log_block);
                logs_placed += 1;
            }
        }

        let mut attachments = Vec::new();
        // Top crown attachment
        attachments.push(FoliageAttachment::new(ox, oy + height, oz, 0, false));
        logs_placed += Self::draw_limb(world, (ox, trunk_top_y, oz), (ox, oy + height, oz), log_block);

        // Angled branch limbs extending outward and upward
        let branch_count = 3 + rng.next_int_bounded(3); // 3-5 limbs
        let branch_start_y = oy + (height / 3);

        for i in 0..branch_count {
            let angle = (i as f64 / branch_count as f64) * 2.0 * PI + (rng.next_float() as f64 * 0.4);
            let branch_length = 3.0 + (rng.next_float() as f64 * 3.0); // 3-6 blocks reach
            let branch_dy = 2 + rng.next_int_bounded(3);

            let branch_y = branch_start_y + rng.next_int_bounded((trunk_top_y - branch_start_y).max(1) + 1);
            let end_x = ox + (angle.cos() * branch_length).round() as i32;
            let end_y = (branch_y + branch_dy).min(oy + height);
            let end_z = oz + (angle.sin() * branch_length).round() as i32;

            logs_placed += Self::draw_limb(world, (ox, branch_y, oz), (end_x, end_y, end_z), log_block);
            attachments.push(FoliageAttachment::new(end_x, end_y, end_z, -1, false));
        }

        (attachments, logs_placed)
    }
}

/// Mega Jungle Trunk Placer generating a 2x2 thick base with stair-stepping trunk growth and vine attachments.
/// REA Reference: `dxa.class` (`MegaJungleTrunkPlacer`) & `dwz.class` (`GiantTrunkPlacer`)
#[derive(Clone, Copy, Debug)]
pub struct MegaJungleTrunkPlacer {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
}

impl Default for MegaJungleTrunkPlacer {
    fn default() -> Self {
        Self {
            base_height: 12,
            height_rand_a: 6,
            height_rand_b: 2,
        }
    }
}

impl MegaJungleTrunkPlacer {
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
}

impl TrunkPlacer for MegaJungleTrunkPlacer {
    fn place_trunk(
        &self,
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        log_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> (Vec<FoliageAttachment>, usize) {
        let (ox, oy, oz) = origin;
        let height = self.get_tree_height(rng);

        // Dirt under 2x2 base
        for dx in 0..2 {
            for dz in 0..2 {
                world.set_block(ox + dx, oy - 1, oz + dz, SurfaceBlock::Dirt);
            }
        }

        let mut logs_placed = 0;

        // 1. Place central 2x2 vertical pillar (dwz.class)
        for dy in 0..height {
            let py = oy + dy;
            for dx in 0..2 {
                for dz in 0..2 {
                    let bx = ox + dx;
                    let bz = oz + dz;
                    if is_replaceable_for_tree(world.get_block(bx, py, bz)) {
                        world.set_block(bx, py, bz, log_block);
                        logs_placed += 1;
                    }
                }
            }
        }

        let mut attachments = Vec::new();
        // Top 2x2 canopy attachment
        attachments.push(FoliageAttachment::new(ox, oy + height, oz, 0, true));

        // 2. Stair-stepping branches (dxa.class lines 26-186)
        let mut branch_y = height - 2 - rng.next_int_bounded(4);
        while branch_y > height / 2 {
            let angle = rng.next_float() * (2.0 * PI as f32);
            let mut end_x = 0;
            let mut end_z = 0;

            for step in 0..5 {
                end_x = (1.5 + angle.cos() * step as f32) as i32;
                end_z = (1.5 + angle.sin() * step as f32) as i32;
                let step_y = branch_y - 3 + step / 2;

                let px = ox + end_x;
                let py = oy + step_y;
                let pz = oz + end_z;

                if is_replaceable_for_tree(world.get_block(px, py, pz)) {
                    world.set_block(px, py, pz, log_block);
                    logs_placed += 1;
                }
            }

            attachments.push(FoliageAttachment::new(ox + end_x, oy + branch_y, oz + end_z, -2, false));
            branch_y -= 2 + rng.next_int_bounded(4);
        }

        (attachments, logs_placed)
    }
}

/// Forking trunk placer generating diagonal fork splits (Acacia).
/// REA Reference: `dwy.class` (`ForkingTrunkPlacer`)
#[derive(Clone, Copy, Debug)]
pub struct ForkingTrunkPlacer {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
}

impl Default for ForkingTrunkPlacer {
    fn default() -> Self {
        Self {
            base_height: 5,
            height_rand_a: 2,
            height_rand_b: 2,
        }
    }
}

impl ForkingTrunkPlacer {
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

    /// Selects a horizontal cardinal direction (dx, dz).
    fn pick_horizontal_direction(rng: &mut LegacyRandom) -> (i32, i32) {
        match rng.next_int_bounded(4) {
            0 => (1, 0),  // East
            1 => (-1, 0), // West
            2 => (0, 1),  // South
            _ => (0, -1), // North
        }
    }
}

impl TrunkPlacer for ForkingTrunkPlacer {
    fn place_trunk(
        &self,
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        log_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> (Vec<FoliageAttachment>, usize) {
        let (ox, oy, oz) = origin;
        let height = self.get_tree_height(rng);

        world.set_block(ox, oy - 1, oz, SurfaceBlock::Dirt);

        let mut attachments = Vec::new();
        let mut logs_placed = 0;

        let dir1 = Self::pick_horizontal_direction(rng);
        let split_y1 = height - rng.next_int_bounded(4) - 1;
        let mut steps1 = 3 - rng.next_int_bounded(3);

        let mut cur_x = ox;
        let mut cur_z = oz;
        let mut last_y1 = oy;

        // Primary fork branch
        for step in 0..height {
            let py = oy + step;
            if step >= split_y1 && steps1 > 0 {
                cur_x += dir1.0;
                cur_z += dir1.1;
                steps1 -= 1;
            }

            if is_replaceable_for_tree(world.get_block(cur_x, py, cur_z)) {
                world.set_block(cur_x, py, cur_z, log_block);
                logs_placed += 1;
            }
            last_y1 = py;
        }

        attachments.push(FoliageAttachment::new(cur_x, last_y1 + 1, cur_z, 1, false));

        // Secondary fork branch splitting in a different direction
        let mut dir2 = Self::pick_horizontal_direction(rng);
        if dir2 == dir1 {
            dir2 = (-dir1.1, dir1.0); // 90 degree turn if same
        }

        let split_y2 = split_y1 - rng.next_int_bounded(2) - 1;
        let mut branch2_len = 1 + rng.next_int_bounded(3);
        let mut cur2_x = ox;
        let mut cur2_z = oz;
        let mut last_y2 = None;

        for step in split_y2..height {
            if branch2_len <= 0 {
                break;
            }
            if step < 1 {
                continue;
            }
            let py = oy + step;
            cur2_x += dir2.0;
            cur2_z += dir2.1;

            if is_replaceable_for_tree(world.get_block(cur2_x, py, cur2_z)) {
                world.set_block(cur2_x, py, cur2_z, log_block);
                logs_placed += 1;
                last_y2 = Some(py);
            }
            branch2_len -= 1;
        }

        if let Some(top_y2) = last_y2 {
            attachments.push(FoliageAttachment::new(cur2_x, top_y2 + 1, cur2_z, 0, false));
        }

        (attachments, logs_placed)
    }
}

/// Dark Oak Trunk Placer generating a 2x2 base with roots and thick dense foliage.
/// REA Reference: `dww.class` (`DarkOakTrunkPlacer`)
#[derive(Clone, Copy, Debug)]
pub struct DarkOakTrunkPlacer {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
}

impl Default for DarkOakTrunkPlacer {
    fn default() -> Self {
        Self {
            base_height: 6,
            height_rand_a: 2,
            height_rand_b: 1,
        }
    }
}

impl DarkOakTrunkPlacer {
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

    fn pick_horizontal_direction(rng: &mut LegacyRandom) -> (i32, i32) {
        match rng.next_int_bounded(4) {
            0 => (1, 0),
            1 => (-1, 0),
            2 => (0, 1),
            _ => (0, -1),
        }
    }
}

impl TrunkPlacer for DarkOakTrunkPlacer {
    fn place_trunk(
        &self,
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        log_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> (Vec<FoliageAttachment>, usize) {
        let (ox, oy, oz) = origin;
        let height = self.get_tree_height(rng);

        // Base 2x2 dirt
        for dx in 0..2 {
            for dz in 0..2 {
                world.set_block(ox + dx, oy - 1, oz + dz, SurfaceBlock::Dirt);
            }
        }

        let mut logs_placed = 0;
        let mut attachments = Vec::new();

        let dir = Self::pick_horizontal_direction(rng);
        let split_y = height - rng.next_int_bounded(4);
        let mut steps = 2 - rng.next_int_bounded(3);

        let mut cur_x = ox;
        let mut cur_z = oz;
        let top_y = oy + height - 1;

        // Main 2x2 trunk pillar with upper lean
        for dy in 0..height {
            if dy >= split_y && steps > 0 {
                cur_x += dir.0;
                cur_z += dir.1;
                steps -= 1;
            }

            let py = oy + dy;
            for dx in 0..2 {
                for dz in 0..2 {
                    let bx = cur_x + dx;
                    let bz = cur_z + dz;
                    if is_replaceable_for_tree(world.get_block(bx, py, bz)) {
                        world.set_block(bx, py, bz, log_block);
                        logs_placed += 1;
                    }
                }
            }
        }

        // Top 2x2 giant attachment
        attachments.push(FoliageAttachment::new(cur_x, top_y, cur_z, 0, true));

        // Lateral canopy branches around the 2x2 perimeter
        for dx in -1..=2 {
            for dz in -1..=2 {
                // Skip the inner 2x2 core
                if (0..=1).contains(&dx) && (0..=1).contains(&dz) {
                    continue;
                }

                if rng.next_int_bounded(3) > 0 {
                    let branch_len = 2 + rng.next_int_bounded(3);
                    for step in 0..branch_len {
                        let bx = ox + dx;
                        let by = top_y - step - 1;
                        let bz = oz + dz;
                        if is_replaceable_for_tree(world.get_block(bx, by, bz)) {
                            world.set_block(bx, by, bz, log_block);
                            logs_placed += 1;
                        }
                    }

                    attachments.push(FoliageAttachment::new(cur_x + dx, top_y, cur_z + dz, 0, false));
                }
            }
        }

        (attachments, logs_placed)
    }
}

// ============================================================================
// FOLIAGE PLACERS
// ============================================================================

/// Trait for procedural foliage placers.
pub trait FoliagePlacer: Send + Sync {
    fn place_foliage(
        &self,
        world: &mut dyn OreBlockAccess,
        attachment: &FoliageAttachment,
        leaf_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> usize;
}

/// Blob foliage placer generating spherical radius clusters with leaf decay chance at corners.
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
}

impl FoliagePlacer for BlobFoliagePlacer {
    fn place_foliage(
        &self,
        world: &mut dyn OreBlockAccess,
        attachment: &FoliageAttachment,
        leaf_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> usize {
        let center_x = attachment.x;
        let center_y = attachment.y;
        let center_z = attachment.z;
        let effective_radius = self.radius + attachment.radius_offset;

        let mut leaves_placed = 0;

        for dy in (-(self.height - 1)..=0).rev() {
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

                    // Vanilla corner decay: 100% on top layer, 50% on lower layers
                    if is_corner && (dy == 0 || rng.next_int_bounded(2) == 0) {
                        continue;
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

/// Spruce foliage placer generating stepped conical foliage rings tapering to a 1-block tip.
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
}

impl FoliagePlacer for SpruceFoliagePlacer {
    fn place_foliage(
        &self,
        world: &mut dyn OreBlockAccess,
        attachment: &FoliageAttachment,
        leaf_block: SurfaceBlock,
        rng: &mut LegacyRandom,
    ) -> usize {
        let center_x = attachment.x;
        let center_y = attachment.y;
        let center_z = attachment.z;

        let mut leaves_placed = 0;

        // 1-block tip at top
        if world.get_block(center_x, center_y, center_z) == SurfaceBlock::Air {
            world.set_block(center_x, center_y, center_z, leaf_block);
            leaves_placed += 1;
        }

        // Stepped conical rings tapering down
        let mut current_radius = 1;
        let total_layers = self.height.max(3);

        for dy in 1..total_layers {
            let py = center_y - dy;
            let step = dy % 2;
            if step == 0 && current_radius < (self.base_radius + attachment.radius_offset) {
                current_radius += 1;
            }

            for dx in -current_radius..=current_radius {
                for dz in -current_radius..=current_radius {
                    let is_corner = dx.abs() == current_radius && dz.abs() == current_radius;
                    if is_corner && (current_radius > 1 && rng.next_int_bounded(2) == 0) {
                        continue;
                    }

                    let px = center_x + dx;
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

/// Mega Pine foliage placer generating crown foliage starting high on trunk.
/// REA Reference: `dvn.class` (`MegaPineFoliagePlacer`)
#[derive(Clone, Copy, Debug)]
pub struct MegaPineFoliagePlacer {
    pub base_radius: i32,
    pub crown_height: i32,
}

impl Default for MegaPineFoliagePlacer {
    fn default() -> Self {
        Self { base_radius: 0, crown_height: 13 }
    }
}

impl MegaPineFoliagePlacer {
    pub const fn new(base_radius: i32, crown_height: i32) -> Self {
        Self { base_radius, crown_height }
    }
}

impl FoliagePlacer for MegaPineFoliagePlacer {
    fn place_foliage(
        &self,
        world: &mut dyn OreBlockAccess,
        attachment: &FoliageAttachment,
        leaf_block: SurfaceBlock,
        _rng: &mut LegacyRandom,
    ) -> usize {
        let center_x = attachment.x;
        let center_y = attachment.y;
        let center_z = attachment.z;

        let mut leaves_placed = 0;
        let mut prev_radius = 0;

        // Iterate from top of crown downward
        for cur_y in (center_y - self.crown_height)..=center_y {
            let dy_from_top = center_y - cur_y;
            let expansion = ((dy_from_top as f32 / self.crown_height as f32) * 3.5).floor() as i32;
            let mut radius = self.base_radius + attachment.radius_offset + expansion;

            // Alternating stepped rings matching dvn.class line 82
            if dy_from_top > 0 && radius == prev_radius && (cur_y & 1) == 0 {
                radius += 1;
            }

            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let abs_x = dx.abs();
                    let abs_z = dz.abs();

                    // REA dvn.class line 1-33:
                    // dx + dz >= 7 -> skip
                    // dx^2 + dz^2 > radius^2 -> skip
                    if abs_x + abs_z >= 7 || (abs_x * abs_x + abs_z * abs_z > radius * radius) {
                        continue;
                    }

                    let px = center_x + dx;
                    let py = cur_y;
                    let pz = center_z + dz;

                    let current = world.get_block(px, py, pz);
                    if current == SurfaceBlock::Air || current == SurfaceBlock::SnowLayer {
                        world.set_block(px, py, pz, leaf_block);
                        leaves_placed += 1;
                    }
                }
            }

            prev_radius = radius;
        }

        leaves_placed
    }
}

/// Acacia foliage placer generating flat umbrella canopy disks.
/// REA Reference: `dve.class` (`AcaciaFoliagePlacer`)
#[derive(Clone, Copy, Debug)]
pub struct AcaciaFoliagePlacer {
    pub radius: i32,
}

impl Default for AcaciaFoliagePlacer {
    fn default() -> Self {
        Self { radius: 2 }
    }
}

impl AcaciaFoliagePlacer {
    pub const fn new(radius: i32) -> Self {
        Self { radius }
    }

    /// Tests if a block at (dx, dy, dz) relative to disk center should be skipped.
    /// REA Reference: `dve.class` lines 0-53
    fn should_skip(dx: i32, dy: i32, dz: i32, radius: i32) -> bool {
        let abs_x = dx.abs();
        let abs_z = dz.abs();
        if dy == 0 {
            // Top layer of disk: skip corner extensions (lines 4-28)
            (abs_x > 1 || abs_z > 1) && abs_x != 0 && abs_z != 0
        } else {
            // Lower layers: skip sharp corners
            abs_x == radius && abs_z == radius && radius > 0
        }
    }

    fn place_disk(
        world: &mut dyn OreBlockAccess,
        cx: i32,
        cy: i32,
        cz: i32,
        radius: i32,
        dy: i32,
        leaf_block: SurfaceBlock,
    ) -> usize {
        let mut placed = 0;
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if Self::should_skip(dx, dy, dz, radius) {
                    continue;
                }
                let px = cx + dx;
                let py = cy;
                let pz = cz + dz;
                let current = world.get_block(px, py, pz);
                if current == SurfaceBlock::Air || current == SurfaceBlock::SnowLayer {
                    world.set_block(px, py, pz, leaf_block);
                    placed += 1;
                }
            }
        }
        placed
    }
}

impl FoliagePlacer for AcaciaFoliagePlacer {
    fn place_foliage(
        &self,
        world: &mut dyn OreBlockAccess,
        attachment: &FoliageAttachment,
        leaf_block: SurfaceBlock,
        _rng: &mut LegacyRandom,
    ) -> usize {
        let cx = attachment.x;
        let cy = attachment.y;
        let cz = attachment.z;
        let r_offset = attachment.radius_offset;

        let mut placed = 0;

        // REA Reference: dve.class lines 19-88
        // Disk 1 at Y = cy - 1, radius = self.radius + r_offset
        let r1 = (self.radius + r_offset).max(1);
        placed += Self::place_disk(world, cx, cy - 1, cz, r1, -1, leaf_block);

        // Disk 2 at Y = cy, radius = (self.radius - 1).max(1)
        let r2 = (self.radius - 1).max(1);
        placed += Self::place_disk(world, cx, cy, cz, r2, 0, leaf_block);

        // Disk 3 at Y = cy + 1, radius = (self.radius + r_offset - 1).max(0)
        let r3 = (self.radius + r_offset - 1).max(0);
        if r3 > 0 {
            placed += Self::place_disk(world, cx, cy + 1, cz, r3, 0, leaf_block);
        }

        placed
    }
}

// ============================================================================
// TREE FEATURE
// ============================================================================

/// Tree species variant configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TreeSpecies {
    Oak,
    FancyOak,
    Birch,
    Spruce,
    MegaPine,
    MegaJungle,
    Acacia,
    DarkOak,
    PaleOak,
}

impl TreeSpecies {
    pub fn log_block(&self) -> SurfaceBlock {
        match self {
            Self::Oak | Self::FancyOak => SurfaceBlock::OakLog,
            Self::Birch => SurfaceBlock::BirchLog,
            Self::Spruce | Self::MegaPine => SurfaceBlock::SpruceLog,
            Self::MegaJungle => SurfaceBlock::JungleLog,
            Self::Acacia => SurfaceBlock::AcaciaLog,
            Self::DarkOak => SurfaceBlock::DarkOakLog,
            Self::PaleOak => SurfaceBlock::PaleOakLog,
        }
    }

    pub fn leaf_block(&self) -> SurfaceBlock {
        match self {
            Self::Oak | Self::FancyOak => SurfaceBlock::OakLeaves,
            Self::Birch => SurfaceBlock::BirchLeaves,
            Self::Spruce | Self::MegaPine => SurfaceBlock::SpruceLeaves,
            Self::MegaJungle => SurfaceBlock::JungleLeaves,
            Self::Acacia => SurfaceBlock::AcaciaLeaves,
            Self::DarkOak => SurfaceBlock::DarkOakLeaves,
            Self::PaleOak => SurfaceBlock::PaleOakLeaves,
        }
    }
}

/// Orchestrator generating full procedural trees with matched trunk and foliage placers.
pub struct TreeFeature;

impl TreeFeature {
    /// Generates a procedural tree of the specified species at the given origin.
    pub fn place_tree(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        species: TreeSpecies,
        rng: &mut LegacyRandom,
    ) -> (usize, usize) {
        let (x, y, z) = origin;
        let soil = world.get_block(x, y - 1, z);
        if !is_valid_tree_soil(soil) {
            return (0, 0);
        }

        let log = species.log_block();
        let leaves = species.leaf_block();

        match species {
            TreeSpecies::Oak => {
                let trunk = StraightTrunkPlacer::new(4, 2, 0); // 4-6 blocks
                let foliage = BlobFoliagePlacer::new(2, 3);
                let (attachments, logs) = trunk.place_trunk(world, origin, log, rng);
                let mut leaves_placed = 0;
                for att in &attachments {
                    leaves_placed += foliage.place_foliage(world, att, leaves, rng);
                }
                (logs, leaves_placed)
            }
            TreeSpecies::FancyOak => {
                let trunk = FancyTrunkPlacer::new(8, 4, 2);
                let foliage = BlobFoliagePlacer::new(2, 3);
                let (attachments, logs) = trunk.place_trunk(world, origin, log, rng);
                let mut leaves_placed = 0;
                for att in &attachments {
                    leaves_placed += foliage.place_foliage(world, att, leaves, rng);
                }
                (logs, leaves_placed)
            }
            TreeSpecies::Birch => {
                let trunk = StraightTrunkPlacer::new(5, 2, 0); // 5-7 blocks
                let foliage = BlobFoliagePlacer::new(2, 3);
                let (attachments, logs) = trunk.place_trunk(world, origin, log, rng);
                let mut leaves_placed = 0;
                for att in &attachments {
                    leaves_placed += foliage.place_foliage(world, att, leaves, rng);
                }
                (logs, leaves_placed)
            }
            TreeSpecies::Spruce => {
                let trunk = StraightTrunkPlacer::new(6, 3, 0); // 6-9 blocks
                let foliage = SpruceFoliagePlacer::new(2, 6);
                let (attachments, logs) = trunk.place_trunk(world, origin, log, rng);
                let mut leaves_placed = 0;
                for att in &attachments {
                    leaves_placed += foliage.place_foliage(world, att, leaves, rng);
                }
                (logs, leaves_placed)
            }
            TreeSpecies::MegaPine => {
                let trunk = StraightTrunkPlacer::new(13, 4, 0);
                let foliage = MegaPineFoliagePlacer::new(0, 13);
                let (attachments, logs) = trunk.place_trunk(world, origin, log, rng);
                let mut leaves_placed = 0;
                for att in &attachments {
                    leaves_placed += foliage.place_foliage(world, att, leaves, rng);
                }
                (logs, leaves_placed)
            }
            TreeSpecies::MegaJungle => {
                let trunk = MegaJungleTrunkPlacer::new(12, 6, 2);
                let foliage = BlobFoliagePlacer::new(3, 3);
                let (attachments, logs) = trunk.place_trunk(world, origin, log, rng);
                let mut leaves_placed = 0;
                for att in &attachments {
                    leaves_placed += foliage.place_foliage(world, att, leaves, rng);
                }
                (logs, leaves_placed)
            }
            TreeSpecies::Acacia => {
                let trunk = ForkingTrunkPlacer::new(5, 2, 2);
                let foliage = AcaciaFoliagePlacer::new(2);
                let (attachments, logs) = trunk.place_trunk(world, origin, log, rng);
                let mut leaves_placed = 0;
                for att in &attachments {
                    leaves_placed += foliage.place_foliage(world, att, leaves, rng);
                }
                (logs, leaves_placed)
            }
            TreeSpecies::DarkOak => {
                let trunk = DarkOakTrunkPlacer::new(6, 2, 1);
                let foliage = BlobFoliagePlacer::new(2, 3);
                let (attachments, logs) = trunk.place_trunk(world, origin, log, rng);
                let mut leaves_placed = 0;
                for att in &attachments {
                    leaves_placed += foliage.place_foliage(world, att, leaves, rng);
                }
                (logs, leaves_placed)
            }
            TreeSpecies::PaleOak => {
                let res = crate::biomes::pale_garden::generate_pale_oak_tree(world, origin, rng, false);
                (res.logs_placed, res.leaves_placed + res.moss_vines_placed)
            }
        }
    }
}
