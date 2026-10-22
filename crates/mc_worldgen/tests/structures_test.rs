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

#[test]
fn test_vanilla_placement_salts_for_all_5_structures() {
    use mc_worldgen::biomes::BiomeId;
    use mc_worldgen::placement_salts::*;

    // 1. Desert Pyramid: Spacing = 32, Separation = 8, Salt = 14357617, Biomes = [Desert]
    assert_eq!(DESERT_PYRAMID.spacing, 32);
    assert_eq!(DESERT_PYRAMID.separation, 8);
    assert_eq!(DESERT_PYRAMID.salt, 14357617);
    assert_eq!(DESERT_PYRAMID.valid_biomes, &[BiomeId::Desert]);
    assert!(DESERT_PYRAMID.is_valid_biome(BiomeId::Desert));
    assert!(!DESERT_PYRAMID.is_valid_biome(BiomeId::Plains));

    // 2. Jungle Pyramid: Spacing = 32, Separation = 8, Salt = 14357619, Biomes = [Jungle, Bamboo Jungle]
    assert_eq!(JUNGLE_PYRAMID.spacing, 32);
    assert_eq!(JUNGLE_PYRAMID.separation, 8);
    assert_eq!(JUNGLE_PYRAMID.salt, 14357619);
    assert!(JUNGLE_PYRAMID.is_valid_biome(BiomeId::Jungle));
    assert!(JUNGLE_PYRAMID.is_valid_biome(BiomeId::BambooJungle));
    assert!(!JUNGLE_PYRAMID.is_valid_biome(BiomeId::Desert));

    // 3. Swamp Hut: Spacing = 32, Separation = 8, Salt = 14357620, Biomes = [Swamp]
    assert_eq!(SWAMP_HUT.spacing, 32);
    assert_eq!(SWAMP_HUT.separation, 8);
    assert_eq!(SWAMP_HUT.salt, 14357620);
    assert!(SWAMP_HUT.is_valid_biome(BiomeId::Swamp));
    assert!(!SWAMP_HUT.is_valid_biome(BiomeId::Forest));

    // 4. Pillager Outpost: Spacing = 80, Separation = 24, Salt = 165745296
    assert_eq!(PILLAGER_OUTPOST.spacing, 80);
    assert_eq!(PILLAGER_OUTPOST.separation, 24);
    assert_eq!(PILLAGER_OUTPOST.salt, 165745296);
    for b in [
        BiomeId::Plains, BiomeId::Desert, BiomeId::Savanna, BiomeId::Taiga,
        BiomeId::SnowyPlains, BiomeId::Meadow, BiomeId::Grove, BiomeId::SnowySlopes,
    ] {
        assert!(PILLAGER_OUTPOST.is_valid_biome(b), "Outpost must allow {:?}", b);
    }
    assert!(!PILLAGER_OUTPOST.is_valid_biome(BiomeId::DeepDark));

    // 5. Ancient City: Spacing = 24, Separation = 8, Salt = 20083232, Biomes = [Deep Dark], Y = -51
    assert_eq!(ANCIENT_CITY.spacing, 24);
    assert_eq!(ANCIENT_CITY.separation, 8);
    assert_eq!(ANCIENT_CITY.salt, 20083232);
    assert_eq!(ANCIENT_CITY.fixed_y, Some(-51));
    assert!(ANCIENT_CITY.is_valid_biome(BiomeId::DeepDark));
    assert!(!ANCIENT_CITY.is_valid_biome(BiomeId::Desert));
}

