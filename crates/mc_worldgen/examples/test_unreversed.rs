use mc_worldgen::seed_pipeline::noise::ImprovedNoise;
use mc_worldgen::seed_pipeline::random::PositionalRandomFactory;
use mc_worldgen::seed_pipeline::density::BlendedNoise;

fn main() {
    let generator = mc_worldgen::seed_pipeline::terrain::OverworldGenerator::new(123456789);
    println!("Current Rust (0, 48, 0): {}", generator.blended_noise.sample(0, 48, 0));
    println!("Current Rust (160, 64, -80): {}", generator.blended_noise.sample(160, 64, -80));
    
    // Now without reverse:
    let factory = PositionalRandomFactory::from_world_seed(123456789);
    let mut rng = factory.from_hash_of("minecraft:terrain");
    let mut min_limit = Vec::with_capacity(16);
    for _ in 0..16 { min_limit.push(ImprovedNoise::new(&mut rng)); }
    let mut max_limit = Vec::with_capacity(16);
    for _ in 0..16 { max_limit.push(ImprovedNoise::new(&mut rng)); }
    let mut main_noise = Vec::with_capacity(8);
    for _ in 0..8 { main_noise.push(ImprovedNoise::new(&mut rng)); }
    
    let unreversed = BlendedNoise {
        min_limit,
        max_limit,
        main_noise,
        xz_scale: 0.25,
        y_scale: 0.125,
        xz_factor: 80.0,
        y_factor: 160.0,
        smear_scale: 8.0,
    };
    println!("Unreversed Rust (0, 48, 0): {}", unreversed.sample(0, 48, 0));
    println!("Unreversed Rust (160, 64, -80): {}", unreversed.sample(160, 64, -80));
}
