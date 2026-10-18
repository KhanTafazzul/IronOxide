//! Test single octave sample

use mc_worldgen::{ImprovedNoise, Xoroshiro128PlusPlus};

#[test]
fn test_single_octave() {
    let mut rng = Xoroshiro128PlusPlus::new(-7662126341512960630, -7857115354982292015);
    let oct0 = ImprovedNoise::new(&mut rng);
    println!("Rust xo={}, yo={}, zo={}", oct0.xo, oct0.yo, oct0.zo);
    print!("Rust p[0..15]: ");
    for i in 0..16 {
        print!("{}, ", oct0.p[i]);
    }
    println!();
    let val = oct0.sample(0.0, 0.0, 0.0);
    println!("Rust sample(0,0,0) = {}", val);
}
