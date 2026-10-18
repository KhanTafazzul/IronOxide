//! Vertical density sliders, cave noise routers (cheese, spaghetti 2D, spaghetti 3D, noodle, aquifers)
//! and 3D blended noise matching Minecraft 1.20+ vanilla density pipeline.

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

/// Full 15-channel vanilla Minecraft 1.20+ NoiseRouter
/// (net.minecraft.world.level.levelgen.NoiseRouter).
#[derive(Clone, Debug)]
pub struct NoiseRouter {
    // Aquifer samplers
    pub aquifer_barrier: NormalNoise,
    pub aquifer_fluid_level_floodedness: NormalNoise,
    pub aquifer_fluid_level_spread: NormalNoise,
    pub aquifer_lava: NormalNoise,

    // Climate & continental shaping
    pub offset: NormalNoise,
    pub temperature_noise: NormalNoise,
    pub vegetation_noise: NormalNoise,
    pub continentalness_noise: NormalNoise,
    pub erosion_noise: NormalNoise,
    pub ridge_noise: NormalNoise,

    // Cave samplers
    pub cave_cheese: NormalNoise,
    pub cave_layer: NormalNoise,
    pub cave_entrance: NormalNoise,
    pub spaghetti_2d: NormalNoise,
    pub spaghetti_2d_elevation: NormalNoise,
    pub spaghetti_2d_modulator: NormalNoise,
    pub spaghetti_2d_thickness: NormalNoise,
    pub spaghetti_3d_1: NormalNoise,
    pub spaghetti_3d_2: NormalNoise,
    pub spaghetti_3d_rarity: NormalNoise,
    pub spaghetti_3d_thickness: NormalNoise,
    pub spaghetti_roughness: NormalNoise,
    pub spaghetti_roughness_modulator: NormalNoise,
    pub noodle: NormalNoise,
    pub noodle_thickness: NormalNoise,
    pub noodle_ridge_a: NormalNoise,
    pub noodle_ridge_b: NormalNoise,
    pub pillars: NormalNoise,
    pub pillar_rareness: NormalNoise,
    pub pillar_thickness: NormalNoise,

    // Ore vein samplers
    pub ore_veininess: NormalNoise,
    pub ore_vein_a: NormalNoise,
    pub ore_vein_b: NormalNoise,
    pub ore_gap: NormalNoise,
}

impl NoiseRouter {
    pub fn new(world_seed: i64) -> Self {
        let factory = PositionalRandomFactory::from_world_seed(world_seed);

        // 1. Aquifer Noises
        let mut ab_rng = factory.from_hash_of("minecraft:aquifer_barrier");
        let aquifer_barrier = NormalNoise::from_rng(&mut ab_rng, -3, &[1.0]);

        let mut aff_rng = factory.from_hash_of("minecraft:aquifer_fluid_level_floodedness");
        let aquifer_fluid_level_floodedness = NormalNoise::from_rng(&mut aff_rng, -7, &[1.0]);

        let mut afs_rng = factory.from_hash_of("minecraft:aquifer_fluid_level_spread");
        let aquifer_fluid_level_spread = NormalNoise::from_rng(&mut afs_rng, -5, &[1.0]);

        let mut al_rng = factory.from_hash_of("minecraft:aquifer_lava");
        let aquifer_lava = NormalNoise::from_rng(&mut al_rng, -1, &[1.0]);

        // 2. Continental / Climate Noises
        let mut off_rng = factory.from_hash_of("minecraft:offset");
        let offset = NormalNoise::from_rng(&mut off_rng, -3, &[1.0, 1.0, 1.0, 0.0]);

        let mut temp_rng = factory.from_hash_of("minecraft:temperature");
        let temperature_noise = NormalNoise::from_rng(&mut temp_rng, -10, &[1.5, 0.0, 1.0, 0.0, 0.0, 0.0]);

        let mut veg_rng = factory.from_hash_of("minecraft:vegetation");
        let vegetation_noise = NormalNoise::from_rng(&mut veg_rng, -8, &[1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);

        let mut cont_rng = factory.from_hash_of("minecraft:continentalness");
        let continentalness_noise = NormalNoise::from_rng(&mut cont_rng, -9, &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0]);

        let mut er_rng = factory.from_hash_of("minecraft:erosion");
        let erosion_noise = NormalNoise::from_rng(&mut er_rng, -9, &[1.0, 1.0, 0.0, 1.0, 1.0]);

        let mut rid_rng = factory.from_hash_of("minecraft:ridge");
        let ridge_noise = NormalNoise::from_rng(&mut rid_rng, -7, &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0]);

