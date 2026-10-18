//! Unit tests for Surface Rule Engine AST and Biome Strata.
//! REA Reference: net.minecraft.world.level.levelgen.SurfaceRules (dph.class)
//! REA Reference: net.minecraft.world.level.levelgen.SurfaceSystem (dpi.class)

use mc_worldgen::biomes::{BiomeId, SurfaceBlock};
use mc_worldgen::surface::{Condition, SurfaceContext, SurfaceSystem};

#[test]
fn test_surface_ast_condition_evaluation() {
    let ctx_surface = SurfaceContext::new(0, 70, 0, 70, BiomeId::Plains, 0.0);
    let ctx_subsurface = SurfaceContext::new(0, 68, 0, 70, BiomeId::Plains, 0.0);
    let ctx_underwater = SurfaceContext::new(0, 50, 0, 50, BiomeId::Ocean, 0.0);

    // 1. Condition::AbovePreliminarySurface
    let cond_top = Condition::AbovePreliminarySurface {
        offset: 0,
        add_surface_depth: false,
    };
    assert!(cond_top.test(&ctx_surface), "Depth 0 must match top surface");
    assert!(!cond_top.test(&ctx_subsurface), "Depth 2 must not match top surface offset 0");

    let cond_soil = Condition::AbovePreliminarySurface {
        offset: 3,
        add_surface_depth: false,
    };
    assert!(cond_soil.test(&ctx_subsurface), "Depth 2 must match offset 3");

    // 2. Condition::WaterCheck
    let cond_water = Condition::WaterCheck { offset: 0 };
    assert!(cond_water.test(&ctx_underwater), "Y=50 is under sea level 63");
    assert!(!cond_water.test(&ctx_surface), "Y=70 is above sea level 63");

    // 3. Condition::Biome
    let cond_plains = Condition::Biome(vec![BiomeId::Plains, BiomeId::Forest]);
    assert!(cond_plains.test(&ctx_surface), "Plains context must match plains condition");
    assert!(!cond_plains.test(&ctx_underwater), "Ocean context must not match plains condition");

    // 4. Logical Combinators
    let cond_all = Condition::All(vec![cond_top.clone(), cond_plains.clone()]);
    assert!(cond_all.test(&ctx_surface));

    let cond_not = Condition::Not(Box::new(cond_water));
    assert!(cond_not.test(&ctx_surface));
}

#[test]
fn test_standard_strata_grass_over_dirt_over_stone() {
    let system = SurfaceSystem::default();
    let surface_y = 70; // Above sea level (63)

    // 1 layer GrassBlock on top
    let top = system.get_surface_block(0, 70, 0, surface_y, BiomeId::Plains, 0.0);
    assert_eq!(top, SurfaceBlock::GrassBlock, "Surface block must be GrassBlock");

    // 3-4 layers Dirt subsurface
    let dirt1 = system.get_surface_block(0, 69, 0, surface_y, BiomeId::Plains, 0.0);
    let dirt2 = system.get_surface_block(0, 68, 0, surface_y, BiomeId::Plains, 0.0);
    let dirt3 = system.get_surface_block(0, 67, 0, surface_y, BiomeId::Plains, 0.0);
    assert_eq!(dirt1, SurfaceBlock::Dirt);
    assert_eq!(dirt2, SurfaceBlock::Dirt);
    assert_eq!(dirt3, SurfaceBlock::Dirt);

    // Deep rock foundation: Stone above Y=0, Deepslate below Y=0
    let stone = system.get_surface_block(0, 50, 0, surface_y, BiomeId::Plains, 0.0);
    assert_eq!(stone, SurfaceBlock::Stone, "Base rock above Y=0 must be Stone");

    let deepslate = system.get_surface_block(0, -10, 0, surface_y, BiomeId::Plains, 0.0);
    assert_eq!(deepslate, SurfaceBlock::Deepslate, "Base rock below Y=0 must be Deepslate");
}

#[test]
fn test_underwater_beaches_sand_over_sandstone() {
    let system = SurfaceSystem::default();
    let surface_y = 62; // Beach / Shallow ocean floor

    // Top layers: Sand
    let sand1 = system.get_surface_block(0, 62, 0, surface_y, BiomeId::Beach, 0.0);
    let sand2 = system.get_surface_block(0, 61, 0, surface_y, BiomeId::Beach, 0.0);
    let sand3 = system.get_surface_block(0, 60, 0, surface_y, BiomeId::Beach, 0.0);
    assert_eq!(sand1, SurfaceBlock::Sand);
    assert_eq!(sand2, SurfaceBlock::Sand);
    assert_eq!(sand3, SurfaceBlock::Sand);

    // Subsurface: Sandstone
    let sandstone = system.get_surface_block(0, 58, 0, surface_y, BiomeId::Beach, 0.0);
    assert_eq!(sandstone, SurfaceBlock::Sandstone);

    // Base rock: Stone
    let stone = system.get_surface_block(0, 50, 0, surface_y, BiomeId::Beach, 0.0);
    assert_eq!(stone, SurfaceBlock::Stone);
}

