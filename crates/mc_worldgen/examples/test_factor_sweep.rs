use mc_worldgen::OverworldGenerator;

fn main() {
    let generator = OverworldGenerator::new(123456789);
    for factor in [3.0, 3.1, 3.2, 3.25, 3.3, 3.4, 3.5] {
        let calc_d = |y: i32| -> f64 {
            let base_3d = generator.blended_noise.sample(0, y, 0);
            let target = generator.climate_sampler.sample(0, 50, 0);
            let c = (target.continentalness as f32) / 10000.0;
            let e = (target.erosion as f32) / 10000.0;
            let w = (target.weirdness as f32) / 10000.0;
            let offset = mc_worldgen::seed_pipeline::spline::evaluate_terrain_offset(c, e, w) as f64;
            let base_height = 128.0 * (1.0 + offset);

            let depth = (base_height - (y as f64)) / 128.0;
            let scaled_depth = if depth < 0.0 { depth * factor * 0.25 } else { depth * factor };
            4.0 * scaled_depth + base_3d
        };

        let c48 = calc_d(48);
        let c56 = calc_d(56);
        let d54 = c48 + 0.75 * (c56 - c48);
        let d55 = c48 + 0.875 * (c56 - c48);
        println!("Factor {}: d54={:.4} (solid={}), d55={:.4} (solid={})", factor, d54, d54 > 0.0, d55, d55 > 0.0);
    }
}
