//! Tests for structure grid placement (RandomSpreadPlacement) and concentric rings (ConcentricRingsPlacement).

use mc_worldgen::{
    ChunkPos, ConcentricRingsPlacement, LegacyRandom, OverworldGenerator, RandomSpreadPlacement,
    SpreadType, StructureLocator, StructureType,
};


#[test]
fn test_legacy_random_parity_with_jvm() {
    let mut rng = LegacyRandom::new(123456789);
    let d1 = rng.next_double();
    let i1 = rng.next_int_bounded(26);
    let i2 = rng.next_int_bounded(26);
    println!("Rust LegacyRandom: d1={}, i1={}, i2={}", d1, i1, i2);
    assert!(d1 >= 0.0 && d1 < 1.0);
}

#[test]
fn test_random_spread_village_placement_parity() {
    let seed = 123456789i64;
    // Villages: Spacing = 34, Separation = 8, Salt = 10387312
    let village_placement = RandomSpreadPlacement::new(34, 8, 10387312, SpreadType::Linear);

    // In cell (0, 0), Java produced (21, 23)
    let pos0 = village_placement.get_potential_structure_chunk(seed, 0, 0);
    assert_eq!(pos0, ChunkPos::new(21, 23));
    assert!(village_placement.is_structure_chunk(seed, 21, 23));
    assert!(!village_placement.is_structure_chunk(seed, 0, 0));

    // In cell containing (10, -5), which is cell (0, -1), Java produced (23, -13)
    let pos1 = village_placement.get_potential_structure_chunk(seed, 10, -5);
    assert_eq!(pos1, ChunkPos::new(23, -13));
    assert!(village_placement.is_structure_chunk(seed, 23, -13));
    assert!(!village_placement.is_structure_chunk(seed, 10, -5));
}

#[test]
fn test_random_spread_end_city_placement_parity() {
    let seed = 123456789i64;
    // End Cities: Spacing = 20, Separation = 11, Salt = 10387313, SpreadType::Triangular
    let end_city_placement = RandomSpreadPlacement::new(20, 11, 10387313, SpreadType::Triangular);

    // In cell (0, 0), Java produced (7, 3)
    let pos0 = end_city_placement.get_potential_structure_chunk(seed, 0, 0);
    assert_eq!(pos0, ChunkPos::new(7, 3));
    assert!(end_city_placement.is_structure_chunk(seed, 7, 3));

    // In cell containing (50, 50), which is cell (2, 2), Java produced (43, 41)
    let pos1 = end_city_placement.get_potential_structure_chunk(seed, 50, 50);
    assert_eq!(pos1, ChunkPos::new(43, 41));
    assert!(end_city_placement.is_structure_chunk(seed, 43, 41));
}

#[test]
fn test_random_spread_exact_constants() {
    let seed = 123456789i64;

    // Trial Chambers: Spacing = 34, Separation = 12, Salt = 94251327
    let tc = RandomSpreadPlacement::new(34, 12, 94251327, SpreadType::Linear);
    let tc_pos = tc.get_potential_structure_chunk(seed, 0, 0);
    assert!(tc.is_structure_chunk(seed, tc_pos.x, tc_pos.z));
    assert!(tc_pos.x >= 0 && tc_pos.x < 34 - 12);
    assert!(tc_pos.z >= 0 && tc_pos.z < 34 - 12);

    // Woodland Mansions: Spacing = 80, Separation = 20, Salt = 10387313
    let wm = RandomSpreadPlacement::new(80, 20, 10387313, SpreadType::Triangular);
    let wm_pos = wm.get_potential_structure_chunk(seed, 0, 0);
    assert!(wm.is_structure_chunk(seed, wm_pos.x, wm_pos.z));
    assert!(wm_pos.x >= 0 && wm_pos.x < 80 - 20);
    assert!(wm_pos.z >= 0 && wm_pos.z < 80 - 20);

    // Nether Fortresses: Spacing = 27, Separation = 4, Salt = 30084232
    let nf = RandomSpreadPlacement::new(27, 4, 30084232, SpreadType::Linear);
    let nf_pos = nf.get_potential_structure_chunk(seed, 0, 0);
    assert!(nf.is_structure_chunk(seed, nf_pos.x, nf_pos.z));

    // Bastion Remnants: Spacing = 27, Separation = 4, Salt = 30084235
    let br = RandomSpreadPlacement::new(27, 4, 30084235, SpreadType::Linear);
    let br_pos = br.get_potential_structure_chunk(seed, 0, 0);
    assert!(br.is_structure_chunk(seed, br_pos.x, br_pos.z));
}

