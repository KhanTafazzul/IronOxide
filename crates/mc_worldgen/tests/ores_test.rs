//! Unit and integration test suite for ore vein generation and distribution trapezoids.
//! REA Reference: net.minecraft.world.level.levelgen.feature.OreFeature (dsp.class)
//! REA Reference: net.minecraft.world.level.levelgen.placement.HeightRangePlacement (dyf.class)
//! REA Reference: net.minecraft.data.worldgen.features.OreFeatures (qm.class)
//! REA Reference: net.minecraft.data.worldgen.placement.OrePlacements (qx.class)

use mc_worldgen::biomes::{BiomeId, SurfaceBlock};
use mc_worldgen::ores::{
    coal_lower_distribution, coal_upper_distribution, diamond_distribution,
    gold_badlands_extra_distribution, gold_regular_distribution, iron_middle_distribution,
    iron_upper_distribution, redstone_lower_distribution, redstone_regular_distribution,
    ChunkOreBuffer, OreBlobPlacer, OreBlockAccess, OreGenerator, OreType,
};
use mc_worldgen::structures::LegacyRandom;

#[test]
fn test_ellipsoid_blob_placer_connected_cluster_and_replacement() {
    let mut buffer = ChunkOreBuffer::new(0, 0, 0, 16, 16, 16, SurfaceBlock::Stone);

    // Populate buffer with mixed rock and non-replaceable materials
    // Deepslate in lower half, Stone in upper half, Bedrock at floor, and some Air
    for x in 0..16 {
        for z in 0..16 {
            buffer.set_block(x, 0, z, SurfaceBlock::Bedrock);
            for y in 1..8 {
                buffer.set_block(x, y, z, SurfaceBlock::Deepslate);
            }
            for y in 8..15 {
                buffer.set_block(x, y, z, SurfaceBlock::Stone);
            }
            buffer.set_block(x, 15, z, SurfaceBlock::Air);
        }
    }

    let mut rng = LegacyRandom::new(12345);

    // Place one vein in the upper Stone layer
    let stone_placed = OreBlobPlacer::place_blob(
        &mut buffer,
        (8, 11, 8),
        8,
        OreType::Diamond,
        &mut rng,
    );

    // Place one vein in the lower Deepslate layer near bedrock
    let deepslate_placed = OreBlobPlacer::place_blob(
        &mut buffer,
        (8, 3, 8),
        8,
        OreType::Diamond,
        &mut rng,
    );

    assert!(stone_placed > 0, "Blob placer should substitute Stone with DiamondOre");
    assert!(deepslate_placed > 0, "Blob placer should substitute Deepslate with DeepslateDiamondOre");

    let mut stone_diamond_count = 0;
    let mut deepslate_diamond_count = 0;

    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                let block = buffer.get_block(x, y, z);
                if block == SurfaceBlock::DiamondOre {
                    stone_diamond_count += 1;
                    // DiamondOre should only appear in the Stone layer (Y >= 8)
                    assert!(y >= 8, "DiamondOre should only replace Stone at Y >= 8");
                } else if block == SurfaceBlock::DeepslateDiamondOre {
                    deepslate_diamond_count += 1;
                    // DeepslateDiamondOre should only appear in Deepslate layer (Y < 8)
                    assert!(y < 8 && y > 0, "DeepslateDiamondOre should only replace Deepslate at 0 < Y < 8");
                }

                // Strict Bedrock immunity check
                if y == 0 {
                    assert_eq!(block, SurfaceBlock::Bedrock, "Bedrock must NEVER be replaced by ore placer");
                }
                // Air must not be replaced
                if y == 15 {
                    assert_eq!(block, SurfaceBlock::Air, "Air must NEVER be replaced by ore placer");
                }
            }
        }
    }

    assert!(stone_diamond_count > 0, "Must have placed Stone DiamondOre variants");
    assert!(deepslate_diamond_count > 0, "Must have placed DeepslateDiamondOre variants");
    assert_eq!(stone_placed + deepslate_placed, stone_diamond_count + deepslate_diamond_count);
}

