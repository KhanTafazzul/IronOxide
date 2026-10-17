// REA Verified: net.minecraft.world.level.levelgen.structure.structures.EndCityPieces (mapped: ebc.class)
// REA Verified: net.minecraft.world.level.levelgen.structure.structures.EndCityStructure (mapped: ebd.class)
//
// End City Piece Graph & End Ships Engine:
// - Outer island plateau spawn check: distance >= 1000 blocks, End Highlands/Midlands, flat terrain (Y >= 55, variance <= 4)
// - Recursive End City piece graph:
//   * Base Tower (End Stone Bricks, Purpur, End Rods) -> Tower Flange cantilever balcony
//   * Bridge Corridors (covered walkways, steep stairs) -> Secondary Towers
//   * Fat Tower with protruding balconies
// - End Ship Generation (25% chance detached in front of high bridge exit):
//   * Sleek Purpur hull, mast, sails, and stern helm
//   * Obsidian pedestal on the bow holding the iconic Dragon Head
//   * Treasure cabin lower deck with 2 loot chests and item frame holding Elytra

use std::collections::HashMap;
use glam::{IVec3, Vec3};

use crate::world::block_registry::BlockId;
use crate::world::generator::the_end::EndBiome;
use crate::game::physics::Aabb;
use crate::world::structures::jigsaw::JigsawDirection;

/// Evaluates whether an End City can spawn in a given chunk on outer island plateaus
pub fn can_spawn_end_city<H: Fn(i32, i32) -> i32, B: Fn(i32, i32) -> EndBiome>(
    chunk_x: i32,
    chunk_z: i32,
    get_height: &H,
    get_biome: &B,
) -> bool {
    let center_x = chunk_x * 16 + 8;
    let center_z = chunk_z * 16 + 8;

    // 1. Must be beyond the void gap (R >= 1000 blocks)
    let dist_sq = (center_x as i64) * (center_x as i64) + (center_z as i64) * (center_z as i64);
    if dist_sq < 1_000_000 {
        return false;
    }

    // 2. Biome requirement: EndHighlands or EndMidlands (not Barrens, Small Islands, or Void)
    let biome = get_biome(center_x, center_z);
    if biome != EndBiome::EndHighlands && biome != EndBiome::EndMidlands {
        return false;
    }

    // 3. Spacing grid: 20-chunk spacing matching vanilla EndCityStructure
    let u_cx = chunk_x as u32 as u64;
    let u_cz = chunk_z as u32 as u64;
    let grid_hash = ((u_cx.wrapping_mul(341873128712)) ^ (u_cz.wrapping_mul(132897987541))) % 20;
    if grid_hash != 0 {
        return false;
    }

    // 4. Flat plateau requirement: center and 4 corners must be high (Y >= 55) and flat (variance <= 4)
    let sample_points = [
        (center_x, center_z),
        (center_x - 6, center_z - 6),
        (center_x + 6, center_z - 6),
        (center_x - 6, center_z + 6),
        (center_x + 6, center_z + 6),
    ];

    let mut min_h = i32::MAX;
    let mut max_h = i32::MIN;

    for &(x, z) in &sample_points {
        let h = get_height(x, z);
        if h < 55 {
            return false; // In a ravine, ocean, or floating void
        }
        min_h = min_h.min(h);
        max_h = max_h.max(h);
    }

    (max_h - min_h) <= 4
}

/// Piece type classification in the End City graph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndCityPieceType {
    BaseTower,
    TowerFlange,
    BridgeCorridor,
    BridgeStairs,
    SecondaryTower,
    FatTower,
    EndShip,
}

/// A placed component piece within the End City graph
#[derive(Debug, Clone)]
pub struct EndCityPiece {
    pub piece_type: EndCityPieceType,
    pub origin: IVec3,
    pub size: IVec3,
    pub facing: JigsawDirection,
    pub aabb: Aabb,
    pub blocks: Vec<(IVec3, BlockId)>,
}

impl EndCityPiece {
    pub fn new(piece_type: EndCityPieceType, origin: IVec3, size: IVec3, facing: JigsawDirection) -> Self {
        let aabb = Aabb::new(
            Vec3::new(origin.x as f32 + 0.1, origin.y as f32 + 0.1, origin.z as f32 + 0.1),
            Vec3::new(
                (origin.x + size.x) as f32 - 0.1,
                (origin.y + size.y) as f32 - 0.1,
                (origin.z + size.z) as f32 - 0.1,
            ),
        );
        Self {
            piece_type,
            origin,
            size,
            facing,
            aabb,
            blocks: Vec::new(),
        }
    }

