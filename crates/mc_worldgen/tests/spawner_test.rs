//! Comprehensive test suite for Task 55: Natural Chunk Spawner & Mob Caps.
//! REA Verified: net.minecraft.world.level.NaturalSpawner (mapped: cua.class)
//! REA Verified: net.minecraft.world.entity.MobCategory (mapped: bmo.class)
//! REA Verified: net.minecraft.world.level.biome.MobSpawnSettings (mapped: cva.class)

use glam::Vec3;
use mc_entities::EntityType;
use mc_worldgen::biomes::SurfaceBlock;
use mc_worldgen::spawning::*;
use std::collections::HashMap;

/// Mock test world environment implementing `SpawnEnvironment`.
#[derive(Clone, Debug)]
struct MockWorld {
    blocks: HashMap<(i32, i32, i32), SurfaceBlock>,
    block_lights: HashMap<(i32, i32, i32), u8>,
    sky_lights: HashMap<(i32, i32, i32), u8>,
    is_day: bool,
    top_y: HashMap<(i32, i32), i32>,
}

impl MockWorld {
    fn new(is_day: bool) -> Self {
        Self {
            blocks: HashMap::new(),
            block_lights: HashMap::new(),
            sky_lights: HashMap::new(),
            is_day,
            top_y: HashMap::new(),
        }
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, block: SurfaceBlock) {
        self.blocks.insert((x, y, z), block);
        let cur = self.top_y.entry((x, z)).or_insert(y);
        if y > *cur {
            *cur = y;
        }
    }

    fn set_light(&mut self, x: i32, y: i32, z: i32, block_light: u8, sky_light: u8) {
        self.block_lights.insert((x, y, z), block_light);
        self.sky_lights.insert((x, y, z), sky_light);
    }
}

impl SpawnEnvironment for MockWorld {
    fn get_block(&self, x: i32, y: i32, z: i32) -> SurfaceBlock {
        *self.blocks.get(&(x, y, z)).unwrap_or(&SurfaceBlock::Air)
    }

    fn get_block_light(&self, x: i32, y: i32, z: i32) -> u8 {
        *self.block_lights.get(&(x, y, z)).unwrap_or(&0)
    }

    fn get_sky_light(&self, x: i32, y: i32, z: i32) -> u8 {
        *self.sky_lights.get(&(x, y, z)).unwrap_or(&15)
    }

    fn is_daytime(&self) -> bool {
        self.is_day
    }

    fn get_top_motion_blocking_y(&self, x: i32, z: i32) -> i32 {
        *self.top_y.get(&(x, z)).unwrap_or(&63)
    }
}

#[test]
fn test_mob_category_constants_and_caps() {
    // 1. Base caps matching official bytecode: bmo.<init>
    assert_eq!(MobCategory::Monster.base_cap(), 70);
    assert_eq!(MobCategory::Creature.base_cap(), 10);
    assert_eq!(MobCategory::Ambient.base_cap(), 15);
    assert_eq!(MobCategory::WaterCreature.base_cap(), 5);
    assert_eq!(MobCategory::WaterAmbient.base_cap(), 20);
    assert_eq!(MobCategory::UndergroundWaterCreature.base_cap(), 5);
    assert_eq!(MobCategory::Axolotls.base_cap(), 5);
    assert_eq!(MobCategory::Misc.base_cap(), -1);

    // 2. Dynamic scaling with loaded chunks: formula floor(base * chunks / 289)
    assert_eq!(MobCategoryTracker::calculate_mob_cap(MobCategory::Monster, 289), 70);
    assert_eq!(MobCategoryTracker::calculate_mob_cap(MobCategory::Creature, 289), 10);
    assert_eq!(MobCategoryTracker::calculate_mob_cap(MobCategory::Monster, 578), 140);
    assert_eq!(MobCategoryTracker::calculate_mob_cap(MobCategory::Creature, 578), 20);

    // 3. Minimum cap is at least 1 for valid categories
    assert_eq!(MobCategoryTracker::calculate_mob_cap(MobCategory::Monster, 1), 1);
}

#[test]
fn test_category_tracker_can_spawn_and_cap_limit() {
    let mut tracker = MobCategoryTracker::new();

    // 289 chunks -> Monster cap is 70
    assert!(tracker.can_spawn(MobCategory::Monster, 289));
    tracker.set_count(MobCategory::Monster, 69);
    assert!(tracker.can_spawn(MobCategory::Monster, 289));

    // Reaching exactly cap (70) prevents further spawning
    tracker.increment(MobCategory::Monster);
    assert_eq!(tracker.get_count(MobCategory::Monster), 70);
    assert!(!tracker.can_spawn(MobCategory::Monster, 289));

    // Decrementing re-enables spawning
    tracker.decrement(MobCategory::Monster);
    assert_eq!(tracker.get_count(MobCategory::Monster), 69);
    assert!(tracker.can_spawn(MobCategory::Monster, 289));
}

