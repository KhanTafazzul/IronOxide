use mc_worldgen::{
    decorate_chunk, generate_chunk_column_pipeline, BiomeFeatureRegistry, BiomeId,
    ChunkColumnBuffer, DecorationStep, HeightRangeDef, IntProviderDef, OverworldGenerator,
    SurfaceBlock, VerticalAnchorDef, Xoroshiro128PlusPlus,
    FLUID_SPRINGS, LAKES, LOCAL_MODIFICATIONS, MIN_Y, RAW_GENERATION,
    STRONGHOLDS, SURFACE_STRUCTURES, TOP_LAYER_MODIFICATION, TOTAL_DECORATION_STEPS,
    UNDERGROUND_DECORATION, UNDERGROUND_ORES, UNDERGROUND_STRUCTURES, VEGETAL_DECORATION,
};

#[test]
fn test_vanilla_11_step_decoration_enum_indices() {
    assert_eq!(RAW_GENERATION, 0);
    assert_eq!(LAKES, 1);
    assert_eq!(LOCAL_MODIFICATIONS, 2);
    assert_eq!(UNDERGROUND_STRUCTURES, 3);
    assert_eq!(SURFACE_STRUCTURES, 4);
    assert_eq!(STRONGHOLDS, 5);
    assert_eq!(UNDERGROUND_ORES, 6);
    assert_eq!(UNDERGROUND_DECORATION, 7);
    assert_eq!(FLUID_SPRINGS, 8);
    assert_eq!(VEGETAL_DECORATION, 9);
    assert_eq!(TOP_LAYER_MODIFICATION, 10);
    assert_eq!(TOTAL_DECORATION_STEPS, 11);

    assert_eq!(DecorationStep::RawGeneration.index(), 0);
    assert_eq!(DecorationStep::Lakes.index(), 1);
    assert_eq!(DecorationStep::LocalModifications.index(), 2);
    assert_eq!(DecorationStep::UndergroundStructures.index(), 3);
    assert_eq!(DecorationStep::SurfaceStructures.index(), 4);
    assert_eq!(DecorationStep::Strongholds.index(), 5);
    assert_eq!(DecorationStep::UndergroundOres.index(), 6);
    assert_eq!(DecorationStep::UndergroundDecoration.index(), 7);
    assert_eq!(DecorationStep::FluidSprings.index(), 8);
    assert_eq!(DecorationStep::VegetalDecoration.index(), 9);
    assert_eq!(DecorationStep::TopLayerModification.index(), 10);

    assert_eq!(DecorationStep::RawGeneration.name(), "raw_generation");
    assert_eq!(DecorationStep::Lakes.name(), "lakes");
    assert_eq!(DecorationStep::LocalModifications.name(), "local_modifications");
    assert_eq!(DecorationStep::UndergroundStructures.name(), "underground_structures");
    assert_eq!(DecorationStep::SurfaceStructures.name(), "surface_structures");
    assert_eq!(DecorationStep::Strongholds.name(), "strongholds");
    assert_eq!(DecorationStep::UndergroundOres.name(), "underground_ores");
    assert_eq!(DecorationStep::UndergroundDecoration.name(), "underground_decoration");
    assert_eq!(DecorationStep::FluidSprings.name(), "fluid_springs");
    assert_eq!(DecorationStep::VegetalDecoration.name(), "vegetal_decoration");
    assert_eq!(DecorationStep::TopLayerModification.name(), "top_layer_modification");

    for idx in 0..11 {
        let step = DecorationStep::from_index(idx).expect("Valid step index");
        assert_eq!(step.index(), idx);
    }
    assert!(DecorationStep::from_index(11).is_none());
}

