fn main() {
    let generator = mc_worldgen::seed_pipeline::terrain::OverworldGenerator::new(123456789);
    let (raw_c, raw_e, raw_w) = generator.climate_sampler.sample_raw_climate(160, -80);
    let c = raw_c as f32;
    let e = raw_e as f32;
    let w = raw_w as f32;
    let offset = mc_worldgen::seed_pipeline::spline::evaluate_terrain_offset(c, e, w) as f64;
    let factor = mc_worldgen::seed_pipeline::spline::evaluate_terrain_factor(c, e, w, w) as f64;
    let base_3d = generator.blended_noise.sample(160, 64, -80);
    println!("Rust at (160, 64, -80):");
    println!("  raw_c = {}", raw_c);
    println!("  raw_e = {}", raw_e);
    println!("  raw_w = {}", raw_w);
    println!("  offset = {}", offset);
    println!("  factor = {}", factor);
    println!("  base_3d = {}", base_3d);
}