#[test]
fn test_spawn_distances_player_safety_and_max_radius() {
    let player = Vec3::new(100.0, 64.0, 100.0);
    let players = [player];

    // Too close: exactly 10 blocks away (< 24 blocks) -> Rejected
    let too_close = Vec3::new(110.0, 64.0, 100.0);
    assert!(!is_valid_spawn_distance(too_close, &players));

    // Exactly 23.9 blocks away (< 24 blocks) -> Rejected
    let borderline_close = Vec3::new(100.0 + 23.9, 64.0, 100.0);
    assert!(!is_valid_spawn_distance(borderline_close, &players));

    // Exactly 25 blocks away (between 24 and 128) -> Valid!
    let valid_pos = Vec3::new(125.0, 64.0, 100.0);
    assert!(is_valid_spawn_distance(valid_pos, &players));

    // Beyond 128 blocks -> Rejected
    let too_far = Vec3::new(100.0 + 130.0, 64.0, 100.0);
    assert!(!is_valid_spawn_distance(too_far, &players));
}

#[test]
fn test_despawn_instant_beyond_128_blocks() {
    let player = Vec3::new(0.0, 64.0, 0.0);
    let players = [player];

    // Monster 130 blocks away -> Instant despawn!
    let far_pos = Vec3::new(130.0, 64.0, 0.0);
    assert!(should_despawn(EntityType::Zombie, far_pos, &players, 10, false, 0.99));

    // WaterAmbient has shorter 64 block despawn distance
    let fish_pos = Vec3::new(70.0, 64.0, 0.0);
    assert!(should_despawn(EntityType::Cod, fish_pos, &players, 10, false, 0.99));

    // Persistent mob (nametagged/tamed) NEVER despawns
    assert!(!should_despawn(EntityType::Zombie, far_pos, &players, 1000, true, 0.0));
}

#[test]
fn test_despawn_random_roll_between_32_and_128_blocks() {
    let player = Vec3::new(0.0, 64.0, 0.0);
    let players = [player];

    // 1. Within 32 blocks (safe zone): never naturally despawns
    let safe_pos = Vec3::new(20.0, 64.0, 0.0);
    assert!(!should_despawn(EntityType::Skeleton, safe_pos, &players, 10000, false, 0.0));

    // 2. Between 32 and 128 blocks: 50 blocks away
    let mid_pos = Vec3::new(50.0, 64.0, 0.0);

    // Younger than 600 ticks (30 seconds): does NOT roll despawn
    assert!(!should_despawn(EntityType::Skeleton, mid_pos, &players, 500, false, 0.0));

    // Older than 600 ticks: 1 in 800 roll succeeds when roll < 1/800 = 0.00125
    assert!(should_despawn(EntityType::Skeleton, mid_pos, &players, 600, false, 0.0005));

    // Older than 600 ticks: 1 in 800 roll fails when roll >= 1/800
    assert!(!should_despawn(EntityType::Skeleton, mid_pos, &players, 600, false, 0.5));
}

#[test]
fn test_hostile_mob_conditions_block_light_zero_and_floor() {
    let mut world = MockWorld::new(false); // Night
    let x = 10;
    let y = 64;
    let z = 10;

    // Floor: stone
    world.set_block(x, y - 1, z, SurfaceBlock::Stone);
    // Body and head: air
    world.set_block(x, y, z, SurfaceBlock::Air);
    world.set_block(x, y + 1, z, SurfaceBlock::Air);
    // Modern 1.18+ rule: Block light == 0
    world.set_light(x, y, z, 0, 0);

    // Valid hostile spawn!
    assert!(verify_monster_conditions(&world, x, y, z, EntityType::Zombie));

    // Block light > 0 (e.g. torch nearby) -> Rejected!
    world.set_light(x, y, z, 1, 0);
    assert!(!verify_monster_conditions(&world, x, y, z, EntityType::Zombie));

    // Reset light, remove floor (air beneath) -> Rejected!
    world.set_light(x, y, z, 0, 0);
    world.set_block(x, y - 1, z, SurfaceBlock::Air);
    assert!(!verify_monster_conditions(&world, x, y, z, EntityType::Zombie));
}

#[test]
fn test_passive_animal_conditions_grass_and_light_level() {
    let mut world = MockWorld::new(true); // Daytime
    let x = 20;
    let y = 64;
    let z = 20;

    // Floor: GrassBlock
    world.set_block(x, y - 1, z, SurfaceBlock::GrassBlock);
    world.set_block(x, y, z, SurfaceBlock::Air);
    world.set_block(x, y + 1, z, SurfaceBlock::Air);
    // Light level >= 9 (daylight sky light = 15)
    world.set_light(x, y, z, 0, 15);

    // Valid animal spawn!
    assert!(verify_creature_conditions(&world, x, y, z, EntityType::Cow));

    // Stone floor instead of GrassBlock -> Rejected!
    world.set_block(x, y - 1, z, SurfaceBlock::Stone);
    assert!(!verify_creature_conditions(&world, x, y, z, EntityType::Cow));

    // Restore grass, but nighttime with zero block light (< 9) -> Rejected!
    world.set_block(x, y - 1, z, SurfaceBlock::GrassBlock);
    world.is_day = false;
    world.set_light(x, y, z, 0, 0);
    assert!(!verify_creature_conditions(&world, x, y, z, EntityType::Cow));
}

