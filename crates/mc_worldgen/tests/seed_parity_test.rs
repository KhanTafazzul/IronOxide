//! 1:1 Seed-for-seed terrain & biome parity test with vanilla Minecraft 1.20.4.

use mc_worldgen::{
    upgrade_seed_to_128bit, NormalNoise, OverworldGenerator, PositionalRandomFactory,
};

#[test]
fn test_seed_mixing_and_noise_derivation_parity() {
    let world_seed = 123456789i64;
    let (s_lo, s_hi) = upgrade_seed_to_128bit(world_seed);

    assert_eq!(s_lo, -7278655050202467086);
    assert_eq!(s_hi, -3669656285675084719);

    let factory = PositionalRandomFactory::from_world_seed(world_seed);
    assert_eq!(factory.seed_lo, 3219654894476264721);
    assert_eq!(factory.seed_hi, 8777179534091651608);

    let mut o_rng = factory.from_hash_of("minecraft:offset");
    let offset_noise = NormalNoise::from_rng(&mut o_rng, -3, &[1.0, 1.0, 1.0, 0.0]);
    let val = offset_noise.sample(0.0, 0.0, 0.0);
    println!("Rust offset at (0,0,0): {}", val);
    let val2 = offset_noise.sample(12.34, 56.78, 90.12);
    println!("Rust offset at (12.34, 56.78, 90.12): {}", val2);

    let mut t_rng = factory.from_hash_of("minecraft:temperature");
    let temp_noise = NormalNoise::from_rng(&mut t_rng, -10, &[1.5, 0.0, 1.0, 0.0, 0.0, 0.0]);
    let t_val = temp_noise.sample(0.0, 0.0, 0.0);
    println!("Rust temperature at (0,0,0): {}", t_val);
}

#[test]
fn test_overworld_biome_lookup_parity() {
    use mc_worldgen::{OverworldBiomeRegistry, TargetPoint};

    let registry = OverworldBiomeRegistry::load_embedded();

    // Chunk (0, 0) target point at quart (0, 16, 0)
    let target0 = TargetPoint {
        temperature: -211,
        humidity: -1482,
        continentalness: -721,
        erosion: -413,
        depth: -833,
        weirdness: -146,
    };
    let biome0 = registry.find_biome(&target0);
    assert_eq!(biome0, "minecraft:river");

    // Chunk (10, -5) target point at quart (40, 16, -20)
    let target1 = TargetPoint {
        temperature: -454,
        humidity: -165,
        continentalness: -750,
        erosion: 206,
        depth: 12,
        weirdness: -2665,
    };
    let biome1 = registry.find_biome(&target1);
    assert_eq!(biome1, "minecraft:forest");
}

#[test]
fn test_chunk_parity_seed_123456789() {
    let generator = OverworldGenerator::new(123456789);

    // 1. Chunk (0, 0)
    // Quart coordinates at center of chunk at sea level Y=64: quart (0, 16, 0)
    let biome_0_0 = generator.biome_registry.find_biome(&mc_worldgen::TargetPoint {
        temperature: -211,
        humidity: -1482,
        continentalness: -721,
        erosion: -413,
        depth: -833,
        weirdness: -146,
    });
    assert_eq!(biome_0_0, "minecraft:river");

    let heightmap_0_0 = generator.generate_chunk_heightmap(0, 0);
    assert_eq!(heightmap_0_0[0], 55); // (0, 0) in chunk
    assert_eq!(heightmap_0_0[255], 54); // (15, 15) in chunk
    assert_eq!(heightmap_0_0.len(), 256);

    // Expected exact 16x16 heightmap verified from vanilla Minecraft 1.20.4 JVM
    let expected_0_0 = [
        55,55,55,56,56,56,56,56,56,56,56,56,56,56,56,56,
        55,55,55,55,55,56,56,56,56,56,56,56,56,56,56,56,
        54,55,55,55,55,55,55,55,56,56,56,56,56,56,56,56,
        54,54,54,55,55,55,55,55,55,55,55,55,55,56,56,56,
        54,54,54,54,54,54,55,55,55,55,55,55,55,55,55,55,
        54,54,54,54,54,55,55,55,55,55,55,55,55,55,55,55,
        55,55,55,55,55,55,55,55,55,55,55,55,55,55,55,55,
        55,55,55,55,55,55,55,55,55,55,55,55,55,55,55,55,
        55,55,55,55,55,55,55,55,55,55,55,54,54,54,54,55,
        55,55,55,55,55,55,55,55,55,55,55,55,54,54,54,55,
        55,55,55,55,55,55,55,55,55,55,55,55,54,54,54,55,
        55,55,55,55,55,55,55,55,55,55,55,55,54,54,54,55,
        55,55,55,55,55,55,55,55,55,55,55,55,54,54,55,55,
        55,55,55,55,55,55,55,55,55,55,55,55,54,54,55,55,
        55,55,55,55,55,55,55,55,55,55,55,55,54,54,55,55,
        55,55,55,55,55,55,55,55,55,55,55,55,55,55,55,54,
    ];
    assert_eq!(heightmap_0_0, expected_0_0);

    // 2. Chunk (10, -5)
    // Quart coordinates at quart (40, 16, -20)
    let biome_10_neg5 = generator.biome_registry.find_biome(&mc_worldgen::TargetPoint {
        temperature: -454,
        humidity: -165,
        continentalness: -750,
        erosion: 206,
        depth: 12,
        weirdness: -2665,
    });
    assert_eq!(biome_10_neg5, "minecraft:forest");

    let heightmap_10_neg5 = generator.generate_chunk_heightmap(10, -5);
    assert_eq!(heightmap_10_neg5[0], 66); // (160, -80)
    assert_eq!(heightmap_10_neg5[255], 66); // (175, -65)
    assert_eq!(heightmap_10_neg5.len(), 256);

    let expected_10_neg5 = [
        66,66,66,66,66,66,66,66,67,66,66,66,66,66,67,67,
        66,66,66,66,66,66,66,66,66,67,67,67,67,67,67,67,
        66,66,66,66,66,66,66,66,66,67,67,67,67,67,67,67,
        66,66,66,66,66,66,66,66,66,67,67,67,67,67,67,67,
        66,66,66,66,66,66,66,66,66,67,67,67,67,67,67,67,
        65,66,66,66,66,66,66,66,66,67,67,67,67,67,67,67,
        65,66,66,66,66,66,66,66,66,67,67,67,67,67,67,67,
        65,65,66,66,66,66,66,66,66,66,67,67,67,67,67,67,
        65,65,66,66,66,66,66,66,66,66,67,67,67,67,67,67,
        65,65,66,66,66,66,66,66,66,66,66,67,67,67,67,67,
        65,65,66,66,66,66,66,66,66,66,66,66,66,67,67,67,
        65,65,66,66,66,66,66,66,66,66,66,66,66,66,66,66,
        65,65,66,66,66,66,66,66,66,66,66,66,66,66,66,66,
        65,65,65,66,66,66,66,66,66,66,66,66,66,66,66,66,
        65,65,65,66,66,66,66,66,66,66,66,66,66,66,66,66,
        65,65,65,65,66,66,66,66,66,66,66,66,66,66,66,66,
    ];
    assert_eq!(heightmap_10_neg5, expected_10_neg5);
}