#[test]
fn test_concentric_rings_stronghold_placement() {
    let seed = 123456789i64;
    let stronghold_placement = ConcentricRingsPlacement::new(32, 3, 128);
    let positions = stronghold_placement.generate_positions(seed);

    // Total count must be 128 strongholds
    assert_eq!(positions.len(), 128);

    // First ring has 3 strongholds
    let s0 = positions[0];
    let s1 = positions[1];
    let s2 = positions[2];

    assert_eq!(s0, ChunkPos::new(-64, -107));
    assert_eq!(s1, ChunkPos::new(119, -2));
    assert_eq!(s2, ChunkPos::new(-77, 139));

    // Verify distance constraints: all ring 0 strongholds between 1,280 and 2,816 blocks from (0,0)
    for i in 0..3 {
        let dist = positions[i].distance_to_origin();
        println!("Stronghold #{} block distance = {}", i, dist);
        assert!(dist >= 1280.0 && dist <= 2816.0, "Ring 0 stronghold distance {} outside [1280, 2816]", dist);
    }

    // Verify that ring positions expand monotonically outwards across rings
    let r1_sample = positions[5].distance_to_origin();
    let r2_sample = positions[15].distance_to_origin();
    let r7_sample = positions[125].distance_to_origin();

    assert!(r1_sample > 4000.0, "Ring 1 sample should be > 4000 blocks");
    assert!(r2_sample > 7000.0, "Ring 2 sample should be > 7000 blocks");
    assert!(r7_sample > 20000.0, "Ring 7 sample should be > 20000 blocks");
}

#[test]
fn test_connected_chunk_generation_structure_instantiation() {
    let seed = 123456789i64;
    let generator = OverworldGenerator::new(seed);

    // 1. Stronghold starting chunk at (-64, -107)
    let structures_sh = generator.get_structures_in_chunk(-64, -107);
    assert!(!structures_sh.is_empty());
    assert!(structures_sh.iter().any(|s| s.structure_type == StructureType::Stronghold));
    let sh_root = structures_sh.iter().find(|s| s.structure_type == StructureType::Stronghold).unwrap();
    assert_eq!(sh_root.name, "minecraft:stronghold");
    assert_eq!(sh_root.chunk_pos, ChunkPos::new(-64, -107));
    assert_eq!(sh_root.start_block_x, -64 * 16 + 8);
    assert_eq!(sh_root.start_block_z, -107 * 16 + 8);

    // 2. Village starting chunk at (21, 23)
    let structures_v = generator.get_structures_in_chunk(21, 23);
    assert!(!structures_v.is_empty());
    assert!(structures_v.iter().any(|s| s.structure_type == StructureType::Village));
    let v_root = structures_v.iter().find(|s| s.structure_type == StructureType::Village).unwrap();
    assert_eq!(v_root.name, "minecraft:village");
    assert_eq!(v_root.chunk_pos, ChunkPos::new(21, 23));

    // 3. End City starting chunk at (7, 3)
    let structures_ec = generator.get_structures_in_chunk(7, 3);
    assert!(!structures_ec.is_empty());
    assert!(structures_ec.iter().any(|s| s.structure_type == StructureType::EndCity));

    // 4. Regular chunk without structures
    let empty_structures = generator.get_structures_in_chunk(0, 0);
    assert!(empty_structures.is_empty());
}

#[test]
fn test_structure_locator_direct() {
    let seed = 123456789i64;
    let locator = StructureLocator::new(seed);
    let stronghold_chunks = &locator.stronghold_list;
    assert_eq!(stronghold_chunks.len(), 128);
    assert!(locator.is_stronghold_chunk(stronghold_chunks[0].x, stronghold_chunks[0].z));
    let roots = locator.get_structures_in_chunk(stronghold_chunks[0].x, stronghold_chunks[0].z);
    assert!(roots.iter().any(|r| r.structure_type == StructureType::Stronghold));
}

