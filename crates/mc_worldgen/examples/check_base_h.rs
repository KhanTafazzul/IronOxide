use mc_worldgen::OverworldGenerator;

fn main() {
    let generator = OverworldGenerator::new(123456789);
    let target = generator.climate_sampler.sample(0, 50, 0);
    let c = (target.continentalness as f32) / 10000.0;
    let e = (target.erosion as f32) / 10000.0;
    let w = (target.weirdness as f32) / 10000.0;
    println!("c={}, e={}, w={}", c, e, w);
    let offset = mc_worldgen::seed_pipeline::spline::evaluate_terrain_offset(c, e, w);
    let base_h = 128.0 * (1.0 + offset);
    println!("offset={}, base_h={}", offset, base_h);
}
