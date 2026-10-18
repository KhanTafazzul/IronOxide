//! Climate target search in 6D space matching vanilla Overworld biome IDs.

use super::noise::NormalNoise;
use super::random::{double, PositionalRandomFactory};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParameterRange {
    pub min: i64,
    pub max: i64,
}

impl ParameterRange {
    #[inline]
    pub const fn new(min: i64, max: i64) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn distance(&self, val: i64) -> i64 {
        let d_max = val.wrapping_sub(self.max);
        let d_min = self.min.wrapping_sub(val);
        if d_max > 0 {
            d_max
        } else if d_min > 0 {
            d_min
        } else {
            0
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParameterPoint {
    pub temperature: ParameterRange,
    pub humidity: ParameterRange,
    pub continentalness: ParameterRange,
    pub erosion: ParameterRange,
    pub depth: ParameterRange,
    pub weirdness: ParameterRange,
    pub offset: i64,
}

impl ParameterPoint {
    #[inline]
    pub fn fitness(&self, target: &TargetPoint) -> i64 {
        let dt = self.temperature.distance(target.temperature);
        let dh = self.humidity.distance(target.humidity);
        let dc = self.continentalness.distance(target.continentalness);
        let de = self.erosion.distance(target.erosion);
        let dd = self.depth.distance(target.depth);
        let dw = self.weirdness.distance(target.weirdness);
        let doff = self.offset;

        dt.wrapping_mul(dt)
            .wrapping_add(dh.wrapping_mul(dh))
            .wrapping_add(dc.wrapping_mul(dc))
            .wrapping_add(de.wrapping_mul(de))
            .wrapping_add(dd.wrapping_mul(dd))
            .wrapping_add(dw.wrapping_mul(dw))
            .wrapping_add(doff.wrapping_mul(doff))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetPoint {
    pub temperature: i64,
    pub humidity: i64,
    pub continentalness: i64,
    pub erosion: i64,
    pub depth: i64,
    pub weirdness: i64,
}

impl TargetPoint {
    #[inline]
    pub fn from_floats(t: double, h: double, c: double, e: double, d: double, w: double) -> Self {
        Self {
            temperature: (t * 10000.0) as i64,
            humidity: (h * 10000.0) as i64,
            continentalness: (c * 10000.0) as i64,
            erosion: (e * 10000.0) as i64,
            depth: (d * 10000.0) as i64,
            weirdness: (w * 10000.0) as i64,
        }
    }
}

/// 6-dimensional climate sampler generating TargetPoints from world seed.
#[derive(Clone, Debug)]
pub struct ClimateSampler {
    pub temperature_noise: NormalNoise,
    pub vegetation_noise: NormalNoise,
    pub continentalness_noise: NormalNoise,
    pub erosion_noise: NormalNoise,
    pub ridge_noise: NormalNoise,
    pub offset_noise: NormalNoise,
}

impl ClimateSampler {
    pub fn new(world_seed: i64) -> Self {
        let factory = PositionalRandomFactory::from_world_seed(world_seed);

        let mut t_rng = factory.from_hash_of("minecraft:temperature");
        let temperature_noise = NormalNoise::from_rng(
            &mut t_rng,
            -10,
            &[1.5, 0.0, 1.0, 0.0, 0.0, 0.0],
        );

        let mut v_rng = factory.from_hash_of("minecraft:vegetation");
        let vegetation_noise = NormalNoise::from_rng(
            &mut v_rng,
            -8,
            &[1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
        );

        let mut c_rng = factory.from_hash_of("minecraft:continentalness");
        let continentalness_noise = NormalNoise::from_rng(
            &mut c_rng,
            -9,
            &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0],
        );

        let mut e_rng = factory.from_hash_of("minecraft:erosion");
        let erosion_noise = NormalNoise::from_rng(
            &mut e_rng,
            -9,
            &[1.0, 1.0, 0.0, 1.0, 1.0],
        );

        let mut r_rng = factory.from_hash_of("minecraft:ridge");
        let ridge_noise = NormalNoise::from_rng(
            &mut r_rng,
            -7,
            &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0],
        );

        let mut o_rng = factory.from_hash_of("minecraft:offset");
        let offset_noise = NormalNoise::from_rng(
            &mut o_rng,
            -3,
            &[1.0, 1.0, 1.0, 0.0],
        );

        Self {
            temperature_noise,
            vegetation_noise,
            continentalness_noise,
            erosion_noise,
            ridge_noise,
            offset_noise,
        }
    }

    /// Evaluates the 6D target point at given block coordinates.
    pub fn sample(&self, block_x: i32, block_y: i32, block_z: i32) -> TargetPoint {
        let x = block_x as f64;
        let y = block_y as f64;
        let z = block_z as f64;

        // Shift A and Shift B domain distortion
        let shift_x = self.offset_noise.sample(x * 0.25, 0.0, z * 0.25) * 4.0;
        let shift_z = self.offset_noise.sample(z * 0.25, x * 0.25, 0.0) * 4.0;

        let sample_x = (x * 0.25) + shift_x;
        let sample_z = (z * 0.25) + shift_z;

        let t = self.temperature_noise.sample(sample_x, 0.0, sample_z);
        let h = self.vegetation_noise.sample(sample_x, 0.0, sample_z);
        let c = self.continentalness_noise.sample(sample_x, 0.0, sample_z);
        let e = self.erosion_noise.sample(sample_x, 0.0, sample_z);
        let w = self.ridge_noise.sample(sample_x, 0.0, sample_z);

        // Depth: clamped gradient [-64, 320] -> [1.5, -1.5]
        let clamped_y = y.clamp(-64.0, 320.0);
        let progress = (clamped_y - (-64.0)) / (320.0 - (-64.0));
        let y_grad = 1.5 + progress * (-1.5 - 1.5);
        let d = y_grad;

        TargetPoint::from_floats(t, h, c, e, d, w)
    }
}

/// Static embedded Overworld biome lookup table containing all 7,593 points.
pub struct OverworldBiomeRegistry {
    pub biomes: Vec<String>,
    pub points: Vec<(ParameterPoint, usize)>,
}

impl OverworldBiomeRegistry {
    pub fn load_embedded() -> Self {
        const BYTES: &[u8] = include_bytes!("../../data/overworld_biomes.dat");
        let mut offset = 0;

        let n_biomes = i32::from_be_bytes(BYTES[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let mut biomes = Vec::with_capacity(n_biomes);
        for _ in 0..n_biomes {
            let str_len = i16::from_be_bytes(BYTES[offset..offset + 2].try_into().unwrap()) as usize;
            offset += 2;
            let s = std::str::from_utf8(&BYTES[offset..offset + str_len]).unwrap();
            biomes.push(s.to_string());
            offset += str_len;
        }

        let n_points = i32::from_be_bytes(BYTES[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let mut points = Vec::with_capacity(n_points);
        for _ in 0..n_points {
            let t_min = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;
            let t_max = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let h_min = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;
            let h_max = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let c_min = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;
            let c_max = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let e_min = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;
            let e_max = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let d_min = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;
            let d_max = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let w_min = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;
            let w_max = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let off = i64::from_be_bytes(BYTES[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let biome_idx = i32::from_be_bytes(BYTES[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;

            let point = ParameterPoint {
                temperature: ParameterRange::new(t_min, t_max),
                humidity: ParameterRange::new(h_min, h_max),
                continentalness: ParameterRange::new(c_min, c_max),
                erosion: ParameterRange::new(e_min, e_max),
                depth: ParameterRange::new(d_min, d_max),
                weirdness: ParameterRange::new(w_min, w_max),
                offset: off,
            };
            points.push((point, biome_idx));
        }

        Self { biomes, points }
    }

    /// Finds the exact matching vanilla biome ID for the given 6D target point.
    pub fn find_biome(&self, target: &TargetPoint) -> &str {
        let mut min_dist = i64::MAX;
        let mut best_idx = 0;

        for (point, biome_idx) in &self.points {
            let dist = point.fitness(target);
            if dist < min_dist {
                min_dist = dist;
                best_idx = *biome_idx;
                if dist == 0 {
                    break;
                }
            }
        }

        &self.biomes[best_idx]
    }
}