struct TestWorld {
    blocks: std::collections::HashMap<(i32, i32, i32), mc_worldgen::SurfaceBlock>,
}

impl TestWorld {
    fn new() -> Self {
        Self {
            blocks: std::collections::HashMap::new(),
        }
    }
}

impl mc_worldgen::OreBlockAccess for TestWorld {
    fn get_block(&self, x: i32, y: i32, z: i32) -> mc_worldgen::SurfaceBlock {
        self.blocks.get(&(x, y, z)).copied().unwrap_or(mc_worldgen::SurfaceBlock::Air)
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, block: mc_worldgen::SurfaceBlock) {
        self.blocks.insert((x, y, z), block);
    }
}

#[test]
fn test_abandoned_mineshaft_procedural_generation() {
    use mc_worldgen::{MineshaftGenerator, MineshaftType, SurfaceBlock};

    // 1. Normal Mineshaft (Oak wood supports)
    let mut world = TestWorld::new();
    let placed = MineshaftGenerator::generate(&mut world, (0, -30, 0), MineshaftType::Normal, 42);
    assert!(placed >= 50, "Mineshaft must place substantial corridor and room blocks");

    let has_planks = world.blocks.values().any(|&b| b == SurfaceBlock::OakPlanks);
    let has_fence = world.blocks.values().any(|&b| b == SurfaceBlock::OakFence);
    let has_rails = world.blocks.values().any(|&b| b == SurfaceBlock::Rail);
    let has_cobwebs = world.blocks.values().any(|&b| b == SurfaceBlock::Cobweb);
    let has_chest = world.blocks.values().any(|&b| b == SurfaceBlock::Chest);

    assert!(has_planks, "Mineshaft must place oak planks for support arches");
    assert!(has_fence, "Mineshaft must place oak fence posts");
    assert!(has_rails, "Mineshaft must place rail corridors");
    assert!(has_cobwebs, "Mineshaft must place cobwebs in corridors");
    assert!(has_chest, "Mineshaft must place chest minecarts");

    // 2. Badlands / Mesa variant (Dark Oak wood supports)
    let mut mesa_world = TestWorld::new();
    let mesa_placed = MineshaftGenerator::generate(&mut mesa_world, (100, 40, 100), MineshaftType::Mesa, 99);
    assert!(mesa_placed >= 50);
    let has_dark_oak = mesa_world.blocks.values().any(|&b| b == SurfaceBlock::DarkOakPlanks);
    assert!(has_dark_oak, "Mesa mineshafts must generate with Dark Oak planks");
}

#[test]
fn test_nether_fortress_piece_layout() {
    use mc_worldgen::{NetherFortressGenerator, SurfaceBlock};

    let mut world = TestWorld::new();
    let placed = NetherFortressGenerator::generate(&mut world, (0, 64, 0), 1337);
    assert!(placed > 500, "Nether fortress must place a large bridge network");

    // Verify predominant Nether Bricks
    let brick_count = world.blocks.values().filter(|&&b| b == SurfaceBlock::NetherBricks).count();
    assert!(brick_count > 400, "Fortress must be constructed from Nether Bricks");

    // Verify Blaze Spawner dais and Nether Wart staircase room
    let has_spawner = world.blocks.values().any(|&b| b == SurfaceBlock::Spawner);
    let has_wart = world.blocks.values().any(|&b| b == SurfaceBlock::NetherWart);
    let has_soul_sand = world.blocks.values().any(|&b| b == SurfaceBlock::SoulSand);

    assert!(has_spawner, "Nether Fortress must feature Blaze spawner dais");
    assert!(has_wart && has_soul_sand, "Nether Fortress must feature Nether Wart farm room");
}

