//! Barrier noise evaluation and pressure calculation for leak prevention and dry pocket preservation.
//! REA Reference: net.minecraft.world.level.levelgen.NoiseBasedAquifer (doc$c.class)
//! REA Reference: methods similarity (377), calculatePressure (388)
//! REA Reference: data/minecraft/worldgen/noise/aquifer_barrier.json (firstOctave: -3, [1.0])

use super::fluid::{AquiferFluidType, FluidStatus};
use crate::seed_pipeline::noise::NormalNoise;
use crate::seed_pipeline::random::PositionalRandomFactory;

/// Barrier noise sampler computing fluid dividing barriers and dry pocket dams.
#[derive(Clone, Debug)]
pub struct BarrierNoiseSampler {
    pub barrier_noise: NormalNoise,
}

impl BarrierNoiseSampler {
    /// Constructs a barrier noise sampler matching vanilla 1.20.4 parameters.
    pub fn new(factory: &PositionalRandomFactory) -> Self {
        let mut rng = factory.from_hash_of("minecraft:aquifer_barrier");
        let barrier_noise = NormalNoise::from_rng(&mut rng, -3, &[1.0]);
        Self { barrier_noise }
    }

    /// Evaluates spatial similarity between two cell distances squared.
    /// REA Reference: doc$c.similarity(dist1, dist2)
    /// Returns a value in [0.0, 1.0] indicating proximity to the Voronoi boundary between cells.
    /// If > 0.0, the point is within the 25-block transition zone.
    #[inline]
    pub fn similarity(dist1: i32, dist2: i32) -> f64 {
        1.0 - ((dist2 - dist1).abs() as f64) / 25.0
    }

    /// Calculates fluid barrier pressure at elevation Y between two neighboring fluid statuses.
    /// REA Reference: doc$c.calculatePressure(context, status1, status2)
    ///
    /// Rules:
    /// 1. Water vs Lava boundary creates an unconditional high-pressure stone barrier (pressure = 2.0).
    /// 2. Equal fluid levels have zero pressure (pressure = 0.0).
    /// 3. Asymmetric vertical pressure curve ensures natural stone dams preventing flooding from leaking into dry caverns.
    pub fn calculate_pressure(
        &self,
        x: f64,
        y: i32,
        z: f64,
        status1: &FluidStatus,
        status2: &FluidStatus,
    ) -> f64 {
        let fluid1 = status1.at_elevation(y);
        let fluid2 = status2.at_elevation(y);

        // Water meeting Lava: unconditional stone partition
        let water_lava_interface = (fluid1 == AquiferFluidType::Water && fluid2 == AquiferFluidType::Lava)
            || (fluid1 == AquiferFluidType::Lava && fluid2 == AquiferFluidType::Water);
        if water_lava_interface {
            return 2.0;
        }

        let diff = (status1.fluid_level - status2.fluid_level).abs();
        if diff == 0 {
            return 0.0;
        }

        let mid_level = 0.5 * (status1.fluid_level as f64 + status2.fluid_level as f64);
        let delta_y = (y as f64) + 0.5 - mid_level;
        let half_diff = (diff as f64) / 2.0;
        let dist_from_mid = half_diff - delta_y.abs();

        let pressure = if delta_y > 0.0 {
            let p = dist_from_mid;
            if p > 0.0 {
                p / 1.5
            } else {
                p / 2.5
            }
        } else {
            let p = 3.0 + dist_from_mid;
            if p > 0.0 {
                p / 3.0
            } else {
                p / 10.0
            }
        };

        // If pressure is beyond [-2.0, 2.0], barrier noise is muted
        let barrier_sample = if pressure < -2.0 || pressure > 2.0 {
            0.0
        } else {
            self.barrier_noise.sample(x, y as f64, z)
        };

        2.0 * (barrier_sample + pressure)
    }

    /// Evaluates if a solid rock barrier forms at (x, y, z) preventing fluid leaking.
    /// Returns true if an impermeable stone wall should block fluid flow.
    pub fn is_barrier_wall(
        &self,
        x: f64,
        y: i32,
        z: f64,
        similarity: f64,
        status1: &FluidStatus,
        status2: &FluidStatus,
    ) -> bool {
        if similarity <= 0.0 {
            return false;
        }

        let pressure = self.calculate_pressure(x, y, z, status1, status2);
        let adjusted_density = similarity * pressure;
        adjusted_density > 0.0
    }
}