#[test]
fn test_coal_vertical_distribution_curves() {
    let upper = coal_upper_distribution();
    let lower = coal_lower_distribution();

    let mut rng = LegacyRandom::new(98765);

    // Test Upper Coal: Uniform Y = 136 to 256
    for _ in 0..500 {
        let y = upper.sample(&mut rng);
        assert!(y >= 136 && y <= 256, "Upper coal must stay within Y in [136, 256], got {}", y);
    }

    // Test Lower Coal: Triangular Y = 0 to 192 with peak at Y = 96
    for _ in 0..500 {
        let y = lower.sample(&mut rng);
        assert!(y >= 0 && y <= 192, "Lower coal must stay within Y in [0, 192], got {}", y);
    }

    // Verify mathematical peak at Y = 96
    let peak_density = lower.probability_density(96);
    let edge_density_low = lower.probability_density(10);
    let edge_density_high = lower.probability_density(180);

    assert!(peak_density > edge_density_low * 3.0, "Coal lower density at peak Y=96 must be much higher than at Y=10");
    assert!(peak_density > edge_density_high * 3.0, "Coal lower density at peak Y=96 must be much higher than at Y=180");
}

#[test]
fn test_iron_vertical_distribution_curves() {
    let middle = iron_middle_distribution();
    let upper = iron_upper_distribution();

    let mut rng = LegacyRandom::new(54321);

    // Test Iron Middle: Triangular Y = -24 to 56 with peak at Y = 16
    for _ in 0..500 {
        let y = middle.sample(&mut rng);
        assert!(y >= -24 && y <= 56, "Iron middle must stay within Y in [-24, 56], got {}", y);
    }

    let mid_peak_density = middle.probability_density(16);
    let mid_low_density = middle.probability_density(-20);
    let mid_high_density = middle.probability_density(50);

    assert!(mid_peak_density > mid_low_density * 3.0, "Iron middle peak at Y=16 must exceed edges");
    assert!(mid_peak_density > mid_high_density * 3.0, "Iron middle peak at Y=16 must exceed edges");

    // Test Iron Upper: Peaks in high mountain summits
    let upper_peak_density = upper.probability_density(232);
    let upper_mountain_y256 = upper.probability_density(256);
    assert!(upper_mountain_y256 > 0.003, "Iron upper must generate abundantly at mountain level Y=256");
    assert!(upper_peak_density >= upper_mountain_y256);
}

#[test]
fn test_gold_distribution_and_badlands_extra_abundance() {
    let regular = gold_regular_distribution();
    let badlands_extra = gold_badlands_extra_distribution();

    let mut rng = LegacyRandom::new(777);

    // Regular gold: Uniform -64 to -32
    for _ in 0..200 {
        let y = regular.sample(&mut rng);
        assert!(y >= -64 && y <= -32, "Regular gold must be between -64 and -32, got {}", y);
    }

    // Badlands extra gold: Uniform 32 to 256
    for _ in 0..200 {
        let y = badlands_extra.sample(&mut rng);
        assert!(y >= 32 && y <= 256, "Badlands extra gold must be between 32 and 256, got {}", y);
    }

    // Simulate chunk generation in Badlands vs Plains
    let generator = OreGenerator::vanilla_overworld();

    // Create a tall stone column buffer (Y = -64 to 256)
    let mut badlands_chunk = ChunkOreBuffer::new(0, -64, 0, 16, 320, 16, SurfaceBlock::Stone);
    let mut plains_chunk = ChunkOreBuffer::new(0, -64, 0, 16, 320, 16, SurfaceBlock::Stone);

    let seed = 424242;
    let _ = generator.generate_chunk(&mut badlands_chunk, 0, 0, BiomeId::Badlands, seed);
    let _ = generator.generate_chunk(&mut plains_chunk, 0, 0, BiomeId::Plains, seed);

    // Count gold blocks in each chunk
    let mut badlands_gold = 0;
    let mut plains_gold = 0;

    for x in 0..16 {
        for y in -64..256 {
            for z in 0..16 {
                let b_block = badlands_chunk.get_block(x, y, z);
                if b_block == SurfaceBlock::GoldOre || b_block == SurfaceBlock::DeepslateGoldOre {
                    badlands_gold += 1;
                }

                let p_block = plains_chunk.get_block(x, y, z);
                if p_block == SurfaceBlock::GoldOre || p_block == SurfaceBlock::DeepslateGoldOre {
                    plains_gold += 1;
                }
            }
        }
    }

    assert!(badlands_gold > plains_gold * 3,
        "Badlands must generate dramatically higher gold abundance than Plains (Badlands: {}, Plains: {})",
        badlands_gold, plains_gold
    );
}

