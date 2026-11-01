use mc_worldgen::seed_pipeline::noise::ImprovedNoise;
use mc_worldgen::seed_pipeline::random::PositionalRandomFactory;

fn main() {
    let factory = PositionalRandomFactory::from_world_seed(123456789);
    let mut rng = factory.from_hash_of("minecraft:terrain");
    for i in 0..16 {
        let noise = ImprovedNoise::new(&mut rng);
        println!("Rust oct {}: xo={}, yo={}, zo={}", i, noise.xo, noise.yo, noise.zo);
    }
}
