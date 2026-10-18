use mc_worldgen::{
    BiomeCategory, BiomeId, SurfaceBlock, SurfaceRuleEngine, SEA_LEVEL, MIN_Y,
};

#[test]
fn test_all_overworld_biomes_registration_and_roundtrip() {
    let biomes = [
        // Snowy / Cold
        BiomeId::SnowyPlains,
        BiomeId::IceSpikes,
        BiomeId::SnowyTaiga,
        BiomeId::FrozenPeaks,
        BiomeId::JaggedPeaks,
        BiomeId::Grove,
        BiomeId::SnowySlopes,
        BiomeId::FrozenRiver,
        BiomeId::SnowyBeach,
        // Temperate / Wet
        BiomeId::Plains,
        BiomeId::SunflowerPlains,
        BiomeId::Forest,
        BiomeId::FlowerForest,
        BiomeId::BirchForest,
        BiomeId::OldGrowthBirchForest,
        BiomeId::DarkForest,
        BiomeId::Taiga,
        BiomeId::OldGrowthPineTaiga,
        BiomeId::OldGrowthSpruceTaiga,
        BiomeId::Swamp,
        BiomeId::MangroveSwamp,
        BiomeId::Jungle,
        BiomeId::SparseJungle,
        BiomeId::BambooJungle,
        BiomeId::River,
        BiomeId::Beach,
        BiomeId::StonyShore,
        BiomeId::Meadow,
        BiomeId::CherryGrove,
        // Arid / Hot
        BiomeId::Desert,
        BiomeId::Savanna,
        BiomeId::SavannaPlateau,
        BiomeId::WindsweptSavanna,
        BiomeId::WindsweptHills,
        BiomeId::WindsweptGravellyHills,
        BiomeId::WindsweptForest,
        BiomeId::Badlands,
        BiomeId::ErodedBadlands,
        BiomeId::WoodedBadlands,
        BiomeId::StonyPeaks,
        // Ocean Variants
        BiomeId::WarmOcean,
        BiomeId::LukewarmOcean,
        BiomeId::DeepLukewarmOcean,
        BiomeId::Ocean,
        BiomeId::DeepOcean,
        BiomeId::ColdOcean,
        BiomeId::DeepColdOcean,
        BiomeId::FrozenOcean,
        BiomeId::DeepFrozenOcean,
        // Rare / Caves
        BiomeId::MushroomFields,
        BiomeId::DripstoneCaves,
        BiomeId::LushCaves,
        BiomeId::DeepDark,
    ];

    for biome in biomes {
        let name = biome.name();
        assert!(name.starts_with("minecraft:"));
        let parsed = BiomeId::from_name(name);
        assert_eq!(parsed, Some(biome), "Failed roundtrip for {}", name);

        // Also test without minecraft: prefix
        let raw_name = name.strip_prefix("minecraft:").unwrap();
        let parsed_raw = BiomeId::from_name(raw_name);
        assert_eq!(parsed_raw, Some(biome));
    }
}

#[test]
fn test_biome_categories_and_climates() {
    assert_eq!(BiomeId::SnowyPlains.category(), BiomeCategory::Snowy);
    assert_eq!(BiomeId::FrozenPeaks.category(), BiomeCategory::Cold);
    assert_eq!(BiomeId::Plains.category(), BiomeCategory::Temperate);
    assert_eq!(BiomeId::Swamp.category(), BiomeCategory::Wet);
    assert_eq!(BiomeId::Desert.category(), BiomeCategory::Arid);
    assert_eq!(BiomeId::Badlands.category(), BiomeCategory::Arid);
    assert_eq!(BiomeId::DeepOcean.category(), BiomeCategory::Ocean);

    // Precipitation
    assert!(!BiomeId::Desert.has_precipitation());
    assert!(!BiomeId::Badlands.has_precipitation());
    assert!(!BiomeId::Savanna.has_precipitation());
    assert!(BiomeId::Plains.has_precipitation());
    assert!(BiomeId::SnowyPlains.has_precipitation());

    // Snowy flag
    assert!(BiomeId::SnowyPlains.is_snowy());
    assert!(BiomeId::JaggedPeaks.is_snowy());
    assert!(!BiomeId::Desert.is_snowy());
    assert!(!BiomeId::Plains.is_snowy());
}