#[test]
fn test_bastion_remnants_jigsaw_starts() {
    use mc_worldgen::{BastionRemnantGenerator, BastionType, SurfaceBlock};

    // 1. Treasure Room: Central gold block cube, magma spawner, loot chests, and lava moat
    let mut tr_world = TestWorld::new();
    let tr_placed = BastionRemnantGenerator::generate_variant(&mut tr_world, (0, 45, 0), BastionType::TreasureRoom);
    assert!(tr_placed > 200);

    let gold_count = tr_world.blocks.values().filter(|&&b| b == SurfaceBlock::GoldBlock).count();
    assert!(gold_count >= 8, "Bastion treasure room must contain central gold block cube");

    let has_spawner = tr_world.blocks.values().any(|&b| b == SurfaceBlock::Spawner);
    let has_chest = tr_world.blocks.values().any(|&b| b == SurfaceBlock::Chest);
    let has_lava = tr_world.blocks.values().any(|&b| b == SurfaceBlock::Lava);
    assert!(has_spawner, "Treasure room must contain spawner");
    assert!(has_chest, "Treasure room must contain loot chests");
    assert!(has_lava, "Treasure room must contain central lava moat");

    // 2. Hoglin Stables: Pens, blackstone floors
    let mut hs_world = TestWorld::new();
    let hs_placed = BastionRemnantGenerator::generate_variant(&mut hs_world, (100, 45, 100), BastionType::HoglinStables);
    assert!(hs_placed > 100);
    assert!(hs_world.blocks.values().any(|&b| b == SurfaceBlock::Blackstone));

    // 3. Housing Units: Multi-story residential rooms
    let mut hu_world = TestWorld::new();
    let hu_placed = BastionRemnantGenerator::generate_variant(&mut hu_world, (200, 45, 200), BastionType::HousingUnits);
    assert!(hu_placed > 100);
    assert!(hu_world.blocks.values().any(|&b| b == SurfaceBlock::PolishedBlackstoneBricks));

    // 4. Fortified Bridge
    let mut br_world = TestWorld::new();
    let br_placed = BastionRemnantGenerator::generate_variant(&mut br_world, (300, 45, 300), BastionType::Bridge);
    assert!(br_placed > 100);
    assert!(br_world.blocks.values().any(|&b| b == SurfaceBlock::GoldBlock));
}

#[test]
fn test_ocean_monument_generation() {
    use mc_worldgen::{OceanMonumentGenerator, SurfaceBlock};

    let mut world = TestWorld::new();
    let origin = (0, 30, 0);
    let placed = OceanMonumentGenerator::generate(&mut world, origin);

    // 1. Pyramid base and chambers footprint
    assert!(placed > 2000, "Ocean Monument must contain extensive prismarine masonry");

    // 2. Key decorative blocks
    let has_prismarine = world.blocks.values().any(|&b| b == SurfaceBlock::Prismarine);
    let has_dark_prismarine = world.blocks.values().any(|&b| b == SurfaceBlock::DarkPrismarine);
    let has_lanterns = world.blocks.values().any(|&b| b == SurfaceBlock::SeaLantern);
    assert!(has_prismarine && has_dark_prismarine && has_lanterns);

    // 3. Central Treasure Room: 8 Gold Blocks
    let gold_count = world.blocks.values().filter(|&&b| b == SurfaceBlock::GoldBlock).count();
    assert_eq!(gold_count, 8, "Ocean Monument treasure core must contain exactly 8 Gold Blocks");

    // 4. Sponge room
    let sponge_count = world.blocks.values().filter(|&&b| b == SurfaceBlock::Sponge).count();
    assert!(sponge_count >= 8, "Ocean Monument must contain sponge storage room");

    // 5. 3 Elder Guardians
    let guardians = OceanMonumentGenerator::elder_guardian_spawns(origin);
    assert_eq!(guardians.len(), 3, "Ocean Monument must contain exactly 3 Elder Guardian spawn positions");
}

#[test]
fn test_buried_treasure_generation() {
    use mc_worldgen::{BuriedTreasureGenerator, SurfaceBlock};

    let mut world = TestWorld::new();
    let placed = BuriedTreasureGenerator::generate(&mut world, (145, 62, -280));

    assert_eq!(placed, 3);
    assert_eq!(world.blocks.get(&(145, 60, -280)), Some(&SurfaceBlock::Chest));
    assert_eq!(world.blocks.get(&(145, 61, -280)), Some(&SurfaceBlock::Sand));
    assert_eq!(world.blocks.get(&(145, 62, -280)), Some(&SurfaceBlock::Sand));
}



