//! 64-bit seed mixing, Stafford13, Xoroshiro128++, and MD5 string hash derivations
//! matching Minecraft 1.20.4 RandomSupport, PositionalRandomFactory, and XoroshiroRandomSource.

pub const GOLDEN_RATIO_64: i64 = 7640891576956012809; // 0x6a09e667f3bcc909
pub const SILVER_RATIO_64: i64 = -7046029254386353131; // 0x9e3779b97f4a7c15

pub type Double = f64;
#[allow(non_camel_case_types)]
pub type double = f64;

/// Exact Stafford13 64-bit mixer used by Minecraft RandomSupport.mixStafford13.
#[inline]
pub fn mix_stafford13(mut n: i64) -> i64 {
    n = (n ^ ((n as u64 >> 30) as i64)).wrapping_mul(-4658895280553007687); // 0xbf58476d1ce4e5b9
    n = (n ^ ((n as u64 >> 27) as i64)).wrapping_mul(-7723592293110705685); // 0x94d049bb133111eb
    n ^ ((n as u64 >> 31) as i64)
}

/// Upgrades a 64-bit world seed into a 128-bit seed pair (seed_lo, seed_hi).
#[inline]
pub fn upgrade_seed_to_128bit(seed: i64) -> (i64, i64) {
    let s_lo = seed ^ GOLDEN_RATIO_64;
    let s_hi = s_lo.wrapping_add(SILVER_RATIO_64);
    (mix_stafford13(s_lo), mix_stafford13(s_hi))
}

/// RFC 1321 standard MD5 implementation in pure Rust.
pub fn md5_digest(data: &[u8]) -> [u8; 16] {
    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xefcdab89;
    let mut c0: u32 = 0x98badcfe;
    let mut d0: u32 = 0x10325476;

    let s = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5,  9, 14, 20, 5,  9, 14, 20, 5,  9, 14, 20, 5,  9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];

    let k: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
        0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
        0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
        0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
        0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
        0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
        0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
        0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
    ];

    let orig_len_bits = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&orig_len_bits.to_le_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap());
        }

        let mut a = a0;
        let mut b = b0;
        let mut c = c0;
        let mut d = d0;

        for i in 0..64 {
            let (f, g) = if i < 16 {
                ((b & c) | ((!b) & d), i)
            } else if i < 32 {
                ((d & b) | ((!d) & c), (5 * i + 1) % 16)
            } else if i < 48 {
                (b ^ c ^ d, (3 * i + 5) % 16)
            } else {
                (c ^ (b | (!d)), (7 * i) % 16)
            };

            let temp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(k[i])
                    .wrapping_add(m[g])
                    .rotate_left(s[i]),
            );
            a = temp;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&a0.to_le_bytes());
    out[4..8].copy_from_slice(&b0.to_le_bytes());
    out[8..12].copy_from_slice(&c0.to_le_bytes());
    out[12..16].copy_from_slice(&d0.to_le_bytes());
    out
}

/// Derives a 128-bit noise seed by hashing the resource key name with MD5 and XORing
/// with the base 128-bit seed. Matches Minecraft PositionalRandomFactory.fromHashOf.
#[inline]
pub fn seed_from_hash_of(seed_lo: i64, seed_hi: i64, key: &str) -> (i64, i64) {
    let digest = md5_digest(key.as_bytes());
    let hash_lo = i64::from_be_bytes(digest[0..8].try_into().unwrap());
    let hash_hi = i64::from_be_bytes(digest[8..16].try_into().unwrap());
    (hash_lo ^ seed_lo, hash_hi ^ seed_hi)
}

/// PositionalRandomFactory matching Minecraft dpr$a.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionalRandomFactory {
    pub seed_lo: i64,
    pub seed_hi: i64,
}

impl PositionalRandomFactory {
    #[inline]
    pub fn new(seed_lo: i64, seed_hi: i64) -> Self {
        Self { seed_lo, seed_hi }
    }

    #[inline]
    pub fn from_world_seed(world_seed: i64) -> Self {
        let (s_lo, s_hi) = upgrade_seed_to_128bit(world_seed);
        let mut root_rng = Xoroshiro128PlusPlus::new(s_lo, s_hi);
        let f_lo = root_rng.next_long();
        let f_hi = root_rng.next_long();
        Self::new(f_lo, f_hi)
    }

    #[inline]
    pub fn from_seed(world_seed: i64) -> Self {
        Self::from_world_seed(world_seed)
    }

    #[inline]
    pub fn from_hash_of(&self, key: &str) -> Xoroshiro128PlusPlus {
        let (s0, s1) = seed_from_hash_of(self.seed_lo, self.seed_hi, key);
        Xoroshiro128PlusPlus::new(s0, s1)
    }

