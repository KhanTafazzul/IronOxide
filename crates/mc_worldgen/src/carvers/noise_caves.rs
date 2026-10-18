//! 3D Noise Cave Carvers: Cheese Caves, Spaghetti Caves, and Noodle Caves.
//! REA Reference: net.minecraft.world.level.levelgen.NoiseRouterData (doz.class, dox.class)
//! REA Reference: data/minecraft/worldgen/density_function/overworld/caves/*.json

use crate::seed_pipeline::noise::NormalNoise;
use crate::seed_pipeline::random::PositionalRandomFactory;

/// Cave classification type indicating which noise generator carved the voxel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveType {
    /// Large open cavern bubble formed by 3D cheese noise
    Cheese,
    /// Twisting tunnel network formed by two intersecting 3D noise channels
    Spaghetti3D,
    /// Flat horizontal cavern network formed by 2D spaghetti noise
    Spaghetti2D,
    /// Narrow, claustrophobic passage formed by high-frequency ridge noise
    Noodle,
}

/// Cheese Cave Sampler generating large open underground caverns.
/// REA Reference: cave_cheese (firstOctave: -8, amplitudes [0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0])
#[derive(Clone, Debug)]
pub struct CheeseCaveSampler {
    pub cave_cheese: NormalNoise,
    pub cave_layer: NormalNoise,
    pub cave_entrance: NormalNoise,
}

impl CheeseCaveSampler {
    pub fn new(factory: &PositionalRandomFactory) -> Self {
        let mut cc_rng = factory.from_hash_of("minecraft:cave_cheese");
        let cave_cheese = NormalNoise::from_rng(
            &mut cc_rng,
            -8,
            &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0],
        );

        let mut cl_rng = factory.from_hash_of("minecraft:cave_layer");
        let cave_layer = NormalNoise::from_rng(&mut cl_rng, -8, &[1.0]);

        let mut ce_rng = factory.from_hash_of("minecraft:cave_entrance");
        let cave_entrance = NormalNoise::from_rng(&mut ce_rng, -7, &[0.4, 0.5, 1.0]);

        Self {
            cave_cheese,
            cave_layer,
            cave_entrance,
        }
    }

    /// Evaluates if position (x, y, z) is inside a Cheese Cave cavern.
    /// Large open caverns are formed where 3D cheese noise exceeds positive threshold bounds.
    #[inline]
    pub fn is_cheese_cave(&self, x: f64, y: f64, z: f64) -> bool {
        // Cheese caves are bounded underground between Y=-54 and Y=180
        if y < -54.0 || y > 180.0 {
            return false;
        }

        // 3D Cheese noise: density = 0.27 + (2/3) * noise
        let raw_cheese = self.cave_cheese.sample(x, y, z);
        let cheese_val = 0.27 + (2.0 / 3.0) * raw_cheese;

        // Layer modulation
        let raw_layer = self.cave_layer.sample(x, y, z);
        let layer_mod = 1.0 - (raw_layer.abs() * 0.35);

        // Entrance modulation
        let entrance = self.cave_entrance.sample(x * 0.75, y * 0.5, z * 0.75);
        let threshold = 0.12 - entrance * 0.05;

        // Positive density exceeding threshold hollows out large cavern chamber
        cheese_val * layer_mod > threshold
    }
}

/// Spaghetti Cave Sampler generating long twisting tunnels.
/// REA Reference: spaghetti_3d_1, spaghetti_3d_2, spaghetti_2d
#[derive(Clone, Debug)]
pub struct SpaghettiCaveSampler {
    pub spaghetti_3d_1: NormalNoise,
    pub spaghetti_3d_2: NormalNoise,
    pub spaghetti_3d_rarity: NormalNoise,
    pub spaghetti_3d_thickness: NormalNoise,
    pub spaghetti_roughness: NormalNoise,
    pub spaghetti_roughness_modulator: NormalNoise,
    pub spaghetti_2d: NormalNoise,
    pub spaghetti_2d_elevation: NormalNoise,
    pub spaghetti_2d_thickness: NormalNoise,
}