        // 3. Cave Noises
        let mut cc_rng = factory.from_hash_of("minecraft:cave_cheese");
        let cave_cheese = NormalNoise::from_rng(&mut cc_rng, -8, &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0]);

        let mut cl_rng = factory.from_hash_of("minecraft:cave_layer");
        let cave_layer = NormalNoise::from_rng(&mut cl_rng, -8, &[1.0]);

        let mut ce_rng = factory.from_hash_of("minecraft:cave_entrance");
        let cave_entrance = NormalNoise::from_rng(&mut ce_rng, -7, &[0.4, 0.5, 1.0]);

        let mut s2d_rng = factory.from_hash_of("minecraft:spaghetti_2d");
        let spaghetti_2d = NormalNoise::from_rng(&mut s2d_rng, -7, &[1.0]);

        let mut s2de_rng = factory.from_hash_of("minecraft:spaghetti_2d_elevation");
        let spaghetti_2d_elevation = NormalNoise::from_rng(&mut s2de_rng, -8, &[1.0]);

        let mut s2dm_rng = factory.from_hash_of("minecraft:spaghetti_2d_modulator");
        let spaghetti_2d_modulator = NormalNoise::from_rng(&mut s2dm_rng, -11, &[1.0]);

        let mut s2dt_rng = factory.from_hash_of("minecraft:spaghetti_2d_thickness");
        let spaghetti_2d_thickness = NormalNoise::from_rng(&mut s2dt_rng, -11, &[1.0]);

        let mut s3d1_rng = factory.from_hash_of("minecraft:spaghetti_3d_1");
        let spaghetti_3d_1 = NormalNoise::from_rng(&mut s3d1_rng, -7, &[1.0]);

        let mut s3d2_rng = factory.from_hash_of("minecraft:spaghetti_3d_2");
        let spaghetti_3d_2 = NormalNoise::from_rng(&mut s3d2_rng, -7, &[1.0]);

        let mut s3dr_rng = factory.from_hash_of("minecraft:spaghetti_3d_rarity");
        let spaghetti_3d_rarity = NormalNoise::from_rng(&mut s3dr_rng, -11, &[1.0]);

        let mut s3dt_rng = factory.from_hash_of("minecraft:spaghetti_3d_thickness");
        let spaghetti_3d_thickness = NormalNoise::from_rng(&mut s3dt_rng, -8, &[1.0]);

        let mut sr_rng = factory.from_hash_of("minecraft:spaghetti_roughness");
        let spaghetti_roughness = NormalNoise::from_rng(&mut sr_rng, -5, &[1.0]);

        let mut srm_rng = factory.from_hash_of("minecraft:spaghetti_roughness_modulator");
        let spaghetti_roughness_modulator = NormalNoise::from_rng(&mut srm_rng, -8, &[1.0]);

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

        let mut pr_rng = factory.from_hash_of("minecraft:pillar_rareness");
        let pillar_rareness = NormalNoise::from_rng(&mut pr_rng, -8, &[1.0]);

        let mut pt_rng = factory.from_hash_of("minecraft:pillar_thickness");
        let pillar_thickness = NormalNoise::from_rng(&mut pt_rng, -8, &[1.0]);

        // 4. Ore Vein Noises
        let mut ov_rng = factory.from_hash_of("minecraft:ore_veininess");
        let ore_veininess = NormalNoise::from_rng(&mut ov_rng, -8, &[1.0]);

        let mut ova_rng = factory.from_hash_of("minecraft:ore_vein_a");
        let ore_vein_a = NormalNoise::from_rng(&mut ova_rng, -7, &[1.0]);

        let mut ovb_rng = factory.from_hash_of("minecraft:ore_vein_b");
        let ore_vein_b = NormalNoise::from_rng(&mut ovb_rng, -7, &[1.0]);

        let mut og_rng = factory.from_hash_of("minecraft:ore_gap");
        let ore_gap = NormalNoise::from_rng(&mut og_rng, -5, &[1.0]);

        Self {
            aquifer_barrier,
            aquifer_fluid_level_floodedness,
            aquifer_fluid_level_spread,
            aquifer_lava,
            offset,
            temperature_noise,
            vegetation_noise,
            continentalness_noise,
            erosion_noise,
            ridge_noise,
            cave_cheese,
            cave_layer,
            cave_entrance,
            spaghetti_2d,
            spaghetti_2d_elevation,
            spaghetti_2d_modulator,
            spaghetti_2d_thickness,
            spaghetti_3d_1,
            spaghetti_3d_2,
            spaghetti_3d_rarity,
            spaghetti_3d_thickness,
            spaghetti_roughness,
            spaghetti_roughness_modulator,
            noodle,
            noodle_thickness,
            noodle_ridge_a,
            noodle_ridge_b,
            pillars,
            pillar_rareness,
            pillar_thickness,
            ore_veininess,
            ore_vein_a,
            ore_vein_b,
            ore_gap,
        }
    }

    // --- 15 Vanilla Noise Router Functions ---

    /// 1. `barrier`: Noise boundary controlling dry caves vs. flooded aquifers.
    #[inline]
    pub fn barrier(&self, x: f64, y: f64, z: f64) -> f64 {
        self.aquifer_barrier.sample(x, y * 0.5, z)
    }

    /// 2. `fluid_level_floodedness`: Dynamic fluid threshold for subsurface aquifers.
    #[inline]
    pub fn fluid_level_floodedness(&self, x: f64, y: f64, z: f64) -> f64 {
        self.aquifer_fluid_level_floodedness.sample(x, y * 0.67, z)
    }

    /// 3. `fluid_level_spread`: Dynamic fluid spread threshold for subsurface aquifers.
    #[inline]
    pub fn fluid_level_spread(&self, x: f64, y: f64, z: f64) -> f64 {
        self.aquifer_fluid_level_spread.sample(x, y * 0.7142857142857143, z)
    }

    /// 4. `lava`: Subsurface lava noise. Fills underground voids below Y=-54 with Lava; water fills voids below Y=63.
    #[inline]
    pub fn lava(&self, x: f64, y: f64, z: f64) -> f64 {
        self.aquifer_lava.sample(x, y, z)
    }

    /// Determines aquifer backfill fluid at given block Y level.
    #[inline]
    pub fn aquifer_fluid_at(&self, y: i32) -> &'static str {
        if y < -54 {
            "minecraft:lava"
        } else if y < 63 {
            "minecraft:water"
        } else {
            "minecraft:air"
        }
    }

    /// 5. `temperature`: Continental temperature noise.
    #[inline]
    pub fn temperature(&self, x: f64, _y: f64, z: f64) -> f64 {
        let sx = self.offset.sample(x * 0.25, 0.0, z * 0.25) * 4.0;
        let sz = self.offset.sample(z * 0.25, 0.0, x * 0.25) * 4.0;
        self.temperature_noise.sample((x + sx) * 0.25, 0.0, (z + sz) * 0.25)
    }

    /// 6. `vegetation`: Continental vegetation noise.
    #[inline]
    pub fn vegetation(&self, x: f64, _y: f64, z: f64) -> f64 {
        let sx = self.offset.sample(x * 0.25, 0.0, z * 0.25) * 4.0;
        let sz = self.offset.sample(z * 0.25, 0.0, x * 0.25) * 4.0;
        self.vegetation_noise.sample((x + sx) * 0.25, 0.0, (z + sz) * 0.25)
    }

    /// 7. `continents`: Macro continentalness noise.
    #[inline]
    pub fn continents(&self, x: f64, _y: f64, z: f64) -> f64 {
        let sx = self.offset.sample(x * 0.25, 0.0, z * 0.25) * 4.0;
        let sz = self.offset.sample(z * 0.25, 0.0, x * 0.25) * 4.0;
        self.continentalness_noise.sample((x + sx) * 0.25, 0.0, (z + sz) * 0.25)
    }

    /// 8. `erosion`: Macro erosion noise.
    #[inline]
    pub fn erosion(&self, x: f64, _y: f64, z: f64) -> f64 {
        let sx = self.offset.sample(x * 0.25, 0.0, z * 0.25) * 4.0;
        let sz = self.offset.sample(z * 0.25, 0.0, x * 0.25) * 4.0;
        self.erosion_noise.sample((x + sx) * 0.25, 0.0, (z + sz) * 0.25)
    }

    /// 9. `depth`: Target depth / vertical position function with offset.
    #[inline]
    pub fn depth(&self, x: f64, y: f64, z: f64) -> f64 {
        let grad = 1.5 - 3.0 * ((y + 64.0) / 384.0).clamp(0.0, 1.0);
        let off = self.offset.sample(x * 0.25, 0.0, z * 0.25);
        grad + off
    }

    /// 10. `ridges`: Macro ridges / weirdness noise.
    #[inline]
    pub fn ridges(&self, x: f64, _y: f64, z: f64) -> f64 {
        let sx = self.offset.sample(x * 0.25, 0.0, z * 0.25) * 4.0;
        let sz = self.offset.sample(z * 0.25, 0.0, x * 0.25) * 4.0;
        self.ridge_noise.sample((x + sx) * 0.25, 0.0, (z + sz) * 0.25)
    }

    /// 11. `caves_sponge`: Large cheese caverns using 3D noise thresholding.
    pub fn caves_sponge(&self, x: f64, y: f64, z: f64, sloped_cheese: f64) -> f64 {
        let layer = self.cave_layer.sample(x, y * 8.0, z);
        let layer_term = 4.0 * (layer * layer);
        let cheese = self.cave_cheese.sample(x, y * (2.0 / 3.0), z);
        let cheese_term = (0.27 + cheese).clamp(-1.0, 1.0);
        let slope_bias = (1.5 - 0.64 * sloped_cheese).clamp(0.0, 0.5);
        layer_term + cheese_term + slope_bias
    }

    /// 12a. `caves_spaghetti_2d`: Twisting ribbon tunnels based on 2D dual-noise intersections.
    pub fn caves_spaghetti_2d(&self, x: f64, y: f64, z: f64) -> f64 {
        let thick_mod = -0.95 - 0.35 * self.spaghetti_2d_thickness.sample(x * 2.0, y, z);
        let elev = 8.0 * self.spaghetti_2d_elevation.sample(x, 0.0, z)
            + (8.0 + ((y + 64.0) / 384.0) * (-40.0 - 8.0));
        let elevated = (elev.abs() + thick_mod).powi(3);

        let mod_in = self.spaghetti_2d_modulator.sample(x * 2.0, y, z);
        let scale = if mod_in < -0.75 {
            2.0
        } else if mod_in < 0.0 {
            1.0
        } else if mod_in < 0.75 {
            2.0
        } else {
            3.0
        };

        let weird = scale * self.spaghetti_2d.sample(x / scale, y / scale, z / scale).abs()
            + 0.083 * thick_mod;

        weird.max(elevated).clamp(-1.0, 1.0)
    }

    /// 12b. `caves_spaghetti_roughness`: Modulated high-frequency roughness for ribbon tunnels.
    pub fn caves_spaghetti_roughness(&self, x: f64, y: f64, z: f64) -> f64 {
        let mod_val = self.spaghetti_roughness_modulator.sample(x, y, z);
        let rough_val = self.spaghetti_roughness.sample(x, y, z);
        (-0.05 - 0.05 * mod_val) * (-0.4 + rough_val.abs())
    }

    /// 13. `caves_noodle`: Fine connecting tunnels.
    pub fn caves_noodle(&self, x: f64, y: f64, z: f64) -> f64 {
        if y < -60.0 || y >= 321.0 {
            return 64.0;
        }
        let ndl = self.noodle.sample(x, y, z);
        if ndl < 0.0 {
            return 64.0;
        }
        let thick = -0.075 - 0.025 * self.noodle_thickness.sample(x, y, z);
        let ra = self
            .noodle_ridge_a
            .sample(x * 2.666667, y * 2.666667, z * 2.666667)
            .abs();
        let rb = self
            .noodle_ridge_b
            .sample(x * 2.666667, y * 2.666667, z * 2.666667)
            .abs();
        thick + 1.5 * ra.max(rb)
    }

    /// 14. `initial_density_without_jaggedness`: Base macro terrain shaping + vertical slides.
    pub fn initial_density_without_jaggedness(&self, x: f64, y: f64, z: f64) -> f64 {
        let depth_val = self.depth(x, y, z);
        let slider = vertical_density_slider(y as i32);
        (depth_val * slider).clamp(-64.0, 64.0)
    }

    /// Cave entrances sampler combining 3D spaghetti noise and vertical transition.
    pub fn sample_caves_entrances(&self, x: f64, y: f64, z: f64) -> f64 {
        let base_ent = 0.37 + self.cave_entrance.sample(x * 0.75, y * 0.5, z * 0.75);
        let grad = if y <= -10.0 {
            0.3
        } else if y >= 30.0 {
            0.0
        } else {
            0.3 * (1.0 - (y + 10.0) / 40.0)
        };
        let ent_part = base_ent + grad;

        let s3d_rarity = self.spaghetti_3d_rarity.sample(x * 2.0, y, z);
        let scale = if s3d_rarity < -0.5 {
            0.75
        } else if s3d_rarity < 0.0 {
            1.0
        } else if s3d_rarity < 0.5 {
            1.5
        } else {
            2.0
        };
        let s3d1 = scale * self.spaghetti_3d_1.sample(x / scale, y / scale, z / scale).abs();
        let s3d2 = scale * self.spaghetti_3d_2.sample(x / scale, y / scale, z / scale).abs();
        let thick = -0.0765 - 0.0115 * self.spaghetti_3d_thickness.sample(x, y, z);
        let s3d_part = self.caves_spaghetti_roughness(x, y, z) + (s3d1.max(s3d2) + thick).clamp(-1.0, 1.0);

        ent_part.min(s3d_part)
    }

    /// Subterranean cave pillars sampler.
    pub fn sample_caves_pillars(&self, x: f64, y: f64, z: f64) -> f64 {
        let p = self.pillars.sample(x * 25.0, y * 0.3, z * 25.0);
        let rareness = self.pillar_rareness.sample(x, y, z);
        let thick = self.pillar_thickness.sample(x, y, z);
        let term1 = 2.0 * p - 1.0 - rareness;
        let term2 = (0.55 + 0.55 * thick).powi(3);
        let val = term1 * term2;
        if val < 0.03 {
            -1000000.0
        } else {
            val
        }
    }

    /// 15. `final_density`: Evaluates complete merged density (sloped cheese, sponge, spaghetti, noodle, pillars).
    pub fn final_density(&self, x: f64, y: f64, z: f64, sloped_cheese: f64) -> f64 {
        let entrances = self.sample_caves_entrances(x, y, z);
        let caves_val = if sloped_cheese < 1.5625 {
            sloped_cheese.min(5.0 * entrances)
        } else {
            let sponge = self.caves_sponge(x, y, z, sloped_cheese);
            let s2d = self.caves_spaghetti_2d(x, y, z);
            let s_rough = self.caves_spaghetti_roughness(x, y, z);
            let pillars = self.sample_caves_pillars(x, y, z);

            let cavern_part = sponge.min(entrances);
            let tunnel_part = s2d + s_rough;
            cavern_part.min(tunnel_part).max(pillars)
        };

        let noodle_val = self.caves_noodle(x, y, z);
        let slider = vertical_density_slider(y as i32);
        let blended = caves_val * slider;
        blended.min(noodle_val)
    }

    // --- Ore Vein Samplers ---

    /// Evaluates ore vein selector noise channel.
    #[inline]
    pub fn vein_toggle(&self, x: f64, y: f64, z: f64) -> f64 {
        if y < -60.0 || y >= 51.0 {
            0.0
        } else {
            self.ore_veininess.sample(x * 1.5, y * 1.5, z * 1.5)
        }
    }

    /// Evaluates ore vein ridged thickness channel.
    #[inline]
    pub fn vein_ridged(&self, x: f64, y: f64, z: f64) -> f64 {
        if y < -60.0 || y >= 51.0 {
            -0.08
        } else {
            let va = self.ore_vein_a.sample(x * 4.0, y * 4.0, z * 4.0).abs();
            let vb = self.ore_vein_b.sample(x * 4.0, y * 4.0, z * 4.0).abs();
            -0.08 + va.max(vb)
        }
    }

    /// Evaluates ore vein gap noise channel.
    #[inline]
    pub fn vein_gap(&self, x: f64, y: f64, z: f64) -> f64 {
        self.ore_gap.sample(x, y, z)
    }

    /// Evaluates combined cave carving density modifier. Returns value <= 0 to carve air pocket.
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

/// Type alias for CaveNoiseRouter retaining 100% compatibility with existing modules.
pub type CaveNoiseRouter = NoiseRouter;

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