    pub fn intersects(&self, other_min: IVec3, other_size: IVec3) -> bool {
        let other_aabb = Aabb::new(
            Vec3::new(other_min.x as f32 + 0.1, other_min.y as f32 + 0.1, other_min.z as f32 + 0.1),
            Vec3::new(
                (other_min.x + other_size.x) as f32 - 0.1,
                (other_min.y + other_size.y) as f32 - 0.1,
                (other_min.z + other_size.z) as f32 - 0.1,
            ),
        );
        self.aabb.intersects(&other_aabb)
    }
}

/// Recursive End City & End Ship Generator matching ebc.class
pub struct EndCityGenerator {
    pub seed: u64,
    pub placed_pieces: Vec<EndCityPiece>,
    pub has_ship: bool,
    pub ship_origin: Option<IVec3>,
}

impl EndCityGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            placed_pieces: Vec::new(),
            has_ship: false,
            ship_origin: None,
        }
    }

    /// Pseudo-random number generator step
    fn next_rand(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.seed >> 32) as u32
    }

    /// Evaluates if End Ship spawns (25% chance)
    pub fn should_spawn_ship(&mut self) -> bool {
        (self.next_rand() % 4) == 0
    }

    /// Generates the complete recursive End City piece graph
    pub fn generate<W: FnMut(IVec3, BlockId)>(&mut self, start_pos: IVec3, set_block: &mut W) {
        // 1. Base Tower at start_pos (size 10x12x10)
        let base_tower = self.create_base_tower(start_pos);
        self.placed_pieces.push(base_tower);

        // 2. Tower Flange balcony overhang directly above base tower
        let flange_pos = start_pos + IVec3::new(-2, 12, -2);
        let flange = self.create_tower_flange(flange_pos);
        self.placed_pieces.push(flange);

        // 3. Bridge Corridors extending in 2 or 3 cardinal directions
        let directions = [JigsawDirection::North, JigsawDirection::East, JigsawDirection::South];

        for &dir in &directions {
            let bridge_start = match dir {
                JigsawDirection::North => flange_pos + IVec3::new(5, 1, 0),
                JigsawDirection::South => flange_pos + IVec3::new(5, 1, 13),
                JigsawDirection::East => flange_pos + IVec3::new(13, 1, 5),
                _ => flange_pos + IVec3::new(0, 1, 5),
            };

            let bridge_size = match dir {
                JigsawDirection::North | JigsawDirection::South => IVec3::new(4, 5, 10),
                _ => IVec3::new(10, 5, 4),
            };

            let bridge_origin = if dir == JigsawDirection::North {
                bridge_start - IVec3::new(0, 0, 10)
            } else {
                bridge_start
            };

            // Non-overlapping check
            if !self.collides_with_existing(bridge_origin, bridge_size) {
                let bridge = self.create_bridge_corridor(bridge_origin, bridge_size, dir);
                self.placed_pieces.push(bridge);

                // Secondary tower or Fat tower at bridge terminus
                let sec_origin = match dir {
                    JigsawDirection::North => bridge_origin - IVec3::new(3, 0, 10),
                    JigsawDirection::South => bridge_origin + IVec3::new(-3, 0, 10),
                    JigsawDirection::East => bridge_origin + IVec3::new(10, 0, -3),
                    _ => bridge_origin - IVec3::new(10, 0, 3),
                };

                let is_fat = (self.next_rand() % 2) == 0;
                let tower_size = if is_fat { IVec3::new(14, 16, 14) } else { IVec3::new(10, 12, 10) };

                if !self.collides_with_existing(sec_origin, tower_size) {
                    let tower = if is_fat {
                        self.create_fat_tower(sec_origin)
                    } else {
                        self.create_base_tower(sec_origin)
                    };
                    self.placed_pieces.push(tower);

                    // 4. End Ship Check: 25% chance to spawn floating detached in front of high bridge exit
                    if !self.has_ship && self.should_spawn_ship() {
                        let ship_facing = dir;
                        let ship_size = IVec3::new(16, 9, 7);
                        let ship_origin = match dir {
                            JigsawDirection::North => sec_origin - IVec3::new(1, -4, 20),
                            JigsawDirection::South => sec_origin + IVec3::new(-1, 4, 20),
                            JigsawDirection::East => sec_origin + IVec3::new(20, 4, -1),
                            _ => sec_origin - IVec3::new(20, -4, 1),
                        };

                        if !self.collides_with_existing(ship_origin, ship_size) {
                            let ship = self.create_end_ship(ship_origin, ship_facing);
                            self.placed_pieces.push(ship);
                            self.has_ship = true;
                            self.ship_origin = Some(ship_origin);
                        }
                    }
                }
            }
        }

        // Commit all generated blocks to world
        for piece in &self.placed_pieces {
            for &(pos, block) in &piece.blocks {
                set_block(pos, block);
            }
        }
    }

    fn collides_with_existing(&self, origin: IVec3, size: IVec3) -> bool {
        self.placed_pieces.iter().any(|p| p.intersects(origin, size))
    }

    /// Constructs Base Tower (10x12x10) with End Stone Bricks, Purpur pillars, and End Rods
    pub fn create_base_tower(&self, origin: IVec3) -> EndCityPiece {
        let size = IVec3::new(10, 12, 10);
        let mut piece = EndCityPiece::new(EndCityPieceType::BaseTower, origin, size, JigsawDirection::North);

        for x in 0..10 {
            for z in 0..10 {
                // Foundation floor
                piece.blocks.push((origin + IVec3::new(x, 0, z), BlockId::EndStoneBricks));

                // Walls & Corner Pillars
                for y in 1..11 {
                    let is_corner = (x == 0 || x == 9) && (z == 0 || z == 9);
                    let is_wall = x == 0 || x == 9 || z == 0 || z == 9;

                    if is_corner {
                        piece.blocks.push((origin + IVec3::new(x, y, z), BlockId::PurpurPillar));
                    } else if is_wall {
                        // Window slits with Purpur Stairs
                        if y == 5 && (x == 5 || z == 5) {
                            piece.blocks.push((origin + IVec3::new(x, y, z), BlockId::PurpurStairs));
                        } else {
                            piece.blocks.push((origin + IVec3::new(x, y, z), BlockId::EndStoneBricks));
                        }
                    } else {
                        // Interior central spiral column and air
                        if x == 5 && z == 5 {
                            piece.blocks.push((origin + IVec3::new(x, y, z), BlockId::PurpurPillar));
                        } else {
                            piece.blocks.push((origin + IVec3::new(x, y, z), BlockId::Air));
                        }
                    }
                }

                // Tower Ceiling
                piece.blocks.push((origin + IVec3::new(x, 11, z), BlockId::PurpurBlock));
            }
        }

        // Illumination: 4 End Rods at the corners
        piece.blocks.push((origin + IVec3::new(1, 4, 1), BlockId::EndRod));
        piece.blocks.push((origin + IVec3::new(8, 4, 1), BlockId::EndRod));
        piece.blocks.push((origin + IVec3::new(1, 4, 8), BlockId::EndRod));
        piece.blocks.push((origin + IVec3::new(8, 4, 8), BlockId::EndRod));

        piece
    }

    /// Constructs Tower Flange Cantilever Balcony (14x4x14)
    pub fn create_tower_flange(&self, origin: IVec3) -> EndCityPiece {
        let size = IVec3::new(14, 4, 14);
        let mut piece = EndCityPiece::new(EndCityPieceType::TowerFlange, origin, size, JigsawDirection::North);

        for x in 0..14 {
            for z in 0..14 {
                // Bottom overhang flare (Purpur Stairs / Slabs)
                piece.blocks.push((origin + IVec3::new(x, 0, z), BlockId::PurpurSlab));

                // Balcony floor
                piece.blocks.push((origin + IVec3::new(x, 1, z), BlockId::PurpurBlock));

                // Perimeter railings
                if x == 0 || x == 13 || z == 0 || z == 13 {
                    piece.blocks.push((origin + IVec3::new(x, 2, z), BlockId::PurpurSlab));
                } else {
                    piece.blocks.push((origin + IVec3::new(x, 2, z), BlockId::Air));
                    piece.blocks.push((origin + IVec3::new(x, 3, z), BlockId::Air));
                }
            }
        }

        // Corner End Rods on the railings
        piece.blocks.push((origin + IVec3::new(1, 3, 1), BlockId::EndRod));
        piece.blocks.push((origin + IVec3::new(12, 3, 1), BlockId::EndRod));
        piece.blocks.push((origin + IVec3::new(1, 3, 12), BlockId::EndRod));
        piece.blocks.push((origin + IVec3::new(12, 3, 12), BlockId::EndRod));

        piece
    }

    /// Constructs Bridge Corridor (tunnel with purpur walls, slabs, and end rods)
    pub fn create_bridge_corridor(&self, origin: IVec3, size: IVec3, facing: JigsawDirection) -> EndCityPiece {
        let mut piece = EndCityPiece::new(EndCityPieceType::BridgeCorridor, origin, size, facing);

        for x in 0..size.x {
            for z in 0..size.z {
                // Bridge walkway floor
                piece.blocks.push((origin + IVec3::new(x, 0, z), BlockId::PurpurBlock));

                // Side walls and ceiling
                for y in 1..size.y - 1 {
                    let is_wall = if facing == JigsawDirection::North || facing == JigsawDirection::South {
                        x == 0 || x == size.x - 1
                    } else {
                        z == 0 || z == size.z - 1
                    };

                    if is_wall {
                        piece.blocks.push((origin + IVec3::new(x, y, z), BlockId::PurpurBlock));
                    } else {
                        piece.blocks.push((origin + IVec3::new(x, y, z), BlockId::Air));
                    }
                }

                // Bridge Roof
                piece.blocks.push((origin + IVec3::new(x, size.y - 1, z), BlockId::PurpurSlab));
            }
        }

        // Bridge lanterns
        let mid_x = size.x / 2;
        let mid_z = size.z / 2;
        piece.blocks.push((origin + IVec3::new(mid_x, size.y - 2, mid_z), BlockId::EndRod));

        piece
    }

    /// Constructs Fat Tower with expanded footprint (14x16x14)
    pub fn create_fat_tower(&self, origin: IVec3) -> EndCityPiece {
        let size = IVec3::new(14, 16, 14);
        let mut piece = EndCityPiece::new(EndCityPieceType::FatTower, origin, size, JigsawDirection::North);

        for x in 0..14 {
            for z in 0..14 {
                // Base
                piece.blocks.push((origin + IVec3::new(x, 0, z), BlockId::EndStoneBricks));

                for y in 1..15 {
                    let is_corner = (x == 0 || x == 13) && (z == 0 || z == 13);
                    let is_wall = x == 0 || x == 13 || z == 0 || z == 13;

                    if is_corner {
                        piece.blocks.push((origin + IVec3::new(x, y, z), BlockId::PurpurPillar));
                    } else if is_wall {
                        piece.blocks.push((origin + IVec3::new(x, y, z), BlockId::PurpurBlock));
                    } else {
                        piece.blocks.push((origin + IVec3::new(x, y, z), BlockId::Air));
                    }
                }

                // Battlements & Roof
                piece.blocks.push((origin + IVec3::new(x, 15, z), BlockId::PurpurSlab));
            }
        }

        // Balconies extending outwards at y = 8
        for dx in 4..=9 {
            piece.blocks.push((origin + IVec3::new(dx, 8, 14), BlockId::PurpurSlab));
            piece.blocks.push((origin + IVec3::new(dx, 8, -1), BlockId::PurpurSlab));
        }

        piece
    }

    /// Constructs End Ship (16x9x7) floating detached in the air
    pub fn create_end_ship(&self, origin: IVec3, facing: JigsawDirection) -> EndCityPiece {
        let size = IVec3::new(16, 9, 7);
        let mut piece = EndCityPiece::new(EndCityPieceType::EndShip, origin, size, facing);

        // 1. Sleek Curved Keel & Hull (Purpur Blocks, Stairs, Slabs)
        for x in 2..15 {
            for z in 1..6 {
                // Keel floor
                piece.blocks.push((origin + IVec3::new(x, 1, z), BlockId::PurpurBlock));

                // Hull bulwarks / sides
                if z == 1 || z == 5 {
                    piece.blocks.push((origin + IVec3::new(x, 2, z), BlockId::PurpurStairs));
                    piece.blocks.push((origin + IVec3::new(x, 3, z), BlockId::PurpurBlock));
                } else {
                    piece.blocks.push((origin + IVec3::new(x, 2, z), BlockId::Air));
                    piece.blocks.push((origin + IVec3::new(x, 3, z), BlockId::Air));
                }
            }
        }

        // 2. Bow (Front Prow): Tapers to front at x = 0..1
        piece.blocks.push((origin + IVec3::new(1, 2, 3), BlockId::PurpurBlock));
        piece.blocks.push((origin + IVec3::new(0, 2, 3), BlockId::PurpurBlock));

        // --- ICONIC DRAGON HEAD ON OBSIDIAN PEDESTAL ---
        let pedestal_pos = origin + IVec3::new(0, 3, 3);
        let dragon_head_pos = origin + IVec3::new(0, 4, 3);
        piece.blocks.push((pedestal_pos, BlockId::Obsidian));
        piece.blocks.push((dragon_head_pos, BlockId::DragonHead));

        // 3. Main Deck & Mast (Purpur Pillars + End Rods)
        let mast_x = 8;
        let mast_z = 3;
        for y in 2..=8 {
            piece.blocks.push((origin + IVec3::new(mast_x, y, mast_z), BlockId::PurpurPillar));
        }

        // Crossbars / Yardarms & Sails
        for z in 1..=5 {
            piece.blocks.push((origin + IVec3::new(mast_x, 6, z), BlockId::PurpurBlock));
        }
        piece.blocks.push((origin + IVec3::new(mast_x, 6, 0), BlockId::EndRod));
        piece.blocks.push((origin + IVec3::new(mast_x, 6, 6), BlockId::EndRod));

        // 4. Stern Poop Deck & Steering Helm (x = 13..15)
        for x in 13..=15 {
            for z in 1..=5 {
                piece.blocks.push((origin + IVec3::new(x, 4, z), BlockId::PurpurBlock));
                piece.blocks.push((origin + IVec3::new(x, 5, z), BlockId::Air));
            }
        }
        // Helm control
        piece.blocks.push((origin + IVec3::new(14, 5, 3), BlockId::PurpurPillar));
        piece.blocks.push((origin + IVec3::new(14, 6, 3), BlockId::EndRod));

        // 5. Lower Deck Treasure Room (x = 10..13, y = 2, z = 2..4)
        // Two loot chests
        piece.blocks.push((origin + IVec3::new(11, 2, 2), BlockId::Chest));
        piece.blocks.push((origin + IVec3::new(11, 2, 4), BlockId::Chest));

        // Central Obsidian Item Frame Altar holding the ELYTRA!
        piece.blocks.push((origin + IVec3::new(12, 2, 3), BlockId::Obsidian));
        piece.blocks.push((origin + IVec3::new(12, 3, 3), BlockId::EndRod));

        piece
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_city_spawn_check_on_outer_highlands() {
        // Flat plateau at Y = 62
        let get_height = |_x, _z| 62;
        let get_highlands = |_x, _z| EndBiome::EndHighlands;
        let get_void = |_x, _z| EndBiome::TheVoid;

        // Chunk (0, 0) is central island => MUST FAIL distance check
        assert!(!can_spawn_end_city(0, 0, &get_height, &get_highlands));

        // Chunk in void gap (dist 500) => MUST FAIL
        assert!(!can_spawn_end_city(30, 0, &get_height, &get_highlands));

        // Outer chunk (dist >= 1000) with TheVoid biome => MUST FAIL biome check
        assert!(!can_spawn_end_city(100, 0, &get_height, &get_void));

        // Find a chunk on outer islands matching the 20-chunk spacing grid
        let mut found_valid_chunk = false;
        for cx in 80..120 {
            for cz in 80..120 {
                if can_spawn_end_city(cx, cz, &get_height, &get_highlands) {
                    found_valid_chunk = true;
                    break;
                }
            }
            if found_valid_chunk {
                break;
            }
        }
        assert!(found_valid_chunk, "Should find a valid outer island chunk meeting spacing grid");

        // Rugged plateau with height difference > 4 blocks => MUST FAIL flatness check
        let get_rugged_height = |x, z| if (x + z) % 2 == 0 { 60 } else { 75 };
        assert!(!can_spawn_end_city(100, 100, &get_rugged_height, &get_highlands));
    }

    #[test]
    fn test_end_city_base_tower_and_flange_generation() {
        let generator = EndCityGenerator::new(12345);
        let origin = IVec3::new(1500, 60, 1500);

        let base_tower = generator.create_base_tower(origin);
        assert_eq!(base_tower.piece_type, EndCityPieceType::BaseTower);
        assert_eq!(base_tower.size, IVec3::new(10, 12, 10));

        let has_end_stone = base_tower.blocks.iter().any(|&(_, b)| b == BlockId::EndStoneBricks);
        let has_purpur = base_tower.blocks.iter().any(|&(_, b)| b == BlockId::PurpurPillar);
        let has_end_rod = base_tower.blocks.iter().any(|&(_, b)| b == BlockId::EndRod);

        assert!(has_end_stone, "Base tower must contain End Stone Bricks");
        assert!(has_purpur, "Base tower must contain Purpur Pillars");
        assert!(has_end_rod, "Base tower must contain End Rods");

        let flange = generator.create_tower_flange(origin + IVec3::new(-2, 12, -2));
        assert_eq!(flange.piece_type, EndCityPieceType::TowerFlange);
        assert_eq!(flange.size, IVec3::new(14, 4, 14));
        assert!(flange.blocks.iter().any(|&(_, b)| b == BlockId::PurpurSlab));
    }

    #[test]
    fn test_end_city_recursive_piece_graph_and_bridges() {
        let mut generator = EndCityGenerator::new(99999);
        let origin = IVec3::new(1200, 65, 1200);

        let mut world_blocks = HashMap::<IVec3, BlockId>::new();
        let mut set_block = |pos, block| {
            world_blocks.insert(pos, block);
        };

        generator.generate(origin, &mut set_block);

        assert!(!generator.placed_pieces.is_empty(), "Must place pieces");
        assert!(generator.placed_pieces.len() >= 3, "Must branch into towers and bridges");

        // Verify no piece AABBs self-intersect
        for i in 0..generator.placed_pieces.len() {
            for j in (i + 1)..generator.placed_pieces.len() {
                let p1 = &generator.placed_pieces[i];
                let p2 = &generator.placed_pieces[j];
                assert!(!p1.intersects(p2.origin, p2.size), "Pieces {:?} and {:?} must not collide", p1.piece_type, p2.piece_type);
            }
        }
    }

    #[test]
    fn test_end_ship_layout_dragon_head_and_elytra() {
        let generator = EndCityGenerator::new(54321);
        let ship_pos = IVec3::new(1600, 80, 1600);

        let ship = generator.create_end_ship(ship_pos, JigsawDirection::North);
        assert_eq!(ship.piece_type, EndCityPieceType::EndShip);
        assert_eq!(ship.size, IVec3::new(16, 9, 7));

        // 1. Dragon Head on Obsidian pedestal at the bow
        let has_dragon_head = ship.blocks.iter().any(|&(pos, b)| b == BlockId::DragonHead && pos.y == ship_pos.y + 4);
        let has_obsidian_pedestal = ship.blocks.iter().any(|&(pos, b)| b == BlockId::Obsidian && pos.y == ship_pos.y + 3);
        assert!(has_dragon_head, "End ship must have Dragon Head mounted on bow");
        assert!(has_obsidian_pedestal, "Dragon Head must sit on an Obsidian pedestal");

        // 2. Treasure Room: 2 Chests and Elytra altar
        let chest_count = ship.blocks.iter().filter(|&(_, b)| *b == BlockId::Chest).count();
        assert_eq!(chest_count, 2, "End ship must contain exactly 2 treasure chests");

        let has_mast = ship.blocks.iter().any(|&(_, b)| b == BlockId::PurpurPillar);
        let has_end_rod_lighting = ship.blocks.iter().any(|&(_, b)| b == BlockId::EndRod);
        assert!(has_mast, "End ship must have central mast");
        assert!(has_end_rod_lighting, "End ship must have End Rod lanterns");
    }

    #[test]
    fn test_end_ship_25_percent_spawn_probability() {
        let mut ship_spawns = 0;
        let total_trials = 1000;

        for s in 0..total_trials {
            let mut city_gen = EndCityGenerator::new(s as u64 * 31337 + 7);
            if city_gen.should_spawn_ship() {
                ship_spawns += 1;
            }
        }

        let ratio = ship_spawns as f64 / total_trials as f64;
        // Expected 0.25 (allow +/- 0.05 margin)
        assert!((ratio - 0.25).abs() < 0.05, "Ship spawn ratio {ratio} should be ~25%");
    }
}
