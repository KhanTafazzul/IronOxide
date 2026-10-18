//! Unit and integration test suite for procedural tree and foliage placers.
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.StraightTrunkPlacer (dxb.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer (dwx.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.foliageplacers.BlobFoliagePlacer (dvf.class)
//! REA Reference: net.minecraft.world.level.levelgen.feature.foliageplacers.SpruceFoliagePlacer (dvq.class)

use mc_worldgen::biomes::SurfaceBlock;
use mc_worldgen::ores::{ChunkOreBuffer, OreBlockAccess};
use mc_worldgen::structures::LegacyRandom;
use mc_worldgen::vegetation::{
    BlobFoliagePlacer, BranchingMegaTrunkPlacer, FoliageAttachment, GroundCoverConfig,
    GroundCoverPlacer, SpruceFoliagePlacer, StraightTrunkPlacer, TreePlacer, TreeType,
};

#[test]
fn test_straight_trunk_placer_oak_and_birch() {
    let mut buffer = ChunkOreBuffer::new(0, 0, 0, 16, 24, 16, SurfaceBlock::Air);
    buffer.set_block(8, 3, 8, SurfaceBlock::GrassBlock);

    let trunk = StraightTrunkPlacer::new(4, 2, 0); // 4-6 blocks height
    let mut rng = LegacyRandom::new(12345);

    let (attachments, logs_placed) = trunk.place_trunk(
        &mut buffer,
        (8, 4, 8),
        SurfaceBlock::OakLog,
        &mut rng,
    );

    assert!(logs_placed >= 4 && logs_placed <= 6, "Oak straight trunk must be 4-6 blocks, got {}", logs_placed);
    assert_eq!(attachments.len(), 1, "Straight trunk must produce exactly 1 foliage attachment at summit");
    assert_eq!(attachments[0].x, 8);
    assert_eq!(attachments[0].z, 8);
    assert_eq!(attachments[0].y, 4 + logs_placed as i32);

    // Verify soil beneath was converted to Dirt
    assert_eq!(buffer.get_block(8, 3, 8), SurfaceBlock::Dirt);

    // Verify vertical column of OakLog
    for y in 4..(4 + logs_placed as i32) {
        assert_eq!(buffer.get_block(8, y, 8), SurfaceBlock::OakLog);
    }
}

#[test]
fn test_branching_mega_trunk_placer_dark_oak_and_jungle() {
    let mut buffer = ChunkOreBuffer::new(0, 0, 0, 24, 32, 24, SurfaceBlock::Air);

    // Prepare 2x2 grass soil at Y=3
    for dx in 0..2 {
        for dz in 0..2 {
            buffer.set_block(10 + dx, 3, 10 + dz, SurfaceBlock::GrassBlock);
        }
    }

    let mega_trunk = BranchingMegaTrunkPlacer::new(8, 2, 1);
    let mut rng = LegacyRandom::new(99999);

    let (attachments, logs_placed) = mega_trunk.place_trunk(
        &mut buffer,
        (10, 4, 10),
        SurfaceBlock::DarkOakLog,
        &mut rng,
    );

    // Verify dirt under all 4 base positions
    for dx in 0..2 {
        for dz in 0..2 {
            assert_eq!(buffer.get_block(10 + dx, 3, 10 + dz), SurfaceBlock::Dirt);
        }
    }

    // Verify main 2x2 pillar logs
    for y in 4..12 {
        for dx in 0..2 {
            for dz in 0..2 {
                assert_eq!(buffer.get_block(10 + dx, y, 10 + dz), SurfaceBlock::DarkOakLog);
            }
        }
    }

    // Must have placed main 2x2 pillar + branching logs
    assert!(logs_placed > 32, "Mega trunk must place 2x2 logs plus branching arms, placed: {}", logs_placed);

    // Multiple foliage attachments (summit + branch tips)
    assert!(attachments.len() >= 3, "Branching trunk must have multiple attachments (got {})", attachments.len());
    assert!(attachments[0].is_giant_trunk, "Summit attachment must be marked as giant trunk");
}

#[test]
fn test_blob_foliage_placer_spherical_clusters_and_corner_decay() {
    let mut buffer = ChunkOreBuffer::new(0, 0, 0, 16, 16, 16, SurfaceBlock::Air);
    let foliage = BlobFoliagePlacer::new(2, 3);
    let mut rng = LegacyRandom::new(54321);

    let attachment = FoliageAttachment::new(8, 8, 8, 0, false);
    let leaves_placed = foliage.place_foliage(
        &mut buffer,
        &attachment,
        SurfaceBlock::OakLeaves,
        &mut rng,
    );

    assert!(leaves_placed > 15, "Blob foliage should place a dense spherical cluster (placed: {})", leaves_placed);

    // Center of foliage must be solid leaves
    assert_eq!(buffer.get_block(8, 8, 8), SurfaceBlock::OakLeaves);
    assert_eq!(buffer.get_block(8, 7, 8), SurfaceBlock::OakLeaves);
    assert_eq!(buffer.get_block(8, 6, 8), SurfaceBlock::OakLeaves);

    // Top layer (Y=8) corners at (8±2, 8, 8±2) must decay
    let top_corner = buffer.get_block(10, 8, 10);
    assert_eq!(top_corner, SurfaceBlock::Air, "Top corners of blob foliage must always decay");
}

#[test]
fn test_spruce_foliage_placer_stepped_cone_shape() {
    let mut buffer = ChunkOreBuffer::new(0, 0, 0, 16, 20, 16, SurfaceBlock::Air);
    let spruce_foliage = SpruceFoliagePlacer::new(2, 6);
    let mut rng = LegacyRandom::new(777);

    let attachment = FoliageAttachment::new(8, 12, 8, 0, false);
    let leaves_placed = spruce_foliage.place_foliage(
        &mut buffer,
        &attachment,
        SurfaceBlock::SpruceLeaves,
        &mut rng,
    );

    assert!(leaves_placed > 20, "Spruce foliage must place conical leaf layers, placed: {}", leaves_placed);

    // Verify top apex at Y=12 is narrow
    assert_eq!(buffer.get_block(8, 12, 8), SurfaceBlock::SpruceLeaves);

    // Verify wider layers near bottom
    let mut bottom_leaf_count = 0;
    for dx in -2..=2 {
        for dz in -2..=2 {
            if buffer.get_block(8 + dx, 7, 8 + dz) == SurfaceBlock::SpruceLeaves {
                bottom_leaf_count += 1;
            }
        }
    }
    assert!(bottom_leaf_count > 5, "Spruce foliage base layer must be wider than summit");
}

#[test]
fn test_tree_placer_full_tree_generation() {
    let mut buffer = ChunkOreBuffer::new(0, 0, 0, 32, 40, 32, SurfaceBlock::Air);
    let mut rng = LegacyRandom::new(10101);

    // Valid soil for Oak at (8, 4, 8)
    buffer.set_block(8, 3, 8, SurfaceBlock::GrassBlock);
    let (oak_logs, oak_leaves) = TreePlacer::place_tree(&mut buffer, (8, 4, 8), TreeType::Oak, &mut rng);
    assert!(oak_logs >= 4);
    assert!(oak_leaves >= 15);

    // Valid soil for Spruce at (16, 4, 16)
    buffer.set_block(16, 3, 16, SurfaceBlock::Podzol);
    let (spruce_logs, spruce_leaves) = TreePlacer::place_tree(&mut buffer, (16, 4, 16), TreeType::Spruce, &mut rng);
    assert!(spruce_logs >= 6);
    assert!(spruce_leaves >= 18);

    // Invalid soil (Stone) should be rejected
    buffer.set_block(24, 3, 24, SurfaceBlock::Stone);
    let (invalid_logs, invalid_leaves) = TreePlacer::place_tree(&mut buffer, (24, 4, 24), TreeType::Oak, &mut rng);
    assert_eq!(invalid_logs, 0);
    assert_eq!(invalid_leaves, 0);
}

#[test]
fn test_ground_cover_scatter_pass_distribution() {
    // 16x16 chunk column with GrassBlock surface at Y=10
    let mut buffer = ChunkOreBuffer::new(0, 0, 0, 16, 20, 16, SurfaceBlock::Air);
    for x in 0..16 {
        for z in 0..16 {
            buffer.set_block(x, 9, z, SurfaceBlock::Dirt);
            buffer.set_block(x, 10, z, SurfaceBlock::GrassBlock);
        }
    }

    let config = GroundCoverConfig {
        attempts_per_chunk: 20,
        xz_spread: 6,
        y_spread: 1,
    };

    let heightmap = |_x: i32, _z: i32| 11; // Air block directly above GrassBlock
    let placed = GroundCoverPlacer::scatter_chunk(
        &mut buffer,
        0,
        0,
        &heightmap,
        1234567,
        &config,
    );

    assert!(placed > 0, "Ground cover scatter pass should place decorations on grass surfaces");

    let mut found_short_grass = 0;
    let mut found_tall_grass = 0;
    let mut found_poppy = 0;
    let mut found_dandelion = 0;

    for x in 0..16 {
        for z in 0..16 {
            for y in 10..15 {
                let block = buffer.get_block(x, y, z);
                match block {
                    SurfaceBlock::ShortGrass => found_short_grass += 1,
                    SurfaceBlock::TallGrass => found_tall_grass += 1,
                    SurfaceBlock::Poppy => found_poppy += 1,
                    SurfaceBlock::Dandelion => found_dandelion += 1,
                    _ => {}
                }

                // Any placed ground cover must rest on valid soil
                if matches!(block, SurfaceBlock::ShortGrass | SurfaceBlock::TallGrass | SurfaceBlock::Poppy | SurfaceBlock::Dandelion) {
                    let soil = buffer.get_block(x, y - 1, z);
                    assert!(GroundCoverPlacer::is_valid_soil(soil), "Ground cover must rest on valid soil (found {:?})", soil);
                }
            }
        }
    }

    assert!(found_short_grass > 0, "Should have placed ShortGrass");
    assert!(found_tall_grass + found_poppy + found_dandelion > 0, "Should have placed varied vegetation types");
    assert_eq!(placed, found_short_grass + found_tall_grass + found_poppy + found_dandelion);
}