#[test]
fn test_redstone_vertical_distribution_density_increasing_toward_bedrock() {
    let regular = redstone_regular_distribution();
    let lower = redstone_lower_distribution();

    let mut rng = LegacyRandom::new(13579);

    // Regular redstone: Uniform -64 to 15
    for _ in 0..300 {
        let y = regular.sample(&mut rng);
        assert!(y >= -64 && y <= 15, "Regular redstone must be between -64 and 15, got {}", y);
    }

    // Lower redstone: Increasing density toward bedrock floor (Y = -64)
    let density_near_bedrock = lower.probability_density(-64);
    let density_mid = lower.probability_density(-48);
    let density_top = lower.probability_density(-34);

    assert!(density_near_bedrock > density_mid, "Redstone lower density must peak toward bedrock (Y=-64)");
    assert!(density_mid > density_top, "Redstone lower density must decrease moving upward away from bedrock");
}

#[test]
fn test_diamond_triangular_distribution_peak_at_bedrock() {
    let diamond = diamond_distribution();

    // Diamond triangular distribution spans down from Y=16 to bedrock level Y=-64
    let density_bedrock = diamond.probability_density(-64);
    let density_y_minus_32 = diamond.probability_density(-32);
    let density_y_0 = diamond.probability_density(0);
    let density_y_16 = diamond.probability_density(16);
    let density_above_16 = diamond.probability_density(20);

    assert_eq!(density_above_16, 0.0, "Diamond density above Y=16 must be 0");
    assert!(density_bedrock > density_y_minus_32, "Diamond density at bedrock must be higher than at Y=-32");
    assert!(density_y_minus_32 > density_y_0, "Diamond density at Y=-32 must be higher than at Y=0");
    assert!(density_y_0 > density_y_16, "Diamond density at Y=0 must be higher than at Y=16");

    // Sample diamond heights and ensure maximum concentration is below Y=0
    let mut rng = LegacyRandom::new(999);
    let mut deep_diamonds = 0;
    let mut shallow_diamonds = 0;

    for _ in 0..1000 {
        let y = diamond.sample(&mut rng);
        // Diamond samples are clamped to the playable world range [-64, 16]
        let clamped_y = y.clamp(-64, 16);
        if clamped_y < 0 {
            deep_diamonds += 1;
        } else {
            shallow_diamonds += 1;
        }
    }

    assert!(deep_diamonds > shallow_diamonds * 2,
        "Diamonds must generate predominantly deep underground below Y=0 (Deep: {}, Shallow: {})",
        deep_diamonds, shallow_diamonds
    );
}

#[test]
fn test_copper_vertical_distribution_and_dripstone_doubling() {
    use mc_worldgen::ores::{copper_dripstone_distribution, copper_regular_distribution};

    let regular = copper_regular_distribution();
    let dripstone = copper_dripstone_distribution();

    let mut rng = LegacyRandom::new(12345);

    // Copper: Triangular -16 to 112 with peak at Y=48
    for _ in 0..500 {
        let y = regular.sample(&mut rng);
        assert!(y >= -16 && y <= 112, "Copper must stay within [-16, 112], got {}", y);
    }

    let peak_density = regular.probability_density(48);
    let low_density = regular.probability_density(-10);
    let high_density = regular.probability_density(100);

    assert!(peak_density > low_density * 2.5, "Copper density at peak Y=48 must exceed Y=-10");
    assert!(peak_density > high_density * 2.5, "Copper density at peak Y=48 must exceed Y=100");

    // Dripstone caves distribution has matching elevation range
    assert_eq!(regular, dripstone);
}

