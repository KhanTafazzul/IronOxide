//! Procedural Abandoned Mineshaft generator implementing corridor networks, supports, and webs.
//! REA Reference: net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure
//! REA Reference: net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces (dzx / MSCorridor, MSCrossing, MSRoom)

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::LegacyRandom;

/// Mineshaft variant classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MineshaftType {
    Normal,
    Mesa,
}

impl MineshaftType {
    pub fn plank_block(&self) -> SurfaceBlock {
        match self {
            Self::Normal => SurfaceBlock::OakPlanks,
            Self::Mesa => SurfaceBlock::DarkOakPlanks,
        }
    }

    pub fn fence_block(&self) -> SurfaceBlock {
        SurfaceBlock::OakFence
    }
}

/// Abandoned Mineshaft generator.
pub struct MineshaftGenerator;

impl MineshaftGenerator {
    /// Generates a procedural Abandoned Mineshaft network at the given origin.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `origin`: Starting center position (X, Y, Z). Usually $Y \in [-50, 30]$.
    /// * `mineshaft_type`: Normal or Mesa variant.
    /// * `seed`: World generation or chunk seed.
    ///
    /// Returns the total count of structure blocks placed.
    pub fn generate(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        mineshaft_type: MineshaftType,
        seed: i64,
    ) -> usize {
        let mut rng = LegacyRandom::new(seed);
        let mut blocks_placed = 0;
        let (ox, oy, oz) = origin;

        let planks = mineshaft_type.plank_block();
        let fence = mineshaft_type.fence_block();

        // 1. Central Starting Room (MSRoom: 10x6x10 chamber)
        let room_w = 8;
        let room_h = 5;
        let room_d = 8;

        for dx in -room_w / 2..=room_w / 2 {
            for dz in -room_d / 2..=room_d / 2 {
                // Dirt/Cobblestone floor
                world.set_block(ox + dx, oy - 1, oz + dz, SurfaceBlock::Cobblestone);
                blocks_placed += 1;

                // Hollow room air space
                for dy in 0..room_h {
                    world.set_block(ox + dx, oy + dy, oz + dz, SurfaceBlock::Air);
                }

                // Ceiling
                world.set_block(ox + dx, oy + room_h, oz + dz, planks);
                blocks_placed += 1;
            }
        }

        // Center room support pillars
        for &px in &[-2, 2] {
            for &pz in &[-2, 2] {
                for dy in 0..room_h {
                    world.set_block(ox + px, oy + dy, oz + pz, planks);
                    blocks_placed += 1;
                }
            }
        }

        // 2. Corridors branching in the 4 cardinal directions (MSCorridor)
        let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        for (dir_x, dir_z) in directions {
            let corridor_len = 15 + rng.next_int_bounded(10); // 15-24 blocks long
            let start_x = ox + dir_x * (room_w / 2 + 1);
            let start_z = oz + dir_z * (room_d / 2 + 1);

            for step in 0..corridor_len {
                let cx = start_x + dir_x * step;
                let cz = start_z + dir_z * step;

                // Transverse vector (perpendicular to direction)
                let (tx, tz) = (-dir_z, dir_x);

                // Clear 3x3 tunnel
                for t in -1..=1 {
                    for dy in 0..3 {
                        let bx = cx + tx * t;
                        let by = oy + dy;
                        let bz = cz + tz * t;
                        world.set_block(bx, by, bz, SurfaceBlock::Air);
                    }
                    // Floor foundation
                    let fx = cx + tx * t;
                    let fz = cz + tz * t;
                    if world.get_block(fx, oy - 1, fz) == SurfaceBlock::Air {
                        world.set_block(fx, oy - 1, fz, planks);
                        blocks_placed += 1;
                    }
                }

                // Place central rail line
                world.set_block(cx, oy, cz, SurfaceBlock::Rail);
                blocks_placed += 1;

                // Support arches every 5 blocks: 2 vertical fence posts + 1 horizontal plank beam
                if step % 5 == 0 {
                    // Left and right fence posts
                    let lx = cx + tx * -1;
                    let lz = cz + tz * -1;
                    let rx = cx + tx * 1;
                    let rz = cz + tz * 1;

                    world.set_block(lx, oy, lz, fence);
                    world.set_block(lx, oy + 1, lz, fence);
                    world.set_block(rx, oy, rz, fence);
                    world.set_block(rx, oy + 1, rz, fence);
                    blocks_placed += 4;

                    // Horizontal top plank beam across ceiling
                    for t in -1..=1 {
                        world.set_block(cx + tx * t, oy + 2, cz + tz * t, planks);
                        blocks_placed += 1;
                    }

                    // 15% chance of cobwebs near support
                    if rng.next_int_bounded(100) < 15 {
                        world.set_block(cx + tx * -1, oy + 1, cz + tz * -1, SurfaceBlock::Cobweb);
                        blocks_placed += 1;
                    }

                    // 5% chance of chest minecart / chest on the side
                    if rng.next_int_bounded(100) < 5 {
                        world.set_block(cx + tx * 1, oy, cz + tz * 1, SurfaceBlock::Chest);
                        blocks_placed += 1;
                    }
                }
            }
        }

        blocks_placed
    }
}
