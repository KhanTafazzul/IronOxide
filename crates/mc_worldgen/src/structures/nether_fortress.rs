//! Nether Fortress structure layout generator implementing bridges, blaze spawners, and wart rooms.
//! REA Reference: net.minecraft.world.level.levelgen.structure.structures.NetherFortressStructure (ekg.class)
//! REA Reference: net.minecraft.world.level.levelgen.structure.structures.NetherFortressPieces (ejo.class)

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;
use crate::structures::LegacyRandom;

/// Nether Fortress procedural generator.
pub struct NetherFortressGenerator;

impl NetherFortressGenerator {
    /// Generates a complete Nether Fortress assembly around the specified starting origin.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `origin`: Fortress starting origin (X, Y, Z). Usually $Y \in [48, 72]$.
    /// * `seed`: Fortress generation seed.
    ///
    /// Returns total count of placed structure blocks.
    pub fn generate(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
        seed: i64,
    ) -> usize {
        let mut rng = LegacyRandom::new(seed);
        let mut blocks_placed = 0;
        let (ox, oy, oz) = origin;

        let brick = SurfaceBlock::NetherBricks;
        let fence = SurfaceBlock::OakFence; // Standing in for nether brick fence in voxel proxy
        let soul_sand = SurfaceBlock::SoulSand;
        let wart = SurfaceBlock::NetherWart;
        let spawner = SurfaceBlock::Spawner;

        // 1. Central 4-Way Crossing (NeCCS: 9x6x9 bridge intersection)
        for dx in -4..=4 {
            for dz in -4..=4 {
                // Bridge deck floor at Y = oy
                world.set_block(ox + dx, oy, oz + dz, brick);
                blocks_placed += 1;

                // Air clearance inside junction
                for dy in 1..=4 {
                    world.set_block(ox + dx, oy + dy, oz + dz, SurfaceBlock::Air);
                }
            }
        }

        // Support columns under crossing corners extending down 10 blocks
        for &cx in &[-4, 4] {
            for &cz in &[-4, 4] {
                for dy in 1..=10 {
                    let py = oy - dy;
                    if py >= -64 {
                        world.set_block(ox + cx, py, oz + cz, brick);
                        blocks_placed += 1;
                    }
                }
            }
        }

        // 2. Straight Bridge Section extending North (NeBS: length 16)
        let bridge_len = 16;
        for step in 5..=(5 + bridge_len) {
            let cz = oz - step;
            for dx in -2..=2 {
                // Deck
                world.set_block(ox + dx, oy, cz, brick);
                blocks_placed += 1;

                // Air
                for dy in 1..=3 {
                    world.set_block(ox + dx, oy + dy, cz, SurfaceBlock::Air);
                }

                // Parapet railing on outer edges
                if dx == -2 || dx == 2 {
                    world.set_block(ox + dx, oy + 1, cz, fence);
                    blocks_placed += 1;
                }
            }

            // Support pillars every 8 blocks
            if step % 8 == 0 {
                for &px in &[-2, 2] {
                    for dy in 1..=8 {
                        let py = oy - dy;
                        if py >= -64 {
                            world.set_block(ox + px, py, cz, brick);
                            blocks_placed += 1;
                        }
                    }
                }
            }
        }

        // 3. Blaze Spawner Room at end of North Bridge (NeSCRC)
        let spawner_center_z = oz - (5 + bridge_len + 5);
        for dx in -4..=4 {
            for dz in -4..=4 {
                let px = ox + dx;
                let pz = spawner_center_z + dz;

                // Floor
                world.set_block(px, oy, pz, brick);
                blocks_placed += 1;

                // Air clearance
                for dy in 1..=5 {
                    world.set_block(px, oy + dy, pz, SurfaceBlock::Air);
                }

                // Perimeter railing
                if dx == -4 || dx == 4 || dz == -4 || dz == 4 {
                    world.set_block(px, oy + 1, pz, fence);
                    blocks_placed += 1;
                }
            }
        }

        // Raised spawner dais (3x3 at center, Y = oy + 1)
        for dx in -1..=1 {
            for dz in -1..=1 {
                world.set_block(ox + dx, oy + 1, spawner_center_z + dz, brick);
                blocks_placed += 1;
            }
        }
        // Monster Spawner on top of central dais
        world.set_block(ox, oy + 2, spawner_center_z, spawner);
        blocks_placed += 1;

        // 4. Nether Wart Staircase Room extending East (NeCSR)
        let wart_room_x = ox + (5 + bridge_len + 4);
        for dx in -4..=4 {
            for dz in -4..=4 {
                let px = wart_room_x + dx;
                let pz = oz + dz;

                // Base floor
                world.set_block(px, oy, pz, brick);
                blocks_placed += 1;

                // Hollow air
                for dy in 1..=5 {
                    world.set_block(px, oy + dy, pz, SurfaceBlock::Air);
                }

                // Enclosed walls
                if dx == -4 || dx == 4 || dz == -4 || dz == 4 {
                    for dy in 1..=4 {
                        world.set_block(px, oy + dy, pz, brick);
                        blocks_placed += 1;
                    }
                }
            }
        }

        // Soul Sand rows planted with Nether Wart on left and right flanks
        for &offset_z in &[-2, 2] {
            for dx in -2..=2 {
                let sx = wart_room_x + dx;
                let sz = oz + offset_z;

                world.set_block(sx, oy, sz, soul_sand);
                world.set_block(sx, oy + 1, sz, wart);
                blocks_placed += 2;
            }
        }

        // Small chest in wart room corner
        if rng.next_int_bounded(100) < 80 {
            world.set_block(wart_room_x + 3, oy + 1, oz + 3, SurfaceBlock::Chest);
            blocks_placed += 1;
        }

        blocks_placed
    }
}