#[test]
fn test_desert_strata_deep_sand_and_sandstone() {
    let system = SurfaceSystem::default();
    let surface_y = 75;

    // Top 3-4 layers: Sand
    for y in (72..=75).rev() {
        let block = system.get_surface_block(0, y, 0, surface_y, BiomeId::Desert, 0.0);
        assert_eq!(block, SurfaceBlock::Sand, "Desert surface layer at Y={} must be Sand", y);
    }

    // Subsurface: Sandstone (depth 4..8)
    for y in (68..=71).rev() {
        let block = system.get_surface_block(0, y, 0, surface_y, BiomeId::Desert, 0.0);
        assert_eq!(block, SurfaceBlock::Sandstone, "Desert subsurface layer at Y={} must be Sandstone", y);
    }
}

#[test]
fn test_badlands_mesa_terracotta_banding() {
    let system = SurfaceSystem::new(12345);

    // Valleys below Y=65: Red Sand over Red Sandstone
    let red_sand = system.get_surface_block(0, 64, 0, 64, BiomeId::Badlands, 0.0);
    assert_eq!(red_sand, SurfaceBlock::RedSand);

    let red_sandstone = system.get_surface_block(0, 62, 0, 64, BiomeId::Badlands, 0.0);
    assert_eq!(red_sandstone, SurfaceBlock::RedSandstone);

    // High plateau: Terracotta surface over colored Terracotta bands
    let plateau_y = 90;
    let surface_terracotta = system.get_surface_block(0, 90, 0, plateau_y, BiomeId::Badlands, 0.0);
    assert_eq!(surface_terracotta, SurfaceBlock::Terracotta);

    // Subsurface terracotta banding: samples distinct terracotta colors across Y levels
    let mut colors = std::collections::HashSet::new();
    for y in 70..89 {
        let band = system.get_surface_block(0, y, 0, plateau_y, BiomeId::Badlands, 0.0);
        match band {
            SurfaceBlock::Terracotta
            | SurfaceBlock::WhiteTerracotta
            | SurfaceBlock::OrangeTerracotta
            | SurfaceBlock::YellowTerracotta
            | SurfaceBlock::BrownTerracotta
            | SurfaceBlock::RedTerracotta
            | SurfaceBlock::LightGrayTerracotta => {
                colors.insert(band);
            }
            _ => panic!("Expected terracotta band block at Y={}, got {:?}", y, band),
        }
    }
    assert!(colors.len() >= 3, "Badlands must evaluate multiple terracotta band colors, got {}", colors.len());
}

#[test]
fn test_mountain_peaks_exposed_stone_gravel_snow() {
    let system = SurfaceSystem::default();

    // 1. Stony Peaks: Exposed Stone & Gravel
    let stony_surface = system.get_surface_block(0, 120, 0, 120, BiomeId::StonyPeaks, 0.0);
    let stony_sub = system.get_surface_block(0, 119, 0, 120, BiomeId::StonyPeaks, 0.0);
    assert_eq!(stony_surface, SurfaceBlock::Gravel);
    assert_eq!(stony_sub, SurfaceBlock::Gravel);

    let stony_deep = system.get_surface_block(0, 115, 0, 120, BiomeId::StonyPeaks, 0.0);
    assert_eq!(stony_deep, SurfaceBlock::Stone);

    // 2. Jagged Peaks: SnowBlock surface over PackedIce over Stone
    let jagged_top = system.get_surface_block(0, 140, 0, 140, BiomeId::JaggedPeaks, 0.0);
    assert_eq!(jagged_top, SurfaceBlock::SnowBlock);

    let jagged_ice = system.get_surface_block(0, 138, 0, 140, BiomeId::JaggedPeaks, 0.0);
    assert_eq!(jagged_ice, SurfaceBlock::PackedIce);

    let jagged_stone = system.get_surface_block(0, 130, 0, 140, BiomeId::JaggedPeaks, 0.0);
    assert_eq!(jagged_stone, SurfaceBlock::Stone);
}