#[test]
fn test_aquatic_mob_conditions_water_blocks() {
    let mut world = MockWorld::new(true);
    let x = 30;
    let y = 60;
    let z = 30;

    // Water column
    world.set_block(x, y, z, SurfaceBlock::Water);
    world.set_block(x, y - 1, z, SurfaceBlock::Water);

    // Valid aquatic spawn!
    assert!(verify_aquatic_conditions(&world, x, y, z, EntityType::Cod));

    // Dry air -> Rejected!
    world.set_block(x, y, z, SurfaceBlock::Air);
    assert!(!verify_aquatic_conditions(&world, x, y, z, EntityType::Cod));
}

#[test]
fn test_biome_spawn_settings_weighted_selection() {
    let mut settings = BiomeSpawnSettings::new();
    settings.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Zombie, 100, 4, 4));
    settings.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Skeleton, 100, 4, 4));
    settings.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Creeper, 100, 4, 4));
    settings.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Witch, 5, 1, 1));

    // Total weight = 305
    // Roll 0.1 (target ~30) should pick Zombie (weight 0..100)
    let picked1 = settings.sample_mob(MobCategory::Monster, 0.1).unwrap();
    assert_eq!(picked1.entity_type, EntityType::Zombie);

    // Roll 0.5 (target ~152) should pick Skeleton (weight 100..200)
    let picked2 = settings.sample_mob(MobCategory::Monster, 0.5).unwrap();
    assert_eq!(picked2.entity_type, EntityType::Skeleton);

    // Roll 0.99 (target ~301) should pick Witch (weight 300..305)
    let picked3 = settings.sample_mob(MobCategory::Monster, 0.99).unwrap();
    assert_eq!(picked3.entity_type, EntityType::Witch);
}

#[test]
fn test_natural_spawner_tick_cycle_pack_spawning() {
    let mut world = MockWorld::new(false); // Night
    let player = Vec3::new(0.0, 64.0, 0.0);

    // Set up a flat stone area in chunk (2, 0) -> X: 32..47, Z: 0..15
    // Distance from (0, 64, 0) to (36, 64, 8) is ~36 blocks (in range 24..128)
    for x in 32..48 {
        for z in 0..16 {
            world.set_block(x, 63, z, SurfaceBlock::Stone);
            world.set_block(x, 64, z, SurfaceBlock::Air);
            world.set_block(x, 65, z, SurfaceBlock::Air);
            world.set_light(x, 64, z, 0, 0);
            world.top_y.insert((x, z), 63);
        }
    }

    let mut spawner = NaturalSpawner::new();
    let biome_provider = |_x: i32, _z: i32| "minecraft:plains";

    // Fixed deterministic pseudo-random sequence
    let mut seed = 0.25f32;
    let rng = move || {
        seed = (seed + 0.137) % 1.0;
        seed
    };

    let spawned = spawner.run_spawn_cycle(&world, &biome_provider, &[player], rng);

    // Mobs should have spawned in the dark valid zone
    assert!(!spawned.is_empty(), "Expected hostile mobs to spawn in dark chunk");
    assert!(spawner.tracker.get_count(MobCategory::Monster) > 0);

    // Check first spawned mob attributes
    let first = &spawned[0];
    assert_eq!(first.category, MobCategory::Monster);
    assert!(first.health.is_alive());
    assert!(first.position.x >= 24.0); // Outside 24-block player radius
}

#[test]
fn test_natural_spawner_despawn_sweep_integration() {
    let mut spawner = NaturalSpawner::new();
    let player = Vec3::new(0.0, 64.0, 0.0);

    let id1 = spawner.next_id();
    let id2 = spawner.next_id();

    // Mob 1: Close to player (20 blocks away)
    let mob1 = SpawnedMob::new(id1, EntityType::Zombie, MobCategory::Monster, Vec3::new(20.0, 64.0, 0.0));
    // Mob 2: Very far away (150 blocks away)
    let mob2 = SpawnedMob::new(id2, EntityType::Skeleton, MobCategory::Monster, Vec3::new(150.0, 64.0, 0.0));

    spawner.tracker.register_spawn(EntityType::Zombie);
    spawner.tracker.register_spawn(EntityType::Skeleton);
    assert_eq!(spawner.tracker.get_count(MobCategory::Monster), 2);

    let mut mobs = vec![mob1, mob2];

    // Run despawn sweep
    let despawned = spawner.run_despawn_sweep(&mut mobs, &[player], || 0.5);

    // Mob 2 should have been despawned immediately (> 128 blocks)
    assert_eq!(despawned, vec![id2]);
    assert_eq!(mobs.len(), 1);
    assert_eq!(mobs[0].id, id1);

    // Category tracker was decremented
    assert_eq!(spawner.tracker.get_count(MobCategory::Monster), 1);
}