#[test]
fn test_lapis_lazuli_triangular_distribution_peak_at_zero() {
    use mc_worldgen::ores::lapis_regular_distribution;

    let lapis = lapis_regular_distribution();
    let mut rng = LegacyRandom::new(54321);

    // Lapis regular: Triangular -32 to 32 with peak at Y=0
    for _ in 0..500 {
        let y = lapis.sample(&mut rng);
        assert!(y >= -32 && y <= 32, "Lapis must stay within [-32, 32], got {}", y);
    }

    let peak_density = lapis.probability_density(0);
    let low_density = lapis.probability_density(-28);
    let high_density = lapis.probability_density(28);

    assert!(peak_density > low_density * 3.0, "Lapis density at peak Y=0 must exceed Y=-28");
    assert!(peak_density > high_density * 3.0, "Lapis density at peak Y=0 must exceed Y=28");
}

#[test]
fn test_gold_regular_triangular_distribution_peak_at_minus_16() {
    use mc_worldgen::ores::gold_regular_triangular_distribution;

    let gold = gold_regular_triangular_distribution();
    let mut rng = LegacyRandom::new(998877);

    // Gold triangular: -64 to 32 with peak at Y = (-64 + 32)/2 = -16
    for _ in 0..500 {
        let y = gold.sample(&mut rng);
        assert!(y >= -64 && y <= 32, "Triangular gold must stay within [-64, 32], got {}", y);
    }

    let peak_density = gold.probability_density(-16);
    let low_density = gold.probability_density(-60);
    let high_density = gold.probability_density(25);

    assert!(peak_density > low_density * 2.5, "Gold density at peak Y=-16 must exceed Y=-60");
    assert!(peak_density > high_density * 2.5, "Gold density at peak Y=-16 must exceed Y=25");
}

#[test]
fn test_diamond_reduced_air_exposure_rule() {
    use mc_worldgen::features::ores::{ChunkOreBuffer, OreBlobPlacer, OreType};

    // Buffer 1: Solid stone (no air exposure)
    let mut solid_buf = ChunkOreBuffer::new(0, 0, 0, 16, 16, 16, SurfaceBlock::Stone);
    let mut rng1 = LegacyRandom::new(42);
    let placed_solid = OreBlobPlacer::place_blob_with_air_exposure(
        &mut solid_buf,
        (8, 8, 8),
        8,
        OreType::Diamond,
        0.5,
        &mut rng1,
    );

    // Buffer 2: Center hollowed out with Air (lots of air exposure)
    let mut exposed_buf = ChunkOreBuffer::new(0, 0, 0, 16, 16, 16, SurfaceBlock::Stone);
    for x in 6..=10 {
        for y in 6..=10 {
            for z in 6..=10 {
                exposed_buf.set_block(x, y, z, SurfaceBlock::Air);
            }
        }
    }
    let mut rng2 = LegacyRandom::new(42);
    let placed_exposed = OreBlobPlacer::place_blob_with_air_exposure(
        &mut exposed_buf,
        (8, 8, 8),
        8,
        OreType::Diamond,
        0.5,
        &mut rng2,
    );

    // Air exposure discard rule ensures significantly fewer diamonds placed when exposed to air caves
    assert!(
        placed_exposed < placed_solid,
        "Air-exposed diamond vein ({}) must place fewer blocks than solid vein ({}) due to discard chance",
        placed_exposed, placed_solid
    );
}

#[test]
fn test_features_ores_module_parity() {
    use mc_worldgen::features::ores::{OreGenerator, OreType};

    let generator = OreGenerator::vanilla_overworld();
    assert_eq!(generator.configs.len(), 17, "Vanilla ore generator must configure 17 distinct ore feature passes");

    let has_copper = generator.configs.iter().any(|c| c.ore_type == OreType::Copper);
    let has_lapis = generator.configs.iter().any(|c| c.ore_type == OreType::Lapis);
    let has_diamond = generator.configs.iter().any(|c| c.ore_type == OreType::Diamond && c.discard_chance_on_air_exposure > 0.0);

    assert!(has_copper, "Must configure Copper ore");
    assert!(has_lapis, "Must configure Lapis Lazuli ore");
    assert!(has_diamond, "Must configure Diamond ore with air discard rule");
}