impl SpaghettiCaveSampler {
    pub fn new(factory: &PositionalRandomFactory) -> Self {
        let mut s3d1_rng = factory.from_hash_of("minecraft:spaghetti_3d_1");
        let spaghetti_3d_1 = NormalNoise::from_rng(&mut s3d1_rng, -7, &[1.0, 1.0]);

        let mut s3d2_rng = factory.from_hash_of("minecraft:spaghetti_3d_2");
        let spaghetti_3d_2 = NormalNoise::from_rng(&mut s3d2_rng, -7, &[1.0, 1.0]);

        let mut s3dr_rng = factory.from_hash_of("minecraft:spaghetti_3d_rarity");
        let spaghetti_3d_rarity = NormalNoise::from_rng(&mut s3dr_rng, -11, &[1.0]);

        let mut s3dt_rng = factory.from_hash_of("minecraft:spaghetti_3d_thickness");
        let spaghetti_3d_thickness = NormalNoise::from_rng(&mut s3dt_rng, -8, &[1.0]);

        let mut sr_rng = factory.from_hash_of("minecraft:spaghetti_roughness");
        let spaghetti_roughness = NormalNoise::from_rng(&mut sr_rng, -5, &[1.0]);

        let mut srm_rng = factory.from_hash_of("minecraft:spaghetti_roughness_modulator");
        let spaghetti_roughness_modulator = NormalNoise::from_rng(&mut srm_rng, -8, &[1.0]);

        let mut s2d_rng = factory.from_hash_of("minecraft:spaghetti_2d");
        let spaghetti_2d = NormalNoise::from_rng(&mut s2d_rng, -7, &[1.0]);

        let mut s2de_rng = factory.from_hash_of("minecraft:spaghetti_2d_elevation");
        let spaghetti_2d_elevation = NormalNoise::from_rng(&mut s2de_rng, -8, &[1.0]);

        let mut s2dt_rng = factory.from_hash_of("minecraft:spaghetti_2d_thickness");
        let spaghetti_2d_thickness = NormalNoise::from_rng(&mut s2dt_rng, -11, &[1.0]);

        Self {
            spaghetti_3d_1,
            spaghetti_3d_2,
            spaghetti_3d_rarity,
            spaghetti_3d_thickness,
            spaghetti_roughness,
            spaghetti_roughness_modulator,
            spaghetti_2d,
            spaghetti_2d_elevation,
            spaghetti_2d_thickness,
        }
    }

    /// Evaluates if position is inside a 3D Spaghetti tunnel (sqrt(N1^2 + N2^2) < radius).
    #[inline]
    pub fn is_spaghetti_3d_cave(&self, x: f64, y: f64, z: f64) -> bool {
        if y < -58.0 || y > 210.0 {
            return false;
        }

        // Two intersecting 3D noise channels
        let n1 = self.spaghetti_3d_1.sample(x, y, z);
        let n2 = self.spaghetti_3d_2.sample(x, y, z);
        let dist = (n1 * n1 + n2 * n2).sqrt();

        // Thickness modulation
        let thick = self.spaghetti_3d_thickness.sample(x, y, z);
        let radius = 0.28 + thick * 0.05;

        // Roughness for organic cave walls
        let roughness = self.spaghetti_roughness.sample(x * 2.0, y * 2.0, z * 2.0);
        let rough_mod = self.spaghetti_roughness_modulator.sample(x, y, z);
        let wall_offset = (roughness * rough_mod).abs() * 0.025;

        dist < (radius + wall_offset)
    }

    /// Evaluates if position is inside a 2D Spaghetti horizontal cave layer.
    #[inline]
    pub fn is_spaghetti_2d_cave(&self, x: f64, y: f64, z: f64) -> bool {
        if y < -50.0 || y > 150.0 {
            return false;
        }

        let n2d = self.spaghetti_2d.sample(x * 2.0, 0.0, z * 2.0);
        if n2d.abs() > 0.15 {
            return false;
        }

        // Vertical elevation target centered along noise plane
        let elev = self.spaghetti_2d_elevation.sample(x, 0.0, z);
        let target_y = 30.0 + elev * 8.0;
        let dy = (y - target_y).abs();

        let thick = self.spaghetti_2d_thickness.sample(x, 0.0, z);
        let half_height = 2.5 + thick * 1.5;

        dy < half_height
    }
}

