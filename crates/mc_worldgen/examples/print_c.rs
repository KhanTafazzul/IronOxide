use mc_worldgen::OverworldGenerator;

fn main() {
    let factory = mc_worldgen::seed_pipeline::random::PositionalRandomFactory::from_world_seed(123456789);
    println!("world factory: lo={}, hi={}", factory.seed_lo, factory.seed_hi);
    let mut c_rng = factory.from_hash_of("minecraft:continentalness");
    println!("c_rng initial: s0={}, s1={}", c_rng.s0, c_rng.s1);
    let f1 = c_rng.fork_positional();
    println!("f1: lo={}, hi={}", f1.seed_lo, f1.seed_hi);
    println!("c_rng after f1: s0={}, s1={}", c_rng.s0, c_rng.s1);
    let f2 = c_rng.fork_positional();
    println!("f2: lo={}, hi={}", f2.seed_lo, f2.seed_hi);
    let generator = OverworldGenerator::new(123456789);
    let (raw_c, raw_e, raw_w) = generator.climate_sampler.sample_raw_climate(0, 0);
    let sx = generator.climate_sampler.offset_noise.sample(0.0, 0.0, 0.0) * 4.0;
    println!("sx={}, sz={}", sx, sx);
    let cs = &generator.climate_sampler;
    let oct0 = cs.continentalness_noise.first.octaves[0].as_ref().unwrap();
    println!("IronOxide oct0: xo={}, yo={}, zo={}", oct0.xo, oct0.yo, oct0.zo);
    let amps = [1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0];
    let sample_with_amps = |perlin: &mc_worldgen::seed_pipeline::noise::PerlinNoise, x: f64, y: f64, z: f64| -> f64 {
        let mut total = 0.0;
        let mut cur_scale = perlin.scale_factor;
        let mut cur_amp = perlin.amplitude_factor;
        for (i, oct) in perlin.octaves.iter().enumerate() {
            if let Some(noise) = oct {
                let nx = mc_worldgen::seed_pipeline::noise::wrap_coord(x * cur_scale);
                let ny = mc_worldgen::seed_pipeline::noise::wrap_coord(y * cur_scale);
                let nz = mc_worldgen::seed_pipeline::noise::wrap_coord(z * cur_scale);
                total += noise.sample(nx, ny, nz) * amps[i] * cur_amp;
            }
            cur_scale *= 2.0;
            cur_amp /= 2.0;
        }
        total
    };
    let test_v1 = sample_with_amps(&cs.continentalness_noise.first, sx, 0.0, sx);
    let test_v2 = sample_with_amps(&cs.continentalness_noise.second, sx * 1.0181268882175227, 0.0, sx * 1.0181268882175227);
    println!("test_v1={}, test_v2={}", test_v1, test_v2);
    let test_total = (test_v1 + test_v2) * cs.continentalness_noise.value_factor;
    println!("test_total={}", test_total);
    let (raw_c, raw_e, raw_w) = generator.climate_sampler.sample_raw_climate(0, 0);
    println!("raw_c={}, raw_e={}, raw_w={}", raw_c, raw_e, raw_w);
    let c = raw_c as f32;
    let e = raw_e as f32;
    let w = raw_w as f32;
    let offset = mc_worldgen::seed_pipeline::spline::evaluate_terrain_offset(c, e, w) as f64;
    let factor = mc_worldgen::seed_pipeline::spline::evaluate_terrain_factor(c, e, w, w) as f64;
    println!("offset={}, factor={}", offset, factor);

    for y in [48, 56] {
        let base_3d = generator.blended_noise.sample(0, y, 0);
        let base_height = 128.0 * (1.0 + offset);
        let depth = (base_height - (y as f64)) / 128.0;
        let scaled_depth = if depth < 0.0 { depth * factor * 0.25 } else { depth * factor };
        let depth_density = 4.0 * scaled_depth;
        println!("y={}: base_3d={}, depth={}, depth_density={}, total={}", y, base_3d, depth, depth_density, depth_density + base_3d);
    }
}
