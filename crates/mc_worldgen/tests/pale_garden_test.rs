use mc_worldgen::biomes::pale_garden::*;
use mc_worldgen::biomes::{BiomeCategory, BiomeId, SurfaceBlock};
use mc_worldgen::features::pale_garden::*;
use mc_worldgen::features::trees::{is_replaceable_for_tree, is_valid_tree_soil, TreeFeature, TreeSpecies};
use mc_worldgen::ores::{ChunkOreBuffer, OreBlockAccess};
use mc_worldgen::pipeline::ChunkColumnBuffer;
use mc_worldgen::seed_pipeline::climate::TargetPoint;
use mc_worldgen::seed_pipeline::random::Xoroshiro128PlusPlus;
use mc_worldgen::structures::LegacyRandom;

#[test]
fn test_pale_garden_climate_definition() {
    // 1. Verify 6D climate ranges: low vegetation, moderate continentalness, weirdness offset
    assert_eq!(PALE_GARDEN_CLIMATE.temperature.min, -1000);
    assert_eq!(PALE_GARDEN_CLIMATE.temperature.max, 2000);

    // Low vegetation
    assert_eq!(PALE_GARDEN_CLIMATE.humidity.min, -4000);
    assert_eq!(PALE_GARDEN_CLIMATE.humidity.max, -1000);

    // Moderate continentalness
    assert_eq!(PALE_GARDEN_CLIMATE.continentalness.min, 1000);
    assert_eq!(PALE_GARDEN_CLIMATE.continentalness.max, 3000);

    // Unique weirdness band and offset
    assert_eq!(PALE_GARDEN_CLIMATE.weirdness.min, 3000);
    assert_eq!(PALE_GARDEN_CLIMATE.weirdness.max, 6000);
    assert_eq!(PALE_GARDEN_CLIMATE.offset, 500);

    // 2. In-bounds target point check
    let inside_point = TargetPoint {
        temperature: 0,
        humidity: -2500,
        continentalness: 2000,
        erosion: 0,
        depth: 0,
        weirdness: 4500,
    };
    assert!(is_pale_garden_climate(&inside_point));

    // 3. Out-of-bounds target point (high vegetation)
    let outside_point = TargetPoint {
        temperature: 0,
        humidity: 3000, // Lush/high vegetation
        continentalness: 2000,
        erosion: 0,
        depth: 0,
        weirdness: 4500,
    };
    assert!(!is_pale_garden_climate(&outside_point));

    // 4. Fitness evaluation
    let fitness_inside = pale_garden_fitness(&inside_point);
    let fitness_outside = pale_garden_fitness(&outside_point);
    assert!(fitness_inside < fitness_outside);
}

#[test]
fn test_pale_garden_atmospheric_effects_and_colors() {
    // Exact hex colors:
    // Fog color: #9E9EA0 ([158, 158, 160])
    assert_eq!(PALE_GARDEN_FOG_COLOR, [0x9E, 0x9E, 0xA0]);

    // Foliage color: #87857E ([135, 133, 126])
    assert_eq!(PALE_GARDEN_FOLIAGE_COLOR, [0x87, 0x85, 0x7E]);

    // Grass color: #84877D ([132, 135, 125])
    assert_eq!(PALE_GARDEN_GRASS_COLOR, [0x84, 0x87, 0x7D]);

    // Sky color: #C0C0C0 ([192, 192, 192])
    assert_eq!(PALE_GARDEN_SKY_COLOR, [0xC0, 0xC0, 0xC0]);

    // Default atmosphere struct
    let atmosphere = PaleGardenAtmosphere::default();
    assert!((atmosphere.fog_color[0] - (158.0 / 255.0)).abs() < 0.01);
    assert!((atmosphere.foliage_color[0] - (135.0 / 255.0)).abs() < 0.01);
    assert!((atmosphere.grass_color[0] - (132.0 / 255.0)).abs() < 0.01);

    // Desaturation function test
    let vibrant_green = [50, 220, 30];
    let desaturated = desaturate_to_pale_garden(vibrant_green, 1.0);
    // Fully desaturated should have R ~= G ~= B
    assert!((desaturated[0] as i32 - desaturated[1] as i32).abs() <= 2);
    assert!((desaturated[1] as i32 - desaturated[2] as i32).abs() <= 2);
}