#[test]
fn test_surface_rules_snowy_biomes() {
    let engine = SurfaceRuleEngine::default();
    let surface_y = 70;

    // 1. Snowy Plains: GrassBlock at surface, SnowLayer at surface+1, Dirt below
    let top_block = engine.get_block(0, surface_y, 0, surface_y, BiomeId::SnowyPlains, 0.0);
    assert_eq!(top_block, SurfaceBlock::GrassBlock);

    let snow_layer = engine.get_block(0, surface_y + 1, 0, surface_y, BiomeId::SnowyPlains, 0.0);
    assert_eq!(snow_layer, SurfaceBlock::SnowLayer);

    let sub_block = engine.get_block(0, surface_y - 2, 0, surface_y, BiomeId::SnowyPlains, 0.0);
    assert_eq!(sub_block, SurfaceBlock::Dirt);

    let deep_block = engine.get_block(0, surface_y - 10, 0, surface_y, BiomeId::SnowyPlains, 0.0);
    assert_eq!(deep_block, SurfaceBlock::Stone);

    // 2. Frozen Peaks: SnowBlock at surface, PackedIce beneath
    let peak_surface = 180;
    let peak_top = engine.get_block(0, peak_surface, 0, peak_surface, BiomeId::FrozenPeaks, 0.0);
    assert_eq!(peak_top, SurfaceBlock::SnowBlock);

    let peak_ice = engine.get_block(0, peak_surface - 2, 0, peak_surface, BiomeId::FrozenPeaks, 0.0);
    assert_eq!(peak_ice, SurfaceBlock::PackedIce);
}

#[test]
fn test_surface_rules_desert_and_beach() {
    let engine = SurfaceRuleEngine::default();
    let surface_y = 66;

    // Top 3 layers: Sand
    for d in 0..3 {
        let block = engine.get_block(0, surface_y - d, 0, surface_y, BiomeId::Desert, 0.0);
        assert_eq!(block, SurfaceBlock::Sand, "Depth {} should be sand", d);
    }

    // Depth 3 & 4: Sandstone
    let sandstone1 = engine.get_block(0, surface_y - 3, 0, surface_y, BiomeId::Desert, 0.0);
    assert_eq!(sandstone1, SurfaceBlock::Sandstone);

    let sandstone2 = engine.get_block(0, surface_y - 4, 0, surface_y, BiomeId::Desert, 0.0);
    assert_eq!(sandstone2, SurfaceBlock::Sandstone);

    // Depth 5+: Stone
    let stone = engine.get_block(0, surface_y - 6, 0, surface_y, BiomeId::Desert, 0.0);
    assert_eq!(stone, SurfaceBlock::Stone);
}

#[test]
fn test_surface_rules_badlands_banded_terracotta() {
    let engine = SurfaceRuleEngine::new(42);
    let plateau_y = 100;

    // Badlands plateau surface and subsurface should produce Terracotta variants
    let surface = engine.get_block(10, plateau_y, 10, plateau_y, BiomeId::Badlands, 0.2);
    assert!(
        matches!(
            surface,
            SurfaceBlock::Terracotta
                | SurfaceBlock::WhiteTerracotta
                | SurfaceBlock::OrangeTerracotta
                | SurfaceBlock::YellowTerracotta
                | SurfaceBlock::BrownTerracotta
                | SurfaceBlock::RedTerracotta
                | SurfaceBlock::LightGrayTerracotta
        ),
        "Expected terracotta variant on Badlands plateau, got {:?}",
        surface
    );

    // Across vertical column Y=60..120, verify color banding varies with Y
    let mut observed_colors = std::collections::HashSet::new();
    for y in 60..120 {
        let block = engine.get_block(10, y, 10, plateau_y, BiomeId::Badlands, 0.0);
        observed_colors.insert(block);
    }

    // Must exhibit multiple colored terracotta strata
    assert!(
        observed_colors.len() >= 3,
        "Badlands column should contain at least 3 distinct terracotta band colors, got {}",
        observed_colors.len()
    );

    // Low valley Badlands (Y <= 65) should have Red Sand surface
    let valley_y = 64;
    let valley_top = engine.get_block(0, valley_y, 0, valley_y, BiomeId::Badlands, 0.0);
    assert_eq!(valley_top, SurfaceBlock::RedSand);

    let valley_sub = engine.get_block(0, valley_y - 2, 0, valley_y, BiomeId::Badlands, 0.0);
    assert_eq!(valley_sub, SurfaceBlock::RedSandstone);
}

