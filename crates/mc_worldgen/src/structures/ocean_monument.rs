//! Ocean Monument procedural generator implementing the 58x58 prismarine complex and guardian chambers.
//! REA Reference: net.minecraft.world.level.levelgen.structure.structures.OceanMonumentStructure (elc.class)
//! REA Reference: net.minecraft.world.level.levelgen.structure.structures.OceanMonumentPieces (ejq.class)

use crate::biomes::SurfaceBlock;
use crate::ores::OreBlockAccess;

/// Ocean Monument generator.
pub struct OceanMonumentGenerator;

impl OceanMonumentGenerator {
    /// Generates a complete Ocean Monument temple complex seated at the given ocean floor origin.
    ///
    /// # Arguments
    /// * `world`: The block access buffer.
    /// * `origin`: Monument base center (X, Y, Z). Usually $Y \in [39, 45]$ on ocean floor.
    ///
    /// Returns the total count of structure blocks placed.
    pub fn generate(
        world: &mut dyn OreBlockAccess,
        origin: (i32, i32, i32),
    ) -> usize {
        let mut blocks_placed = 0;
        let (ox, oy, oz) = origin;

        let prismarine = SurfaceBlock::Prismarine;
        let bricks = SurfaceBlock::PrismarineBricks;
        let dark_prismarine = SurfaceBlock::DarkPrismarine;
        let sea_lantern = SurfaceBlock::SeaLantern;
        let sponge = SurfaceBlock::Sponge;
        let gold_block = SurfaceBlock::GoldBlock;

        let half_w = 28; // 57x57 base footprint centered at origin
        let total_h = 22;

        // 1. Foundation Base Slab (57x57)
        for dx in -half_w..=half_w {
            for dz in -half_w..=half_w {
                let px = ox + dx;
                let pz = oz + dz;

                // Base floor
                world.set_block(px, oy, pz, prismarine);
                blocks_placed += 1;

                // Foundation pillars extending down to solid seabed if hollow below
                for dy in 1..=4 {
                    let py = oy - dy;
                    if py >= -64 && world.get_block(px, py, pz) == SurfaceBlock::Water {
                        world.set_block(px, py, pz, prismarine);
                        blocks_placed += 1;
                    }
                }
            }
        }

        // 2. Main Temple Outer Shell (Stepped pyramid rising to Y = oy + 22)
        for layer in 0..total_h {
            let py = oy + layer;
            let inset = (layer / 3) * 2;
            let cur_w = (half_w - inset).max(4);

            for dx in -cur_w..=cur_w {
                for dz in -cur_w..=cur_w {
                    let is_outer_edge = dx == -cur_w || dx == cur_w || dz == -cur_w || dz == cur_w;
                    let is_corner = dx.abs() == cur_w && dz.abs() == cur_w;
                    let px = ox + dx;
                    let pz = oz + dz;

                    if is_corner {
                        // Sea lanterns at exterior corners
                        if layer % 4 == 0 {
                            world.set_block(px, py, pz, sea_lantern);
                            blocks_placed += 1;
                        } else {
                            world.set_block(px, py, pz, dark_prismarine);
                            blocks_placed += 1;
                        }
                    } else if is_outer_edge {
                        // Alternating decorative bands
                        if layer % 2 == 0 {
                            world.set_block(px, py, pz, bricks);
                        } else {
                            world.set_block(px, py, pz, prismarine);
                        }
                        blocks_placed += 1;
                    } else {
                        // Interior water-filled rooms
                        if layer == 0 || layer == 8 || layer == 16 {
                            // Sub-floors
                            world.set_block(px, py, pz, bricks);
                            blocks_placed += 1;
                        } else {
                            world.set_block(px, py, pz, SurfaceBlock::Water);
                        }
                    }
                }
            }
        }

        // 3. Central Treasure Chamber (encased core containing 8 Sponge blocks & Gold Blocks)
        let core_y = oy + 4;
        for dx in -2..=2 {
            for dz in -2..=2 {
                for dy in 0..4 {
                    let px = ox + dx;
                    let py = core_y + dy;
                    let pz = oz + dz;

                    let is_casing = dx == -2 || dx == 2 || dz == -2 || dz == 2 || dy == 0 || dy == 3;
                    if is_casing {
                        world.set_block(px, py, pz, dark_prismarine);
                        blocks_placed += 1;
                    } else {
                        // Core 2x2x2 Gold Block treasury (8 gold blocks) surrounded by Sponges
                        if dy == 1 || dy == 2 {
                            if (dx == 0 || dx == 1) && (dz == 0 || dz == 1) {
                                world.set_block(px, py, pz, gold_block);
                            } else {
                                world.set_block(px, py, pz, sponge);
                            }
                            blocks_placed += 1;
                        }
                    }
                }
            }
        }

        // 4. Three Elder Guardian Spawn Chambers:
        // A. Top Penthouse (Apex of temple)
        let apex_y = oy + total_h - 2;
        world.set_block(ox, apex_y, oz, sea_lantern);
        blocks_placed += 1;

        // B. West Wing Chamber
        world.set_block(ox - 14, oy + 4, oz, sea_lantern);
        blocks_placed += 1;

        // C. East Wing Chamber
        world.set_block(ox + 14, oy + 4, oz, sea_lantern);
        blocks_placed += 1;

        // 5. Grand Entrance Archway at Front (South face)
        let front_z = oz + half_w;
        for dx in -3..=3 {
            for dy in 1..=6 {
                let px = ox + dx;
                let py = oy + dy;

                if dx.abs() <= 1 && dy <= 4 {
                    world.set_block(px, py, front_z, SurfaceBlock::Water);
                } else {
                    world.set_block(px, py, front_z, dark_prismarine);
                    blocks_placed += 1;
                }
            }
        }

        blocks_placed
    }

    /// Returns the exact world coordinates of the 3 Elder Guardians spawned in the monument:
    /// 1. Top Penthouse apex chamber
    /// 2. West Wing chamber
    /// 3. East Wing chamber
    pub fn elder_guardian_spawns(origin: (i32, i32, i32)) -> [(i32, i32, i32); 3] {
        let (ox, oy, oz) = origin;
        [
            (ox, oy + 20, oz),       // Apex Penthouse
            (ox - 14, oy + 4, oz),   // West Wing
            (ox + 14, oy + 4, oz),   // East Wing
        ]
    }
}