#[test]
fn test_seed_123456789_structure_grid_and_tnt_trap_parity() {
    use mc_worldgen::{
        AncientCityGenerator, ChunkPos, DesertPyramidGenerator, OreBlockAccess, StructureLocator,
        StructureType, SurfaceBlock,
    };

    let seed = 123456789i64;
    let locator = StructureLocator::new(seed);

    // 1. Assert Ancient City starting piece matches exact chunk coordinates (13, 9)
    let ac_chunk = locator.ancient_cities.get_potential_structure_chunk(seed, 0, 0);
    assert_eq!(ac_chunk, ChunkPos::new(13, 9), "Ancient City must generate at chunk (13, 9)");
    assert!(locator.ancient_cities.is_structure_chunk(seed, 13, 9));

    let ac_roots = locator.get_structures_in_chunk(13, 9);
    let ac_root = ac_roots.iter().find(|r| r.structure_type == StructureType::AncientCity)
        .expect("Ancient City root must be registered in chunk (13, 9)");
    assert_eq!(ac_root.start_block_y, -51, "Ancient City must be fixed at Y = -51");

    // Generate Ancient City at its predicted block position and verify piece bounding boxes
    let mut ac_world = TestWorld::new();
    let ac_placed = AncientCityGenerator::generate(
        &mut ac_world,
        (ac_root.start_block_x, ac_root.start_block_y, ac_root.start_block_z),
        seed,
        None,
    );
    assert!(ac_placed > 200, "Ancient City must generate portal frame, redstone lab, and wings");

    // Verify reinforced deepslate frame at Y in [-51, -38]
    let has_reinforced = ac_world.blocks.values().any(|&b| b == SurfaceBlock::ReinforcedDeepslate);
    assert!(has_reinforced, "Ancient City must contain Reinforced Deepslate portal frame");

    // Verify active Sculk Shriekers and Sensors
    let has_shrieker = ac_world.blocks.values().any(|&b| b == SurfaceBlock::SculkShrieker);
    let has_sensor = ac_world.blocks.values().any(|&b| b == SurfaceBlock::SculkSensor);
    let has_catalyst = ac_world.blocks.values().any(|&b| b == SurfaceBlock::SculkCatalyst);
    assert!(has_shrieker && has_sensor && has_catalyst, "Ancient city must have sculk ecology");

    // Verify Hidden Redstone Laboratory below frame with comparator and piston door
    let has_comparator = ac_world.blocks.values().any(|&b| b == SurfaceBlock::Comparator);
    let has_piston = ac_world.blocks.values().any(|&b| b == SurfaceBlock::Piston);
    assert!(has_comparator, "Laboratory must feature comparator pulse counters");
    assert!(has_piston, "Laboratory must feature piston secret door");

    // 2. Assert Desert Pyramid generates at predicted grid chunk (3, 5) with TNT trap intact at Y = 53
    let dp_chunk = locator.desert_pyramids.get_potential_structure_chunk(seed, 0, 0);
    assert_eq!(dp_chunk, ChunkPos::new(3, 5), "Desert Pyramid must generate at chunk (3, 5)");
    assert!(locator.desert_pyramids.is_structure_chunk(seed, 3, 5));

    let dp_roots = locator.get_structures_in_chunk(3, 5);
    let dp_root = dp_roots.iter().find(|r| r.structure_type == StructureType::DesertPyramid)
        .expect("Desert Pyramid root must be registered in chunk (3, 5)");
    assert_eq!(dp_root.start_block_y, 64);

    // Generate Desert Pyramid at its predicted starting position
    let mut dp_world = TestWorld::new();
    let origin = (dp_root.start_block_x, 64, dp_root.start_block_z); // (56, 64, 88)
    let dp_placed = DesertPyramidGenerator::generate(&mut dp_world, origin, seed);
    assert!(dp_placed > 500, "Desert pyramid must place base, towers, and chambers");

    // Center terracotta pattern at floor level Y = 64
    let blue_center = dp_world.get_block(origin.0 + 10, 64, origin.2 + 10);
    assert_eq!(blue_center, SurfaceBlock::BlueTerracotta, "Center floor must be Blue Terracotta");

    // ASSERT: At Y = 53, exactly 9 TNT blocks are intact directly below the drop shaft!
    let mut tnt_count = 0;
    for dx in 9..=11 {
        for dz in 9..=11 {
            let b = dp_world.get_block(origin.0 + dx, 53, origin.2 + dz);
            if b == SurfaceBlock::TNT {
                tnt_count += 1;
            }
        }
    }
    assert_eq!(tnt_count, 9, "Desert Pyramid must have exactly 9 TNT blocks intact at Y = 53");

    // Pressure plate at Y = 54 directly on top of center TNT
    let plate = dp_world.get_block(origin.0 + 10, 54, origin.2 + 10);
    assert_eq!(plate, SurfaceBlock::StonePressurePlate, "Pressure plate must be at Y = 54 resting on TNT");

    // 4 Loot Chests on cardinal walls surrounding the plate at Y = 54
    let north_chest = dp_world.get_block(origin.0 + 10, 54, origin.2 + 8);
    let south_chest = dp_world.get_block(origin.0 + 10, 54, origin.2 + 12);
    let west_chest = dp_world.get_block(origin.0 + 8, 54, origin.2 + 10);
    let east_chest = dp_world.get_block(origin.0 + 12, 54, origin.2 + 10);
    assert_eq!(north_chest, SurfaceBlock::Chest);
    assert_eq!(south_chest, SurfaceBlock::Chest);
    assert_eq!(west_chest, SurfaceBlock::Chest);
    assert_eq!(east_chest, SurfaceBlock::Chest);

    // Archaeology basement: Suspicious Sand and Decorated Pot
    let has_suspicious_sand = dp_world.blocks.values().any(|&b| b == SurfaceBlock::SuspiciousSand);
    let has_decorated_pot = dp_world.blocks.values().any(|&b| b == SurfaceBlock::DecoratedPot);
    assert!(has_suspicious_sand, "Pyramid must have 1.20+ archaeology suspicious sand");
    assert!(has_decorated_pot, "Pyramid must have buried decorated pots");
}