#[test]
fn test_pale_garden_biome_id_registry() {
    let biome = BiomeId::PaleGarden;
    assert_eq!(biome.name(), "minecraft:pale_garden");
    assert_eq!(BiomeId::from_name("minecraft:pale_garden"), Some(BiomeId::PaleGarden));
    assert_eq!(BiomeId::from_name("pale_garden"), Some(BiomeId::PaleGarden));
    assert_eq!(biome.category(), BiomeCategory::Temperate);
    assert_eq!(biome.temperature(), 0.7);
    assert!(biome.has_precipitation());
    assert!(!biome.is_snowy());
    assert!(!biome.is_badlands());
    assert!(!biome.is_ocean());
}

#[test]
fn test_pale_oak_wood_set_blocks() {
    // Valid soil check
    assert!(is_valid_tree_soil(SurfaceBlock::PaleMossBlock));
    assert!(is_valid_pale_oak_soil(SurfaceBlock::PaleMossBlock));
    assert!(is_valid_pale_oak_soil(SurfaceBlock::GrassBlock));
    assert!(is_valid_pale_oak_soil(SurfaceBlock::Dirt));
    assert!(!is_valid_pale_oak_soil(SurfaceBlock::Sand));

    // Replaceable block check
    assert!(is_replaceable_for_tree(SurfaceBlock::PaleOakLeaves));
    assert!(is_replaceable_for_tree(SurfaceBlock::PaleMossCarpet));
    assert!(is_replaceable_for_tree(SurfaceBlock::PaleHangingMoss));

    // TreeSpecies enum mapping
    let species = TreeSpecies::PaleOak;
    assert_eq!(species.log_block(), SurfaceBlock::PaleOakLog);
    assert_eq!(species.leaf_block(), SurfaceBlock::PaleOakLeaves);
}

#[test]
fn test_pale_oak_tree_generation_without_heart() {
    let mut buffer = ChunkOreBuffer::new(0, 0, 0, 32, 32, 32, SurfaceBlock::Air);

    // Prepare 2x2 ground patch at Y=5
    for dx in 0..=1 {
        for dz in 0..=1 {
            buffer.set_block(10 + dx, 5, 10 + dz, SurfaceBlock::PaleMossBlock);
        }
    }

    let mut rng = LegacyRandom::new(12345);
    let result = generate_pale_oak_tree(&mut buffer, (10, 6, 10), &mut rng, false);

    assert!(result.logs_placed >= 40, "2x2 trunk should have at least 10 * 4 = 40 logs");
    assert!(result.leaves_placed > 0, "Canopy should have placed leaves");
    assert!(!result.creaking_heart_placed, "Should not place creaking heart when flag is false");

    // Soil underneath trunk should be converted to Dirt
    for dx in 0..=1 {
        for dz in 0..=1 {
            assert_eq!(buffer.get_block(10 + dx, 5, 10 + dz), SurfaceBlock::Dirt);
        }
    }

    // Check 2x2 trunk logs
    for y in 6..14 {
        for dx in 0..=1 {
            for dz in 0..=1 {
                assert_eq!(buffer.get_block(10 + dx, y, 10 + dz), SurfaceBlock::PaleOakLog);
            }
        }
    }
}

