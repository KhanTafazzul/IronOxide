use mc_worldgen::seed_pipeline::noise::ImprovedNoise;
use mc_worldgen::PositionalRandomFactory;

fn main() {
    let factory = PositionalRandomFactory::from_world_seed(123456789);
    let mut rng = factory.from_hash_of("minecraft:terrain");
    println!("Rust terrain RNG: s0={}, s1={}", rng.s0, rng.s1);

    for i in 0..16 {
        let n = ImprovedNoise::new(&mut rng);
        println!("Rust generated #{}: xo={}, yo={}, zo={}", i, n.xo, n.yo, n.zo);
    }
}
