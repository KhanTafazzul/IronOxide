//! Vertical density sliders, cave noise routers (cheese, spaghetti 2D, spaghetti 3D, noodle, aquifers)
//! and 3D blended noise matching Minecraft 1.20.4 density pipeline.

use super::noise::{NormalNoise, PerlinNoise};
use super::random::{double, PositionalRandomFactory};

/// Minecraft classic 3D BlendedNoise (net.minecraft.world.level.levelgen.synth.BlendedNoise).
#[derive(Clone, Debug)]
pub struct BlendedNoise {
    pub min_limit: PerlinNoise,
    pub max_limit: PerlinNoise,
    pub main_noise: PerlinNoise,
    pub xz_scale: double,
    pub y_scale: double,
    pub xz_factor: double,
    pub y_factor: double,
    pub smear_scale: double,
}

impl BlendedNoise {
    pub fn new(world_seed: i64) -> Self {
        let factory = PositionalRandomFactory::from_world_seed(world_seed);
        let mut rng = factory.from_hash_of("minecraft:terrain");

        let f_min = rng.fork_positional();
        let min_limit = PerlinNoise::from_factory(&f_min, -15, &[1.0; 16]);

        let f_max = rng.fork_positional();
        let max_limit = PerlinNoise::from_factory(&f_max, -15, &[1.0; 16]);

        let f_main = rng.fork_positional();
        let main_noise = PerlinNoise::from_factory(&f_main, -7, &[1.0; 8]);

        Self {
            min_limit,
            max_limit,
            main_noise,
            xz_scale: 0.25,
            y_scale: 0.125,
            xz_factor: 80.0,
            y_factor: 160.0,
            smear_scale: 8.0,
        }
    }

    pub fn sample(&self, block_x: i32, block_y: i32, block_z: i32) -> double {
        let x = (block_x as f64) * 684.412 * self.xz_scale;
        let y = (block_y as f64) * 684.412 * self.y_scale;
        let z = (block_z as f64) * 684.412 * self.xz_scale;

        let x_scaled = x / self.xz_factor;
        let y_scaled = y / self.y_factor;
        let z_scaled = z / self.xz_factor;

        let main_val = self.main_noise.sample(x_scaled, y_scaled, z_scaled);
        let alpha = ((main_val / 10.0) + 1.0) * 0.5;

        if alpha <= 0.0 {
            self.min_limit.sample(x, y, z) / 128.0
        } else if alpha >= 1.0 {
            self.max_limit.sample(x, y, z) / 128.0
        } else {
            let min_val = self.min_limit.sample(x, y, z) / 128.0;
            let max_val = self.max_limit.sample(x, y, z) / 128.0;
            min_val + alpha * (max_val - min_val)
        }
    }
}

/// Cave noise routers (spaghetti 2D, spaghetti 3D, noodle, pillars, aquifers).
#[derive(Clone, Debug)]
pub struct CaveNoiseRouter {
    pub spaghetti_2d: NormalNoise,
    pub spaghetti_roughness_modulator: NormalNoise,
    pub spaghetti_3d_1: NormalNoise,
    pub spaghetti_3d_2: NormalNoise,
    pub noodle: NormalNoise,
    pub noodle_thickness: NormalNoise,
    pub noodle_ridge_a: NormalNoise,
    pub noodle_ridge_b: NormalNoise,
    pub pillars: NormalNoise,
    pub aquifer_barrier: NormalNoise,
    pub aquifer_fluid_level_floodedness: NormalNoise,
    pub aquifer_fluid_level_spread: NormalNoise,
    pub aquifer_lava: NormalNoise,
}