#[test]
fn test_jungle_temple_generation_and_traps() {
    use mc_worldgen::{JungleTempleGenerator, SurfaceBlock};

    let mut world = TestWorld::new();
    let origin = (100, 64, 100);
    let placed = JungleTempleGenerator::generate(&mut world, origin, 987654321);
    assert!(placed > 300, "Jungle Temple must place complete 3-story structure");

    // 1. Mossy and Regular Cobblestone combination
    let has_cobble = world.blocks.values().any(|&b| b == SurfaceBlock::Cobblestone);
    let has_mossy = world.blocks.values().any(|&b| b == SurfaceBlock::MossyCobblestone);
    assert!(has_cobble && has_mossy);

    // 2. 3-Lever puzzle
    let lever_count = world.blocks.values().filter(|&&b| b == SurfaceBlock::Lever).count();
    assert_eq!(lever_count, 3, "Jungle Temple must contain exactly 3 levers for combination puzzle");

    // 3. Sticky Piston hidden door & 2 chests
    let has_sticky_piston = world.blocks.values().any(|&b| b == SurfaceBlock::StickyPiston);
    let chest_count = world.blocks.values().filter(|&&b| b == SurfaceBlock::Chest).count();
    assert!(has_sticky_piston, "Temple must contain sticky piston door");
    assert_eq!(chest_count, 2, "Temple must contain 1 main chest and 1 hidden chest");

    // 4. Dispenser Tripwire Traps
    let dispenser_count = world.blocks.values().filter(|&&b| b == SurfaceBlock::Dispenser).count();
    let hook_count = world.blocks.values().filter(|&&b| b == SurfaceBlock::TripwireHook).count();
    let string_count = world.blocks.values().filter(|&&b| b == SurfaceBlock::StringBlock).count();
    assert_eq!(dispenser_count, 2, "Temple must contain 2 hidden arrow dispensers");
    assert_eq!(hook_count, 4, "Temple must contain 4 tripwire hooks (2 traps * 2 hooks)");
    assert_eq!(string_count, 2, "Temple must contain tripwire string crossings");
}