#[test]
fn test_client_jar_biome_feature_registry_loading() {
    let registry = BiomeFeatureRegistry::global();
    assert!(
        !registry.biome_features.is_empty(),
        "Biome feature registry must load biomes"
    );

    let plains_features = registry.get_features_for_biome(BiomeId::Plains);
    assert_eq!(plains_features.len(), 11);

    // Step 6 (Underground Ores) should have dirt, gravel, coal, iron, diamond, etc.
    let ores = &plains_features[UNDERGROUND_ORES];
    assert!(
        !ores.is_empty(),
        "Plains must have underground ores registered"
    );
    assert!(
        ores.iter().any(|f| f.contains("ore_coal")),
        "Ores must contain coal"
    );
    assert!(
        ores.iter().any(|f| f.contains("ore_iron")),
        "Ores must contain iron"
    );
    assert!(
        ores.iter().any(|f| f.contains("ore_diamond")),
        "Ores must contain diamond"
    );

    // Step 8 (Fluid Springs)
    let springs = &plains_features[FLUID_SPRINGS];
    assert!(
        springs.iter().any(|f| f.contains("spring_water") || f.contains("spring_lava")),
        "Must contain fluid springs"
    );

    // Step 9 (Vegetal Decoration)
    let vegetation = &plains_features[VEGETAL_DECORATION];
    assert!(
        vegetation.iter().any(|f| f.contains("trees_plains") || f.contains("flower") || f.contains("grass")),
        "Must contain vegetal decoration"
    );

    // Step 10 (Top Layer Modification)
    let top_layer = &plains_features[TOP_LAYER_MODIFICATION];
    assert!(
        top_layer.iter().any(|f| f.contains("freeze_top_layer")),
        "Must contain freeze_top_layer"
    );
}

#[test]
fn test_placement_modifiers_evaluation() {
    let mut rng = Xoroshiro128PlusPlus::new(1234567, 7654321);

    // Test InSquare modifier logic
    let dx = rng.next_int_bounded(16);
    let dz = rng.next_int_bounded(16);
    assert!((0..16).contains(&dx));
    assert!((0..16).contains(&dz));

    // Test Count modifier logic
    let count_provider = IntProviderDef::WeightedList(vec![(0, 19), (1, 1)]);
    let mut sample_1_count = 0;
    for _ in 0..1000 {
        if count_provider.sample(&mut rng) == 1 {
            sample_1_count += 1;
        }
    }
    assert!(sample_1_count > 0 && sample_1_count < 200, "Weighted list sampling distribution parity");

    // Test HeightRange uniform logic
    let height_uniform = HeightRangeDef::Uniform {
        min: VerticalAnchorDef::AboveBottom(0), // -64
        max: VerticalAnchorDef::Absolute(320),  // 320
    };
    for _ in 0..100 {
        let y = height_uniform.sample(&mut rng);
        assert!((-64..=320).contains(&y));
    }

    // Test HeightRange trapezoid logic
    let height_trapezoid = HeightRangeDef::Trapezoid {
        min: VerticalAnchorDef::AboveBottom(0), // -64
        max: VerticalAnchorDef::Absolute(128),
        plateau: 0,
    };
    for _ in 0..100 {
        let y = height_trapezoid.sample(&mut rng);
        assert!((-64..=128).contains(&y));
    }
}