#[test]
fn test_pale_oak_tree_generation_with_creaking_heart() {
    let mut buffer = ChunkOreBuffer::new(0, 0, 0, 32, 32, 32, SurfaceBlock::Air);

    for dx in 0..=1 {
        for dz in 0..=1 {
            buffer.set_block(10 + dx, 5, 10 + dz, SurfaceBlock::GrassBlock);
        }
    }

    let mut rng = LegacyRandom::new(54321);
    let result = generate_pale_oak_tree(&mut buffer, (10, 6, 10), &mut rng, true);

    assert!(result.creaking_heart_placed, "Creaking Heart should be placed in tree trunk");

    // Locate creaking heart block within trunk coordinates
    let mut found_heart = false;
    let mut heart_pos = (0, 0, 0);
    for y in 6..20 {
        for dx in 0..=1 {
            for dz in 0..=1 {
                if buffer.get_block(10 + dx, y, 10 + dz) == SurfaceBlock::CreakingHeart {
                    found_heart = true;
                    heart_pos = (10 + dx, y, 10 + dz);
                }
            }
        }
    }

    assert!(found_heart, "Heart block must exist in buffer");

    // Verify encasement: Creaking Heart must have PaleOakLog directly above and below
    let (hx, hy, hz) = heart_pos;
    assert_eq!(buffer.get_block(hx, hy - 1, hz), SurfaceBlock::PaleOakLog, "Log below heart");
    assert_eq!(buffer.get_block(hx, hy + 1, hz), SurfaceBlock::PaleOakLog, "Log above heart");
}

#[test]
fn test_tree_feature_dispatch_pale_oak() {
    let mut buffer = ChunkOreBuffer::new(0, 0, 0, 32, 32, 32, SurfaceBlock::Air);
    for dx in 0..=1 {
        for dz in 0..=1 {
            buffer.set_block(10 + dx, 5, 10 + dz, SurfaceBlock::GrassBlock);
        }
    }

    let mut rng = LegacyRandom::new(9999);
    let (logs, leaves) = TreeFeature::place_tree(&mut buffer, (10, 6, 10), TreeSpecies::PaleOak, &mut rng);
    assert!(logs >= 40);
    assert!(leaves > 0);
}

#[test]
fn test_creaking_heart_state_machine_day_night_cycle() {
    let mut heart = CreakingHeartTileEntity::new((10, 64, 10), true);
    assert_eq!(heart.dormancy_state, CreakingHeartDormancy::Dormant);
    assert!(!heart.is_active);
    assert!(heart.bound_creaking_uuid.is_none());

    // 1. Daytime tick with daylight > 0: must remain dormant
    let action_day = heart.tick(15, 6000, true);
    assert_eq!(action_day, CreakingHeartAction::None);
    assert_eq!(heart.dormancy_state, CreakingHeartDormancy::Dormant);
    assert!(!heart.is_active);

    // 2. Night arrives (daylight = 0, time_of_day = 14000), flanked by Pale Oak logs
    let action_night = heart.tick(0, 14000, true);
    assert_eq!(heart.dormancy_state, CreakingHeartDormancy::Awake);
    assert!(heart.is_active);
    assert_eq!(
        action_night,
        CreakingHeartAction::SpawnCreaking {
            heart_pos: (10, 64, 10),
        }
    );

    // 3. Bind a spawned Creaking entity UUID
    let test_uuid: u128 = 0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0;
    heart.bind_creaking(test_uuid);
    assert_eq!(heart.bound_creaking_uuid, Some(test_uuid));

    // Next night tick: puppet already bound, no new spawn action
    let action_night2 = heart.tick(0, 15000, true);
    assert_eq!(action_night2, CreakingHeartAction::None);
    assert!(heart.is_active);

    // 4. Night time but Pale Oak log encasement broken
    let action_unencased = heart.tick(0, 16000, false);
    assert_eq!(
        action_unencased,
        CreakingHeartAction::DespawnCreaking {
            creaking_uuid: test_uuid,
        }
    );
    assert_eq!(heart.dormancy_state, CreakingHeartDormancy::Dormant);
    assert!(!heart.is_active);
    assert!(heart.bound_creaking_uuid.is_none());

    // 5. Re-awaken at night, bind again, then transition to DAY
    heart.tick(0, 17000, true);
    let second_uuid: u128 = 0xFEED_FACE_CAFE_BEEF_0000_1111_2222_3333;
    heart.bind_creaking(second_uuid);

    // Sunrise: daylight level rises to 12
    let action_sunrise = heart.tick(12, 1000, true);
    assert_eq!(
        action_sunrise,
        CreakingHeartAction::DespawnCreaking {
            creaking_uuid: second_uuid,
        }
    );
    assert_eq!(heart.dormancy_state, CreakingHeartDormancy::Dormant);
    assert!(!heart.is_active);
}