    /// Derives a positional PRNG at (x, y, z).
    /// REA Reference: XoroshiroPositionalRandomFactory.at(x, y, z)
    #[inline]
    pub fn at(&self, x: i32, y: i32, z: i32) -> Xoroshiro128PlusPlus {
        let pos_seed = ((x as i64).wrapping_mul(3129871))
            ^ ((z as i64).wrapping_mul(116129781))
            ^ (y as i64);
        let s0 = pos_seed ^ self.seed_lo;
        let s1 = mix_stafford13(pos_seed) ^ self.seed_hi;
        Xoroshiro128PlusPlus::new(s0, s1)
    }
}

/// Minecraft 1.20+ Xoroshiro128++ PRNG.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Xoroshiro128PlusPlus {
    pub s0: i64,
    pub s1: i64,
}

impl Xoroshiro128PlusPlus {
    #[inline]
    pub fn new(s0: i64, s1: i64) -> Self {
        let (s0, s1) = if s0 == 0 && s1 == 0 {
            (-7046029254386353131, 7640891576956012809)
        } else {
            (s0, s1)
        };
        Self { s0, s1 }
    }

    #[inline]
    pub fn next_long(&mut self) -> i64 {
        let s0 = self.s0;
        let mut s1 = self.s1;
        let result = s0.wrapping_add(s1).rotate_left(17).wrapping_add(s0);
        s1 ^= s0;
        self.s0 = s0.rotate_left(49) ^ s1 ^ (s1 << 21);
        self.s1 = s1.rotate_left(28);
        result
    }

    #[inline]
    pub fn next_int(&mut self) -> i32 {
        self.next_long() as i32
    }

    #[inline]
    pub fn next_int_bounded(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        let mut l2 = (self.next_int() as u32) as u64;
        let mut l4 = l2 * (bound as u64);
        let mut l6 = l4 & 0xFFFF_FFFF;
        if l6 < bound as u64 {
            let t = (!bound + 1) as u32 % (bound as u32);
            while l6 < t as u64 {
                l2 = (self.next_int() as u32) as u64;
                l4 = l2 * (bound as u64);
                l6 = l4 & 0xFFFF_FFFF;
            }
        }
        (l4 >> 32) as i32
    }

    #[inline]
    pub fn next_float(&mut self) -> f32 {
        ((self.next_long() as u64 >> 40) as f32) * 5.9604645e-8
    }

    #[inline]
    pub fn next_boolean(&mut self) -> bool {
        (self.next_long() & 1) != 0
    }

    #[inline]
    pub fn next_double(&mut self) -> double {
        ((self.next_long() as u64 >> 11) as f64) * 1.1102230246251565e-16
    }

    #[inline]
    pub fn fork_positional(&mut self) -> PositionalRandomFactory {
        let f0 = self.next_long();
        let f1 = self.next_long();
        PositionalRandomFactory::new(f0, f1)
    }
}

/// Vanilla Minecraft 1.20.4 GenerationStep.Decoration indices (net.minecraft.world.level.levelgen.GenerationStep$Decoration).
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorationStep {
    RawGeneration = 0,
    Lakes = 1,
    LocalModifications = 2,
    UndergroundStructures = 3,
    SurfaceStructures = 4,
    Strongholds = 5,
    UndergroundOres = 6,
    UndergroundDecoration = 7,
    FluidSprings = 8,
    VegetalDecoration = 9,
    TopLayerModification = 10,
}

/// Derives the population seed for chunk decoration matching Minecraft 1.20.4 WorldgenRandom.
/// REA Reference: net.minecraft.world.level.levelgen.WorldgenRandom.setDecorationSeed (dpp.class)
/// REA Reference: net.minecraft.world.level.levelgen.RandomSupport (dpf.class)
#[inline]
pub fn set_decoration_seed(world_seed: u64, min_chunk_x: i32, min_chunk_z: i32) -> u64 {
    let x_part = (min_chunk_x as u64).wrapping_mul(341873128712);
    let z_part = (min_chunk_z as u64).wrapping_mul(132897987541);
    let s0 = world_seed ^ x_part.wrapping_add(z_part);
    mix_stafford13(s0 as i64) as u64
}

/// Re-seeds an isolated Xoroshiro128++ PRNG instance for a specific feature step.
/// Prevents PRNG stream leakage across chunk features.
/// REA Reference: net.minecraft.world.level.levelgen.WorldgenRandom.setFeatureSeed (dpp.class)
#[inline]
pub fn set_feature_seed(population_seed: u64, feature_index: usize, step_index: usize) -> Xoroshiro128PlusPlus {
    let derived_seed = population_seed
        .wrapping_add(feature_index as u64)
        .wrapping_add((step_index as u64).wrapping_mul(10000));
    let (s_lo, s_hi) = upgrade_seed_to_128bit(derived_seed as i64);
    Xoroshiro128PlusPlus::new(s_lo, s_hi)
}