/// Noodle Cave Sampler generating narrow, claustrophobic twisting passages.
/// REA Reference: noodle, noodle_ridge_a, noodle_ridge_b, noodle_thickness
#[derive(Clone, Debug)]
pub struct NoodleCaveSampler {
    pub noodle: NormalNoise,
    pub noodle_thickness: NormalNoise,
    pub noodle_ridge_a: NormalNoise,
    pub noodle_ridge_b: NormalNoise,
}

impl NoodleCaveSampler {
    pub fn new(factory: &PositionalRandomFactory) -> Self {
        let mut ndl_rng = factory.from_hash_of("minecraft:noodle");
        let noodle = NormalNoise::from_rng(&mut ndl_rng, -8, &[1.0]);

        let mut nth_rng = factory.from_hash_of("minecraft:noodle_thickness");
        let noodle_thickness = NormalNoise::from_rng(&mut nth_rng, -8, &[1.0]);

        let mut nra_rng = factory.from_hash_of("minecraft:noodle_ridge_a");
        let noodle_ridge_a = NormalNoise::from_rng(&mut nra_rng, -7, &[1.0]);

        let mut nrb_rng = factory.from_hash_of("minecraft:noodle_ridge_b");
        let noodle_ridge_b = NormalNoise::from_rng(&mut nrb_rng, -7, &[1.0]);

        Self {
            noodle,
            noodle_thickness,
            noodle_ridge_a,
            noodle_ridge_b,
        }
    }

    /// Evaluates if position is inside a Noodle Cave passage.
    #[inline]
    pub fn is_noodle_cave(&self, x: f64, y: f64, z: f64) -> bool {
        if y < -55.0 || y > 160.0 {
            return false;
        }

        // Two ridge lines evaluated at 2.666667x scale
        let scale = 8.0 / 3.0;
        let ra = self.noodle_ridge_a.sample(x * scale, y * scale, z * scale).abs();
        let rb = self.noodle_ridge_b.sample(x * scale, y * scale, z * scale).abs();
        let max_ridge = ra.max(rb);

        let thick = self.noodle_thickness.sample(x, y, z);
        // Formula: 1.5 * max(|ra|, |rb|) - 0.28 - 0.05 * thickness < 0
        let ridge_val = 1.5 * max_ridge - 0.28 - 0.05 * thick;

        ridge_val < 0.0
    }
}

/// Unified 3D Noise Cave Carver Router combining Cheese, Spaghetti, and Noodle generators.
#[derive(Clone, Debug)]
pub struct CaveCarverRouter {
    pub cheese: CheeseCaveSampler,
    pub spaghetti: SpaghettiCaveSampler,
    pub noodle: NoodleCaveSampler,
}

impl CaveCarverRouter {
    pub fn new(world_seed: i64) -> Self {
        let factory = PositionalRandomFactory::from_world_seed(world_seed);
        Self {
            cheese: CheeseCaveSampler::new(&factory),
            spaghetti: SpaghettiCaveSampler::new(&factory),
            noodle: NoodleCaveSampler::new(&factory),
        }
    }

    /// Checks if a coordinate (x, y, z) is carved by any of the 3D noise cave generators.
    /// Returns the CaveType if carved, or None if solid terrain remains.
    #[inline]
    pub fn sample_cave(&self, x: f64, y: f64, z: f64) -> Option<CaveType> {
        // Bedrock floor protection (Y <= -60 is never carved)
        if y <= -60.0 {
            return None;
        }

        // 1. Cheese Caves: Large open caverns
        if self.cheese.is_cheese_cave(x, y, z) {
            return Some(CaveType::Cheese);
        }

        // 2. Spaghetti 3D Caves: Twisting tunnel network
        if self.spaghetti.is_spaghetti_3d_cave(x, y, z) {
            return Some(CaveType::Spaghetti3D);
        }

        // 3. Spaghetti 2D Caves: Wide horizontal subterranean branches
        if self.spaghetti.is_spaghetti_2d_cave(x, y, z) {
            return Some(CaveType::Spaghetti2D);
        }

        // 4. Noodle Caves: Narrow claustrophobic passages
        if self.noodle.is_noodle_cave(x, y, z) {
            return Some(CaveType::Noodle);
        }

        None
    }
}