#[test]
fn test_creaking_damage_redirection_and_resin() {
    let mut heart = CreakingHeartTileEntity::new((15, 68, 15), true);
    heart.is_active = true;
    heart.dormancy_state = CreakingHeartDormancy::Awake;
    assert_eq!(heart.health, 20.0);
    assert_eq!(heart.resin_level, 0);

    // Attack 1: 5.0 damage
    let res1 = heart.on_creaking_attacked(5.0);
    assert!(res1.puppet_damage_negated, "Direct puppet damage must be completely negated");
    assert_eq!(res1.heart_damage, 5.0, "Damage redirected to heart");
    assert_eq!(res1.new_resin_level, 1, "Resin accumulates on hit");
    assert_eq!(heart.health, 15.0);
    assert!(!res1.heart_destroyed);

    // Attack 2: 10.0 damage
    let res2 = heart.on_creaking_attacked(10.0);
    assert_eq!(res2.new_resin_level, 2);
    assert_eq!(heart.health, 5.0);
    assert!(!res2.heart_destroyed);

    // Attack 3: 10.0 fatal damage
    let res3 = heart.on_creaking_attacked(10.0);
    assert!(res3.heart_destroyed);
    assert_eq!(heart.dormancy_state, CreakingHeartDormancy::Destroyed);
    assert!(!heart.is_active);
}

#[test]
fn test_creaking_heart_broken_kills_puppet() {
    let mut heart = CreakingHeartTileEntity::new((20, 65, 20), true);
    let bound_uuid = 0xABCD_EF01_2345_6789_0000_0000_0000_0001;
    heart.bind_creaking(bound_uuid);

    let killed_uuid = heart.on_heart_broken();
    assert_eq!(killed_uuid, Some(bound_uuid), "Breaking heart must return puppet UUID to kill it");
    assert_eq!(heart.dormancy_state, CreakingHeartDormancy::Destroyed);
    assert!(!heart.is_active);
    assert!(heart.bound_creaking_uuid.is_none());
}

#[test]
fn test_pale_garden_chunk_decoration() {
    let mut buffer = ChunkColumnBuffer::new(0, 0, BiomeId::PaleGarden);

    // Build flat ground at Y=64
    for x in 0..16 {
        for z in 0..16 {
            for y in (-64i32..=63i32).rev() {
                buffer.set_local(x, (y + 64) as usize, z, SurfaceBlock::Stone);
            }
            buffer.set_local(x, (64 + 64) as usize, z, SurfaceBlock::GrassBlock);
        }
    }

    let mut rng = Xoroshiro128PlusPlus::new(777, 888);
    let trees = decorate_pale_garden_vegetation(&mut buffer, 0, 0, &mut rng);
    println!("DEBUG: trees placed = {}", trees);
    assert!(trees >= 1, "Chunk decoration should place Pale Oak trees in Pale Garden");

    // Scan chunk for PaleOakLog and PaleMossBlock
    let mut found_logs = 0;
    let mut found_moss = 0;
    let mut found_leaves = 0;

    for y in 0..384 {
        for x in 0..16 {
            for z in 0..16 {
                match buffer.get_local(x, y, z) {
                    SurfaceBlock::PaleOakLog => found_logs += 1,
                    SurfaceBlock::PaleMossBlock | SurfaceBlock::PaleMossCarpet => found_moss += 1,
                    SurfaceBlock::PaleOakLeaves => found_leaves += 1,
                    _ => {}
                }
            }
        }
    }

    assert!(found_logs > 0, "Pale Oak logs must be present");
    assert!(found_leaves > 0, "Pale Oak leaves must be present");
    assert!(found_moss > 0, "Pale Moss ground cover must be present");
}
