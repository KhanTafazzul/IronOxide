use mc_worldgen::OverworldGenerator;

fn main() {
    let generator = OverworldGenerator::new(123456789);
    for y in [48, 56] {
        let base_3d = generator.blended_noise.sample(0, y, 0);
        let target = generator.climate_sampler.sample(0, y, 0);
        let c = (target.continentalness as f32) / 10000.0;
        let e = (target.erosion as f32) / 10000.0;
        let w = (target.weirdness as f32) / 10000.0;
        let offset = mc_worldgen::seed_pipeline::spline::evaluate_terrain_offset(c, e, w) as f64;
        let base_height = 128.0 * (1.0 + offset);

        println!("Y={}: base_h={}, base_3d={}", y, base_height, base_3d);
    }
}