#[test]
fn test_chunk_decoration_pipeline_dispatcher() {
    let world_seed = 123456789u64;
    let chunk_x = 15;
    let chunk_z = 7;

    let mut buffer = ChunkColumnBuffer::new(chunk_x, chunk_z, BiomeId::Plains);

    // Populate baseline stone terrain for testing decoration placement
    for lz in 0..16 {
        for lx in 0..16 {
            buffer.set_local(lx, 0, lz, SurfaceBlock::Bedrock);
            for y in -63..=64 {
                let buf_y = (y - MIN_Y) as usize;
                if y < 0 {
                    buffer.set_local(lx, buf_y, lz, SurfaceBlock::Deepslate);
                } else {
                    buffer.set_local(lx, buf_y, lz, SurfaceBlock::Stone);
                }
            }
            // Grass top soil
            buffer.set_local(lx, (65 - MIN_Y) as usize, lz, SurfaceBlock::GrassBlock);
        }
    }

    let registry = BiomeFeatureRegistry::global();
    decorate_chunk(&mut buffer, chunk_x, chunk_z, world_seed, registry);

    // Verify ores were placed across the 11-step decoration pass
    let mut ore_blocks_found = 0;
    let mut plants_found = 0;
    for &b in buffer.blocks.iter() {
        match b {
            SurfaceBlock::CoalOre
            | SurfaceBlock::DeepslateCoalOre
            | SurfaceBlock::IronOre
            | SurfaceBlock::DeepslateIronOre
            | SurfaceBlock::CopperOre
            | SurfaceBlock::DeepslateCopperOre
            | SurfaceBlock::GoldOre
            | SurfaceBlock::DeepslateGoldOre
            | SurfaceBlock::RedstoneOre
            | SurfaceBlock::DeepslateRedstoneOre
            | SurfaceBlock::DiamondOre
            | SurfaceBlock::DeepslateDiamondOre
            | SurfaceBlock::LapisOre
            | SurfaceBlock::DeepslateLapisOre
            | SurfaceBlock::Dirt
            | SurfaceBlock::Gravel => {
                ore_blocks_found += 1;
            }
            SurfaceBlock::Poppy
            | SurfaceBlock::Dandelion
            | SurfaceBlock::ShortGrass
            | SurfaceBlock::TallGrass
            | SurfaceBlock::OakLog
            | SurfaceBlock::OakLeaves => {
                plants_found += 1;
            }
            _ => {}
        }
    }

    assert!(
        ore_blocks_found > 0,
        "Decorator pipeline must place underground ores (found {})",
        ore_blocks_found
    );
    assert!(
        plants_found > 0,
        "Decorator pipeline must place vegetal features (found {})",
        plants_found
    );
}

#[test]
fn test_generate_chunk_column_full_pipeline() {
    let generator = OverworldGenerator::new(123456789);
    let chunk = generate_chunk_column_pipeline(&generator, 0, 0);

    assert_eq!(chunk.chunk_x, 0);
    assert_eq!(chunk.chunk_z, 0);

    // Check for Deepslate in lower layer (Y = -60 to -50)
    let mut has_deepslate = false;
    for y in -60..-50 {
        let buf_y = (y - MIN_Y) as usize;
        for lz in 0..16 {
            for lx in 0..16 {
                if chunk.get_local(lx, buf_y, lz) == SurfaceBlock::Deepslate {
                    has_deepslate = true;
                    break;
                }
            }
        }
    }
    assert!(has_deepslate, "Must contain deepslate beneath Y=0");

    // Check for solid surface block (Grass, Sand, Water, or Stone)
    let mut has_surface = false;
    for lz in 0..16 {
        for lx in 0..16 {
            for y in 50..=120 {
                let buf_y = (y - MIN_Y) as usize;
                let b = chunk.get_local(lx, buf_y, lz);
                if b == SurfaceBlock::GrassBlock || b == SurfaceBlock::Dirt || b == SurfaceBlock::Water || b == SurfaceBlock::Sand {
                    has_surface = true;
                    break;
                }
            }
        }
    }
    assert!(has_surface, "Chunk must generate terrain surface");

    // Check that ores are generated
    let mut ore_count = 0;
    for idx in 0..chunk.blocks.len() {
        let b = chunk.blocks[idx];
        match b {
            SurfaceBlock::CoalOre
            | SurfaceBlock::DeepslateCoalOre
            | SurfaceBlock::IronOre
            | SurfaceBlock::DeepslateIronOre
            | SurfaceBlock::CopperOre
            | SurfaceBlock::DeepslateCopperOre
            | SurfaceBlock::GoldOre
            | SurfaceBlock::DeepslateGoldOre
            | SurfaceBlock::RedstoneOre
            | SurfaceBlock::DeepslateRedstoneOre
            | SurfaceBlock::DiamondOre
            | SurfaceBlock::DeepslateDiamondOre
            | SurfaceBlock::LapisOre
            | SurfaceBlock::DeepslateLapisOre => {
                ore_count += 1;
            }
            _ => {}
        }
    }
    assert!(ore_count > 0, "Chunk must contain generated ores (found {})", ore_count);
}
