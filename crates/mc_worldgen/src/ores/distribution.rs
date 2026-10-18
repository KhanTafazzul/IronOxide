//! Ore height distribution providers and trapezoid curve evaluators.
//! REA Reference: net.minecraft.world.level.levelgen.placement.HeightRangePlacement (dyf.class)
//! REA Reference: net.minecraft.world.level.levelgen.heightproviders.TrapezoidHeightProvider (dxp.class)
//! REA Reference: net.minecraft.world.level.levelgen.heightproviders.UniformHeightProvider (dxq.class)
//! REA Reference: net.minecraft.data.worldgen.placement.OrePlacements (qx.class)

use crate::structures::LegacyRandom;

/// Height distribution curves controlling ore elevation placement in the world.
#[derive(Clone, Debug, PartialEq)]
pub enum HeightDistribution {
    /// Uniform distribution between `min_inclusive` and `max_inclusive`.
    /// REA Reference: `dxq.class` (`UniformHeightProvider`)
    Uniform {
        min_inclusive: i32,
        max_inclusive: i32,
    },
    /// Trapezoid distribution between `min_inclusive` and `max_inclusive` with plateau width.
    /// When `plateau == 0`, forms an exact symmetric triangle with peak at `(min + max) / 2`.
    /// REA Reference: `dxp.class` (`TrapezoidHeightProvider`)
    Trapezoid {
        min_inclusive: i32,
        max_inclusive: i32,
        plateau: i32,
    },
    /// Biased to bottom distribution with higher probability near `min_inclusive`.
    /// REA Reference: `dxl.class` (`BiasedToBottomHeightProvider`)
    BiasedToBottom {
        min_inclusive: i32,
        max_inclusive: i32,
        bias: i32,
    },
}

impl HeightDistribution {
    /// Creates a uniform height distribution.
    #[inline]
    pub const fn uniform(min: i32, max: i32) -> Self {
        Self::Uniform {
            min_inclusive: min,
            max_inclusive: max,
        }
    }

    /// Creates a symmetric triangular distribution with peak at `(min + max) / 2`.
    #[inline]
    pub const fn triangular(min: i32, max: i32) -> Self {
        Self::Trapezoid {
            min_inclusive: min,
            max_inclusive: max,
            plateau: 0,
        }
    }

    /// Creates a trapezoid distribution with a flat plateau at the peak.
    #[inline]
    pub const fn trapezoid(min: i32, max: i32, plateau: i32) -> Self {
        Self::Trapezoid {
            min_inclusive: min,
            max_inclusive: max,
            plateau,
        }
    }

    /// Samples a Y coordinate from this distribution using the provided Java LCG RNG.
    /// REA Reference: `dxp.a(RandomSource, LevelHeightAccessor)`
    pub fn sample(&self, rng: &mut LegacyRandom) -> i32 {
        match *self {
            HeightDistribution::Uniform { min_inclusive, max_inclusive } => {
                if min_inclusive >= max_inclusive {
                    return min_inclusive;
                }
                let range = max_inclusive - min_inclusive + 1;
                min_inclusive + rng.next_int_bounded(range)
            }
            HeightDistribution::Trapezoid { min_inclusive, max_inclusive, plateau } => {
                if min_inclusive >= max_inclusive {
                    return min_inclusive;
                }
                let range = max_inclusive - min_inclusive;
                if plateau >= range {
                    return min_inclusive + rng.next_int_bounded(range + 1);
                }
                // REA Reference: dxp.class lines 65-100:
                // int diff = (range - plateau) / 2;
                // int k = range - diff;
                // return min + random.nextInt(k + 1) + random.nextInt(diff + 1);
                let diff = (range - plateau) / 2;
                let k = range - diff;
                min_inclusive + rng.next_int_bounded(k + 1) + rng.next_int_bounded(diff + 1)
            }
            HeightDistribution::BiasedToBottom { min_inclusive, max_inclusive, bias } => {
                if min_inclusive >= max_inclusive {
                    return min_inclusive;
                }
                let range = max_inclusive - min_inclusive;
                let bound = (range + 1 - bias).max(1);
                let r1 = rng.next_int_bounded(bound);
                let r2 = rng.next_int_bounded(r1 + bias + 1);
                min_inclusive + r2
            }
        }
    }