#[test]
fn test_surface_rules_bedrock_and_oceans() {
    let engine = SurfaceRuleEngine::default();

    // Bedrock floor at MIN_Y (-64)
    let bedrock = engine.get_block(5, MIN_Y, 5, 70, BiomeId::Plains, 0.0);
    assert_eq!(bedrock, SurfaceBlock::Bedrock);

    // Ocean water filling from surface_y (e.g. 50) up to SEA_LEVEL (63)
    let ocean_surface_y = 50;
    let water_block = engine.get_block(0, 58, 0, ocean_surface_y, BiomeId::Ocean, 0.0);
    assert_eq!(water_block, SurfaceBlock::Water);

    // Frozen Ocean should have Ice at SEA_LEVEL (63)
    let ice_block = engine.get_block(0, SEA_LEVEL, 0, ocean_surface_y, BiomeId::FrozenOcean, 0.0);
    assert_eq!(ice_block, SurfaceBlock::Ice);

    // Subsurface below Y=0 is Deepslate
    let deepslate = engine.get_block(0, -20, 0, 70, BiomeId::Plains, 0.0);
    assert_eq!(deepslate, SurfaceBlock::Deepslate);
}

#[test]
fn test_badlands_column_exact_palette_sequence_y64_to_y120() {
    let engine = SurfaceRuleEngine::new(42);
    let plateau_y = 140;

    // Exact 57-block sequence evaluated from Minecraft 1.20.4 client.jar bytecode
    // (dpi.class / SurfaceSystem terracotta band algorithm with Xoroshiro128++ seed 42)
    let expected = [
        SurfaceBlock::Terracotta,          // 64
        SurfaceBlock::Terracotta,          // 65
        SurfaceBlock::Terracotta,          // 66
        SurfaceBlock::LightGrayTerracotta, // 67
        SurfaceBlock::WhiteTerracotta,     // 68
        SurfaceBlock::LightGrayTerracotta, // 69
        SurfaceBlock::Terracotta,          // 70
        SurfaceBlock::Terracotta,          // 71
        SurfaceBlock::OrangeTerracotta,    // 72
        SurfaceBlock::Terracotta,          // 73
        SurfaceBlock::Terracotta,          // 74
        SurfaceBlock::OrangeTerracotta,    // 75
        SurfaceBlock::Terracotta,          // 76
        SurfaceBlock::OrangeTerracotta,    // 77
        SurfaceBlock::Terracotta,          // 78
        SurfaceBlock::Terracotta,          // 79
        SurfaceBlock::OrangeTerracotta,    // 80
        SurfaceBlock::Terracotta,          // 81
        SurfaceBlock::Terracotta,          // 82
        SurfaceBlock::YellowTerracotta,    // 83
        SurfaceBlock::LightGrayTerracotta, // 84
        SurfaceBlock::WhiteTerracotta,     // 85
        SurfaceBlock::OrangeTerracotta,    // 86
        SurfaceBlock::YellowTerracotta,    // 87
        SurfaceBlock::Terracotta,          // 88
        SurfaceBlock::Terracotta,          // 89
        SurfaceBlock::Terracotta,          // 90
        SurfaceBlock::OrangeTerracotta,    // 91
        SurfaceBlock::Terracotta,          // 92
        SurfaceBlock::WhiteTerracotta,     // 93
        SurfaceBlock::RedTerracotta,       // 94
        SurfaceBlock::RedTerracotta,       // 95
        SurfaceBlock::Terracotta,          // 96
        SurfaceBlock::YellowTerracotta,    // 97
        SurfaceBlock::Terracotta,          // 98
        SurfaceBlock::Terracotta,          // 99
        SurfaceBlock::WhiteTerracotta,     // 100
        SurfaceBlock::Terracotta,          // 101
        SurfaceBlock::Terracotta,          // 102
        SurfaceBlock::Terracotta,          // 103
        SurfaceBlock::Terracotta,          // 104
        SurfaceBlock::Terracotta,          // 105
        SurfaceBlock::OrangeTerracotta,    // 106
        SurfaceBlock::Terracotta,          // 107
        SurfaceBlock::Terracotta,          // 108
        SurfaceBlock::Terracotta,          // 109
        SurfaceBlock::OrangeTerracotta,    // 110
        SurfaceBlock::Terracotta,          // 111
        SurfaceBlock::LightGrayTerracotta, // 112
        SurfaceBlock::WhiteTerracotta,     // 113
        SurfaceBlock::Terracotta,          // 114
        SurfaceBlock::Terracotta,          // 115
        SurfaceBlock::OrangeTerracotta,    // 116
        SurfaceBlock::Terracotta,          // 117
        SurfaceBlock::Terracotta,          // 118
        SurfaceBlock::Terracotta,          // 119
        SurfaceBlock::OrangeTerracotta,    // 120
    ];

    for (idx, &expected_block) in expected.iter().enumerate() {
        let y = 64 + idx as i32;
        let block = engine.get_block(0, y, 0, plateau_y, BiomeId::Badlands, 0.0);
        assert_eq!(
            block, expected_block,
            "Badlands terracotta mismatch at Y={}: expected {:?}, got {:?}",
            y, expected_block, block
        );
    }
}