impl CaveNoiseRouter {
    pub fn new(world_seed: i64) -> Self {
        let factory = PositionalRandomFactory::from_world_seed(world_seed);

        let mut s2d_rng = factory.from_hash_of("minecraft:spaghetti_2d");
        let spaghetti_2d = NormalNoise::from_rng(&mut s2d_rng, -7, &[1.0, 1.0]);

        let mut srm_rng = factory.from_hash_of("minecraft:spaghetti_roughness_modulator");
        let spaghetti_roughness_modulator = NormalNoise::from_rng(&mut srm_rng, -8, &[1.0]);

        let mut s3d1_rng = factory.from_hash_of("minecraft:spaghetti_3d_1");
        let spaghetti_3d_1 = NormalNoise::from_rng(&mut s3d1_rng, -7, &[1.0, 1.0]);

        let mut s3d2_rng = factory.from_hash_of("minecraft:spaghetti_3d_2");
        let spaghetti_3d_2 = NormalNoise::from_rng(&mut s3d2_rng, -7, &[1.0, 1.0]);

        let mut ndl_rng = factory.from_hash_of("minecraft:noodle");
        let noodle = NormalNoise::from_rng(&mut ndl_rng, -8, &[1.0]);

        let mut nth_rng = factory.from_hash_of("minecraft:noodle_thickness");
        let noodle_thickness = NormalNoise::from_rng(&mut nth_rng, -8, &[1.0]);

        let mut nra_rng = factory.from_hash_of("minecraft:noodle_ridge_a");
        let noodle_ridge_a = NormalNoise::from_rng(&mut nra_rng, -7, &[1.0]);

        let mut nrb_rng = factory.from_hash_of("minecraft:noodle_ridge_b");
        let noodle_ridge_b = NormalNoise::from_rng(&mut nrb_rng, -7, &[1.0]);

        let mut pil_rng = factory.from_hash_of("minecraft:pillar");
        let pillars = NormalNoise::from_rng(&mut pil_rng, -7, &[1.0, 1.0]);

        let mut ab_rng = factory.from_hash_of("minecraft:aquifer_barrier");
        let aquifer_barrier = NormalNoise::from_rng(&mut ab_rng, -3, &[1.0]);

        let mut aff_rng = factory.from_hash_of("minecraft:aquifer_fluid_level_floodedness");
        let aquifer_fluid_level_floodedness = NormalNoise::from_rng(&mut aff_rng, -7, &[1.0]);

        let mut afs_rng = factory.from_hash_of("minecraft:aquifer_fluid_level_spread");
        let aquifer_fluid_level_spread = NormalNoise::from_rng(&mut afs_rng, -5, &[1.0]);

        let mut al_rng = factory.from_hash_of("minecraft:aquifer_lava");
        let aquifer_lava = NormalNoise::from_rng(&mut al_rng, -1, &[1.0]);

        Self {
            spaghetti_2d,
            spaghetti_roughness_modulator,
            spaghetti_3d_1,
            spaghetti_3d_2,
            noodle,
            noodle_thickness,
            noodle_ridge_a,
            noodle_ridge_b,
            pillars,
            aquifer_barrier,
            aquifer_fluid_level_floodedness,
            aquifer_fluid_level_spread,
            aquifer_lava,
        }
    }

    /// Evaluates cave carving density modifier. Returns value <= 0 to carve air pocket.
    pub fn sample_caves(&self, x: double, y: double, z: double) -> double {
        // Spaghetti 2D cave entrance check
        let s2d = self.spaghetti_2d.sample(x * 2.0, 0.0, z * 2.0);
        let s2d_carve = s2d.abs() - 0.07;

        // Spaghetti 3D subterranean network
        let s3d1 = self.spaghetti_3d_1.sample(x, y, z);
        let s3d2 = self.spaghetti_3d_2.sample(x, y, z);
        let s3d_carve = (s3d1 * s3d1 + s3d2 * s3d2) - 0.08;

        // Noodle cave tunnels
        let ndl = self.noodle.sample(x, y, z);
        let ndl_carve = ndl.abs() - 0.04;

        s2d_carve.min(s3d_carve).min(ndl_carve)
    }
}

/// Evaluates vertical density slides at given Y block coordinate.
#[inline]
pub fn vertical_density_slider(y: i32) -> double {
    let mut factor = 1.0;

    // Top slide: clamped gradient from 240 (1.0) down to 256 (0.0)
    if y >= 240 {
        if y >= 256 {
            return -1.0;
        }
        let t = ((y - 240) as f64) / 16.0;
        factor *= 1.0 - t;
    }

    // Bottom slide: clamped gradient from -64 (0.0) up to -40 (1.0)
    if y <= -40 {
        if y <= -64 {
            return -1.0;
        }
        let t = ((y - (-64)) as f64) / 24.0;
        factor *= t;
    }

    factor
}