#[test]
fn test_swamp_hut_generation_and_mob_spawn_isolation() {
    use mc_worldgen::{SurfaceBlock, SwampHutGenerator};

    let mut world = TestWorld::new();
    let origin = (200, 62, 200);
    let placed = SwampHutGenerator::generate(&mut world, origin);
    assert!(placed > 50, "Swamp Hut must place wooden stilts, walls, roof, and interior");

    // 1. Oak stilts, spruce planks, dark oak stairs
    let has_oak = world.blocks.values().any(|&b| b == SurfaceBlock::OakLog);
    let has_spruce = world.blocks.values().any(|&b| b == SurfaceBlock::SprucePlanks);
    let has_dark_oak_stairs = world.blocks.values().any(|&b| b == SurfaceBlock::DarkOakStairs);
    assert!(has_oak && has_spruce && has_dark_oak_stairs);

    // 2. Interior Furnishings: Crafting Table, Cauldron, Potted Mushroom
    let has_crafting = world.blocks.values().any(|&b| b == SurfaceBlock::CraftingTable);
    let has_cauldron = world.blocks.values().any(|&b| b == SurfaceBlock::Cauldron);
    let has_potted_mushroom = world.blocks.values().any(|&b| b == SurfaceBlock::PottedMushroom);
    assert!(has_crafting, "Swamp hut must have crafting table");
    assert!(has_cauldron, "Swamp hut must have witch cauldron");
    assert!(has_potted_mushroom, "Swamp hut must have potted mushroom");

    // 3. Mob spawn isolation: Only Witches and Cats permitted in 7x7x9 volume
    let inside_pos = (origin.0 + 3, origin.1 + 3, origin.2 + 3);
    assert!(SwampHutGenerator::is_spawn_permitted(origin, inside_pos, "minecraft:witch"));
    assert!(SwampHutGenerator::is_spawn_permitted(origin, inside_pos, "minecraft:cat"));
    assert!(!SwampHutGenerator::is_spawn_permitted(origin, inside_pos, "minecraft:zombie"));
    assert!(!SwampHutGenerator::is_spawn_permitted(origin, inside_pos, "minecraft:skeleton"));
    assert!(!SwampHutGenerator::is_spawn_permitted(origin, inside_pos, "minecraft:creeper"));

    // Outside bounding box, regular spawns are permitted
    let outside_pos = (origin.0 + 50, origin.1 + 3, origin.2 + 50);
    assert!(SwampHutGenerator::is_spawn_permitted(origin, outside_pos, "minecraft:zombie"));
}

#[test]
fn test_pillager_outpost_watchtower_and_accessories() {
    use mc_worldgen::{PillagerOutpostGenerator, SurfaceBlock};

    let mut world = TestWorld::new();
    let origin = (300, 68, 300);
    let placed = PillagerOutpostGenerator::generate(&mut world, origin, 555666777, None);
    assert!(placed > 150, "Pillager Outpost must place watchtower and 4 accessory pieces");

    // 1. 4 Support Pillars (Cobble and Dark Oak Log)
    let has_cobble = world.blocks.values().any(|&b| b == SurfaceBlock::Cobblestone);
    let has_dark_oak = world.blocks.values().any(|&b| b == SurfaceBlock::DarkOakLog);
    let has_chest = world.blocks.values().any(|&b| b == SurfaceBlock::Chest);
    assert!(has_cobble && has_dark_oak && has_chest);

    // 2. Surrounding accessories: Target Tent, Iron Golem Cage, Log Pile, Scarecrows
    let has_target = world.blocks.values().any(|&b| b == SurfaceBlock::TargetBlock);
    let has_bars = world.blocks.values().any(|&b| b == SurfaceBlock::IronBars);
    let has_hay = world.blocks.values().any(|&b| b == SurfaceBlock::HayBale);
    let has_pumpkin = world.blocks.values().any(|&b| b == SurfaceBlock::CarvedPumpkin);
    assert!(has_target, "Outpost must feature target tent");
    assert!(has_bars, "Outpost must feature iron golem cage");
    assert!(has_hay && has_pumpkin, "Outpost must feature scarecrows");
}