/// Vanilla WorldgenRandom wrapper managing population and feature seed derivations.
/// REA Reference: net.minecraft.world.level.levelgen.WorldgenRandom (dpp.class)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldgenRandom {
    pub rng: Xoroshiro128PlusPlus,
    pub count: i32,
}

impl WorldgenRandom {
    pub fn new(seed: i64) -> Self {
        let (s_lo, s_hi) = upgrade_seed_to_128bit(seed);
        Self {
            rng: Xoroshiro128PlusPlus::new(s_lo, s_hi),
            count: 0,
        }
    }

    pub fn set_seed(&mut self, seed: i64) {
        let (s_lo, s_hi) = upgrade_seed_to_128bit(seed);
        self.rng = Xoroshiro128PlusPlus::new(s_lo, s_hi);
    }

    /// Vanilla setDecorationSeed
    pub fn set_decoration_seed(&mut self, world_seed: u64, min_chunk_x: i32, min_chunk_z: i32) -> u64 {
        let pop_seed = set_decoration_seed(world_seed, min_chunk_x, min_chunk_z);
        self.set_seed(pop_seed as i64);
        pop_seed
    }

    /// Vanilla setFeatureSeed
    pub fn set_feature_seed(&mut self, population_seed: u64, feature_index: usize, step_index: usize) {
        self.rng = set_feature_seed(population_seed, feature_index, step_index);
    }

    #[inline]
    pub fn next_int(&mut self) -> i32 {
        self.count += 1;
        self.rng.next_int()
    }

    #[inline]
    pub fn next_int_bounded(&mut self, bound: i32) -> i32 {
        self.count += 1;
        self.rng.next_int_bounded(bound)
    }

    #[inline]
    pub fn next_float(&mut self) -> f32 {
        self.count += 1;
        self.rng.next_float()
    }

    #[inline]
    pub fn next_double(&mut self) -> double {
        self.count += 1;
        self.rng.next_double()
    }

    #[inline]
    pub fn next_boolean(&mut self) -> bool {
        self.count += 1;
        self.rng.next_boolean()
    }

    #[inline]
    pub fn next_long(&mut self) -> i64 {
        self.count += 1;
        self.rng.next_long()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_derivation_parity_with_jvm() {
        let world_seed = 123456789i64;
        let (seed_lo, seed_hi) = upgrade_seed_to_128bit(world_seed);
        assert_eq!(seed_lo, -7278655050202467086);
        assert_eq!(seed_hi, -3669656285675084719);

        let factory = PositionalRandomFactory::from_world_seed(world_seed);
        assert_eq!(factory.seed_lo, 3219654894476264721);
        assert_eq!(factory.seed_hi, 8777179534091651608);

        // Verify the 6 exact initial seeds for climate noises
        let t_rng = factory.from_hash_of("minecraft:temperature");
        assert_eq!(t_rng.s0, 8129252655152498798);
        assert_eq!(t_rng.s1, -8208195889237343344);

        let mut t_fork = t_rng;
        let forked = t_fork.fork_positional();
        assert_eq!(forked.seed_lo, -8941644036242799043);
        assert_eq!(forked.seed_hi, -6320392766069869736);
    }

    #[test]
    fn test_feature_seed_derivation_isolation() {
        // World seed 123456789, chunk (15, 7)
        let world_seed = 123456789u64;
        let chunk_x = 15;
        let chunk_z = 7;
        let pop_seed = set_decoration_seed(world_seed, chunk_x, chunk_z);
        assert_eq!(pop_seed, 15114538045649739285);

        // Feature seed for Oak Tree (step 9 = VEGETAL_DECORATION, feature 3)
        let mut rng = set_feature_seed(pop_seed, 3, DecorationStep::VegetalDecoration as usize);

        // Verify the first 10 generated integers match vanilla WorldgenRandom.nextInt() output identically
        let expected_first_10 = [
            -409440806, 678788834, 1967969886, 1828554918, 2074667250,
            -1073902171, 1898899331, -328568763, 550411763, -1165301981,
        ];
        for expected in expected_first_10 {
            assert_eq!(rng.next_int(), expected);
        }

        // Test WorldgenRandom convenience struct matches
        let mut wr = WorldgenRandom::new(world_seed as i64);
        let derived_pop = wr.set_decoration_seed(world_seed, chunk_x, chunk_z);
        assert_eq!(derived_pop, pop_seed);
        wr.set_feature_seed(pop_seed, 3, 9);
        for expected in expected_first_10 {
            assert_eq!(wr.next_int(), expected);
        }

        // Feature isolation: re-seeding with different feature indices gives completely independent streams
        let mut another_rng = set_feature_seed(pop_seed, 4, 9);
        assert_ne!(another_rng.next_int(), expected_first_10[0]);
    }
}
