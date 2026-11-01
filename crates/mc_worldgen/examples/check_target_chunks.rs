use mc_worldgen::OverworldGenerator;

fn main() {
    let generator = OverworldGenerator::new(123456789);

    println!("--- Chunk (15, -7) ---");
    let h15 = generator.generate_chunk_heightmap(15, -7);
    println!("Chunk (15, -7) sample heights: min={}, max={}", h15.iter().min().unwrap(), h15.iter().max().unwrap());
    let mut mismatches15 = 0;
    for i in 0..256 {
        let expected = if i == 240 { 64 } else { 63 };
        if h15[i] != expected {
            mismatches15 += 1;
            println!("  [15, -7] idx {} (x={}, z={}): got {}, expected {}", i, i % 16, i / 16, h15[i], expected);
        }
    }
    println!("Chunk (15, -7) total mismatches: {} / 256", mismatches15);

    println!("--- Chunk (-100, 100) ---");
    let h100 = generator.generate_chunk_heightmap(-100, 100);
    println!("Chunk (-100, 100) sample heights: min={}, max={}", h100.iter().min().unwrap(), h100.iter().max().unwrap());
    let mut mismatches100 = 0;
    for i in 0..256 {
        let expected = 47;
        if h100[i] != expected {
            mismatches100 += 1;
            println!("  [-100, 100] idx {} (x={}, z={}): got {}, expected {}", i, i % 16, i / 16, h100[i], expected);
        }
    }
    println!("Chunk (-100, 100) total mismatches: {} / 256", mismatches100);
}