    /// Computes the exact theoretical probability density at elevation `y`.
    /// Useful for verifying distribution peaks, plateaus, and boundaries.
    pub fn probability_density(&self, y: i32) -> f64 {
        match *self {
            HeightDistribution::Uniform { min_inclusive, max_inclusive } => {
                if y < min_inclusive || y > max_inclusive {
                    0.0
                } else {
                    1.0 / ((max_inclusive - min_inclusive + 1) as f64)
                }
            }
            HeightDistribution::Trapezoid { min_inclusive, max_inclusive, plateau } => {
                if y < min_inclusive || y > max_inclusive {
                    return 0.0;
                }
                let range = (max_inclusive - min_inclusive) as f64;
                if plateau as f64 >= range {
                    return 1.0 / (range + 1.0);
                }
                let diff = ((range - plateau as f64) / 2.0).floor();
                let k = range - diff;
                let offset = (y - min_inclusive) as f64;

                // Convolution of two discrete uniform distributions U(0, k) and U(0, diff)
                let num_ways = (0..=(k as i32)).filter(|&i| {
                    let j = (offset as i32) - i;
                    j >= 0 && j <= (diff as i32)
                }).count();

                let total_outcomes = (k + 1.0) * (diff + 1.0);
                (num_ways as f64) / total_outcomes
            }
            HeightDistribution::BiasedToBottom { min_inclusive, max_inclusive, .. } => {
                if y < min_inclusive || y > max_inclusive {
                    0.0
                } else {
                    // Density monotonically decreases from min to max
                    let range = (max_inclusive - min_inclusive) as f64;
                    let norm = (max_inclusive - y) as f64 / range;
                    norm / (range / 2.0)
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Canonical Vanilla Overworld Ore Distribution Presets
// REA Reference: net.minecraft.data.worldgen.placement.OrePlacements (qx.class)
// ----------------------------------------------------------------------------

/// Coal Upper: Uniform Y = 136 to 256 (or 320).
/// REA Reference: `qx.r` -> `dyf.a(136, top)`
#[inline]
pub fn coal_upper_distribution() -> HeightDistribution {
    HeightDistribution::uniform(136, 256)
}

/// Coal Lower: Triangular distribution Y = 0 to 192 with peak at Y = 96.
/// REA Reference: `qx.s` -> `dyf.b(0, 192)`
#[inline]
pub fn coal_lower_distribution() -> HeightDistribution {
    HeightDistribution::triangular(0, 192)
}

/// Iron Upper: Triangular distribution Y = 80 to 384 with peak at Y = 232 (peaks in mountains).
/// REA Reference: `qx.t` -> `dyf.b(80, 384)`
#[inline]
pub fn iron_upper_distribution() -> HeightDistribution {
    HeightDistribution::triangular(80, 384)
}

/// Iron Middle: Triangular distribution Y = -24 to 56 with peak at Y = 16.
/// REA Reference: `qx.u` -> `dyf.b(-24, 56)`
#[inline]
pub fn iron_middle_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-24, 56)
}

/// Iron Small: Uniform distribution Y = -64 to 72.
/// REA Reference: `qx.v` -> `dyf.a(bottom, 72)`
#[inline]
pub fn iron_small_distribution() -> HeightDistribution {
    HeightDistribution::uniform(-64, 72)
}

/// Gold Regular: Uniform Y = -64 to -32 (or trapezoid -64 to 32 with peak at -16).
/// REA Reference: `qx.x` / `qx.y`
#[inline]
pub fn gold_regular_distribution() -> HeightDistribution {
    HeightDistribution::uniform(-64, -32)
}

/// Gold Extra (Badlands Biomes): High-abundance uniform distribution Y = 32 to 256.
/// REA Reference: `qx.w` -> `dyf.a(32, 256)`
#[inline]
pub fn gold_badlands_extra_distribution() -> HeightDistribution {
    HeightDistribution::uniform(32, 256)
}

/// Redstone Regular: Uniform distribution Y = -64 to 15.
/// REA Reference: `qx.z` -> `dyf.a(bottom, 15)`
#[inline]
pub fn redstone_regular_distribution() -> HeightDistribution {
    HeightDistribution::uniform(-64, 15)
}

/// Redstone Lower: Trapezoid centered at bedrock bottom Y = -64, increasing density toward bedrock.
/// REA Reference: `qx.A` -> `dyf.b(aboveBottom(-32), aboveBottom(32))`
#[inline]
pub fn redstone_lower_distribution() -> HeightDistribution {
    // Evaluates between -96 and -32 (clamped/centered at bedrock Y = -64)
    HeightDistribution::triangular(-96, -32)
}

/// Copper Regular: Triangular distribution Y = -16 to 112 with peak at Y = 48 (size 10, count 16).
/// REA Reference: `qx.L` -> `dyf.b(-16, 112)`
#[inline]
pub fn copper_regular_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-16, 112)
}

/// Copper Dripstone Caves (Large): Triangular distribution Y = -16 to 112 with peak at Y = 48 (size 20).
/// REA Reference: `qx.M` -> `dyf.b(-16, 112)`
#[inline]
pub fn copper_dripstone_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-16, 112)
}

/// Gold Regular (Triangular): Triangular distribution Y = -64 to 32 with peak at Y = -16 (size 9).
/// REA Reference: `qx.x` / `qx.y` -> `dyf.b(-64, 32)`
#[inline]
pub fn gold_regular_triangular_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-64, 32)
}

/// Lapis Lazuli Regular: Triangular distribution Y = -32 to 32 with peak at Y = 0 (size 7).
/// REA Reference: `qx.F` -> `dyf.b(-32, 32)`
#[inline]
pub fn lapis_regular_distribution() -> HeightDistribution {
    HeightDistribution::triangular(-32, 32)
}

/// Lapis Lazuli Buried: Uniform distribution Y = -64 to 64 (size 7).
/// REA Reference: `qx.G` -> `dyf.a(bottom, 64)`
#[inline]
pub fn lapis_buried_distribution() -> HeightDistribution {
    HeightDistribution::uniform(-64, 64)
}

/// Diamond: Triangular distribution from Y = 16 down to Y = -64, with peak density at bedrock level (Y = -64).
/// REA Reference: `qx.B` / `qx.D` -> `dyf.b(aboveBottom(-80), aboveBottom(80))`
#[inline]
pub fn diamond_distribution() -> HeightDistribution {
    // Range is [-144, 16], center/peak is at (-144 + 16)/2 = -64 (Bedrock level)
    HeightDistribution::triangular(-144, 16)
}
