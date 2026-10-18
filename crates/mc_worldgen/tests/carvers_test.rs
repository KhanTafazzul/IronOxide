//! Integration tests for 3D Noise Cave Carvers and Canyons.

use mc_worldgen::carvers::*;
use mc_worldgen::biomes::SurfaceBlock;

#[test]
fn test_cheese_caves_cavern_density() {
    let router = CaveCarverRouter::new(123456789);

    // Sample across a subterranean grid looking for Cheese Caves
    let mut cheese_count = 0;
    for x in 0..30 {
        for z in 0..30 {
            for y in 0..50 {
                let xf = (x * 4) as f64;
                let yf = (y * 2) as f64;
                let zf = (z * 4) as f64;

                if router.cheese.is_cheese_cave(xf, yf, zf) {
                    cheese_count += 1;
                }
            }
        }
    }

    // Must generate natural cavern cavities
    assert!(cheese_count > 0, "Cheese caves should carve open subterranean caverns");
}

#[test]
fn test_spaghetti_caves_3d_and_2d() {
    let router = CaveCarverRouter::new(123456789);

    let mut spaghetti_3d_count = 0;
    let mut spaghetti_2d_count = 0;

    for x in 0..40 {
        for z in 0..40 {
            for y in 0..40 {
                let xf = (x * 3) as f64;
                let yf = (y * 2) as f64;
                let zf = (z * 3) as f64;

                if router.spaghetti.is_spaghetti_3d_cave(xf, yf, zf) {
                    spaghetti_3d_count += 1;
                }
                if router.spaghetti.is_spaghetti_2d_cave(xf, yf, zf) {
                    spaghetti_2d_count += 1;
                }
            }
        }
    }

    assert!(spaghetti_3d_count > 0, "Spaghetti 3D should carve twisting tunnels");
    assert!(spaghetti_2d_count > 0, "Spaghetti 2D should carve horizontal cave branches");
}

#[test]
fn test_noodle_caves_claustrophobic_tunnels() {
    let router = CaveCarverRouter::new(123456789);

    let mut noodle_count = 0;
    for x in 0..50 {
        for z in 0..50 {
            for y in 0..50 {
                let xf = (x * 2) as f64;
                let yf = (y * 2) as f64;
                let zf = (z * 2) as f64;

                if router.noodle.is_noodle_cave(xf, yf, zf) {
                    noodle_count += 1;
                }
            }
        }
    }

    assert!(noodle_count > 0, "Noodle caves should carve narrow passages");
}

#[test]
fn test_bedrock_never_carved() {
    let router = CaveCarverRouter::new(123456789);

    // Test across bedrock boundary (-64..=-60)
    for y in -64..=-60 {
        for x in -20..20 {
            for z in -20..20 {
                let res = carve_block(x, y, z, SurfaceBlock::Bedrock, &router);
                assert!(!res.carved, "Bedrock at Y={y} must NEVER be carved");
                assert_eq!(res.new_block, SurfaceBlock::Bedrock);

                // Even if initial block was stone, bedrock ceiling prevents carving
                let res_stone = carve_block(x, y, z, SurfaceBlock::Stone, &router);
                assert!(!res_stone.carved, "Blocks at bedrock floor Y={y} must never be carved");
                assert_eq!(res_stone.new_block, SurfaceBlock::Stone);
            }
        }
    }
}

#[test]
fn test_carve_block_replacement_rules() {
    let router = CaveCarverRouter::new(123456789);

    // Find a coordinate where carving occurs
    let mut found_carve_loc = None;
    for x in 0..100 {
        for z in 0..100 {
            for y in 20..100 {
                if router.sample_cave(x as f64, y as f64, z as f64).is_some() {
                    found_carve_loc = Some((x, y, z));
                    break;
                }
            }
            if found_carve_loc.is_some() {
                break;
            }
        }
        if found_carve_loc.is_some() {
            break;
        }
    }

    assert!(found_carve_loc.is_some(), "Should find a valid carved coordinate");
    let (cx, cy, cz) = found_carve_loc.unwrap();

    // 1. Carving solid stone replaces it
    let res = carve_block(cx, cy, cz, SurfaceBlock::Stone, &router);
    assert!(res.carved);
    if cy > SEA_LEVEL {
        assert_eq!(res.new_block, SurfaceBlock::Air);
    } else {
        assert_eq!(res.new_block, SurfaceBlock::Water);
    }

    // 2. Carving Deepslate, Dirt, Sand, Terracotta also works
    let res_deepslate = carve_block(cx, cy, cz, SurfaceBlock::Deepslate, &router);
    assert!(res_deepslate.carved);

    let res_dirt = carve_block(cx, cy, cz, SurfaceBlock::Dirt, &router);
    assert!(res_dirt.carved);

    let res_terracotta = carve_block(cx, cy, cz, SurfaceBlock::RedTerracotta, &router);
    assert!(res_terracotta.carved);
}

#[test]
fn test_canyon_carver_generation_and_shape() {
    let carver = CanyonWorldCarver::new(CanyonCarverConfiguration::default());

    // Generate canyon nodes with fixed seed
    let nodes = carver.generate_canyon_path(0, 0, 123456789);
    assert_eq!(nodes.len(), 112, "Canyon should have 112 steps");

    // The midpoint should be thicker than the tapered endpoints
    let mid_node = &nodes[56];
    let start_node = &nodes[0];
    assert!(mid_node.width > start_node.width, "Midpoint of canyon should be wider than start");
    assert!((mid_node.height / mid_node.width - 3.0).abs() < 1e-4, "yScale should be 3.0");

    // Test volume carving around the midpoint
    let hit = carver.is_carved_by_canyon(
        mid_node.x.round() as i32,
        mid_node.y.round() as i32,
        mid_node.z.round() as i32,
        &nodes,
    );
    assert!(hit, "Center of canyon node should be carved");

    let far_miss = carver.is_carved_by_canyon(
        mid_node.x.round() as i32 + 500,
        mid_node.y.round() as i32,
        mid_node.z.round() as i32,
        &nodes,
    );
    assert!(!far_miss, "Far distance should not be carved by canyon");
}