#[test]
fn test_surface_rules_steep_and_hole_conditions() {
    let engine = SurfaceRuleEngine::default();

    // 1. Steep cliff face on Plains replaces GrassBlock with bare Stone
    use mc_worldgen::biomes::SurfaceContext;
    let normal_ctx = SurfaceContext::new(0, 70, 0, 70, BiomeId::Plains, 0.0);
    assert_eq!(engine.get_block_with_context(&normal_ctx), SurfaceBlock::GrassBlock);

    let steep_ctx = normal_ctx.with_steep(true);
    assert_eq!(
        engine.get_block_with_context(&steep_ctx),
        SurfaceBlock::Stone,
        "Steep cliff face must replace surface grass with bare stone"
    );

    // Steep + Hole places Gravel
    let steep_hole_ctx = steep_ctx.with_hole(true);
    assert_eq!(
        engine.get_block_with_context(&steep_hole_ctx),
        SurfaceBlock::Gravel,
        "Steep cliff face with hole pocket must generate gravel"
    );

    // 2. Underwater depression / hole pocket generates Gravel
    let ocean_hole_ctx = SurfaceContext::new(0, 50, 0, 50, BiomeId::Ocean, 0.0).with_hole(true);
    assert_eq!(
        engine.get_block_with_context(&ocean_hole_ctx),
        SurfaceBlock::Gravel,
        "Underwater depression pocket in ocean must place gravel"
    );
}

#[test]
fn test_surface_rules_vertical_gradient_bedrock() {
    let engine = SurfaceRuleEngine::new(42);

    // Bedrock floor at MIN_Y (-64) is always 100% solid Bedrock
    for x in -5..=5 {
        for z in -5..=5 {
            let block = engine.get_block(x, -64, z, 70, BiomeId::Plains, 0.0);
            assert_eq!(block, SurfaceBlock::Bedrock, "Y=-64 must be bedrock at ({}, {})", x, z);
        }
    }

    // At Y >= -58, vertical gradient for bedrock floor is 0% (false), so no bedrock
    for x in -5..=5 {
        for z in -5..=5 {
            let block = engine.get_block(x, -58, z, 70, BiomeId::Plains, 0.0);
            assert_ne!(block, SurfaceBlock::Bedrock, "Y=-58 must not be bedrock");
        }
    }

    // Between Y=-63 and Y=-59, transition band contains both Bedrock and Deepslate
    let mut bedrock_count = 0;
    let mut deepslate_count = 0;
    for x in -10..=10 {
        for z in -10..=10 {
            for y in -63..=-60 {
                let block = engine.get_block(x, y, z, 70, BiomeId::Plains, 0.0);
                if block == SurfaceBlock::Bedrock {
                    bedrock_count += 1;
                } else if block == SurfaceBlock::Deepslate {
                    deepslate_count += 1;
                }
            }
        }
    }
    assert!(
        bedrock_count > 0 && deepslate_count > 0,
        "Bedrock transition band (-63..=-60) must have a probabilistic mix of Bedrock and Deepslate (bedrock={}, deepslate={})",
        bedrock_count, deepslate_count
    );
}
