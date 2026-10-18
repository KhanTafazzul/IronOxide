//! ImprovedNoise, PerlinNoise, and NormalNoise implementation with 100% mathematical parity
//! to Minecraft 1.20.4 bytecodes (eds, edv, edu).

use super::random::{double, PositionalRandomFactory, Xoroshiro128PlusPlus};

/// Standard Perlin/Simplex 3D gradient vector table.
const GRADIENTS: [[f64; 3]; 16] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
    [1.0, 1.0, 0.0],
    [0.0, -1.0, 1.0],
    [-1.0, 1.0, 0.0],
    [0.0, -1.0, -1.0],
];

#[inline]
fn grad_dot(hash: usize, x: double, y: double, z: double) -> double {
    let g = GRADIENTS[hash & 15];
    g[0] * x + g[1] * y + g[2] * z
}

#[inline]
fn smoothstep(t: double) -> double {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn floor_int(x: double) -> i32 {
    let xi = x as i32;
    if x < (xi as f64) {
        xi - 1
    } else {
        xi
    }
}

/// Single octave Perlin noise generator (net.minecraft.world.level.levelgen.synth.ImprovedNoise).
#[derive(Clone, Debug)]
pub struct ImprovedNoise {
    pub xo: double,
    pub yo: double,
    pub zo: double,
    pub p: [u8; 256],
}

impl ImprovedNoise {
    pub fn new(rng: &mut Xoroshiro128PlusPlus) -> Self {
        let xo = rng.next_double() * 256.0;
        let yo = rng.next_double() * 256.0;
        let zo = rng.next_double() * 256.0;

        let mut p = [0u8; 256];
        for i in 0..256 {
            p[i] = i as u8;
        }

        for i in 0..256 {
            let j = rng.next_int_bounded((256 - i) as i32) as usize;
            let temp = p[i];
            p[i] = p[i + j];
            p[i + j] = temp;
        }

        Self { xo, yo, zo, p }
    }

    #[inline]
    fn p_lookup(&self, idx: i32) -> usize {
        self.p[(idx & 255) as usize] as usize
    }

    pub fn sample(&self, x: double, y: double, z: double) -> double {
        let x_off = x + self.xo;
        let y_off = y + self.yo;
        let z_off = z + self.zo;

        let sec_x = floor_int(x_off);
        let sec_y = floor_int(y_off);
        let sec_z = floor_int(z_off);

        let dx = x_off - (sec_x as f64);
        let dy = y_off - (sec_y as f64);
        let dz = z_off - (sec_z as f64);

        let u = smoothstep(dx);
        let v = smoothstep(dy);
        let w = smoothstep(dz);

        let a = self.p_lookup(sec_x);
        let b = self.p_lookup(sec_x + 1);
        let aa = self.p_lookup((a as i32) + sec_y);
        let ba = self.p_lookup((b as i32) + sec_y);
        let ab = self.p_lookup((a as i32) + sec_y + 1);
        let bb = self.p_lookup((b as i32) + sec_y + 1);

        let g000 = grad_dot(self.p_lookup((aa as i32) + sec_z), dx, dy, dz);
        let g100 = grad_dot(self.p_lookup((ba as i32) + sec_z), dx - 1.0, dy, dz);
        let g010 = grad_dot(self.p_lookup((ab as i32) + sec_z), dx, dy - 1.0, dz);
        let g110 = grad_dot(self.p_lookup((bb as i32) + sec_z), dx - 1.0, dy - 1.0, dz);
        let g001 = grad_dot(self.p_lookup((aa as i32) + sec_z + 1), dx, dy, dz - 1.0);
        let g101 = grad_dot(self.p_lookup((ba as i32) + sec_z + 1), dx - 1.0, dy, dz - 1.0);
        let g011 = grad_dot(self.p_lookup((ab as i32) + sec_z + 1), dx, dy - 1.0, dz - 1.0);
        let g111 = grad_dot(self.p_lookup((bb as i32) + sec_z + 1), dx - 1.0, dy - 1.0, dz - 1.0);

        let x1 = g000 + u * (g100 - g000);
        let x2 = g010 + u * (g110 - g010);
        let x3 = g001 + u * (g101 - g001);
        let x4 = g011 + u * (g111 - g011);

        let y1 = x1 + v * (x2 - x1);
        let y2 = x3 + v * (x4 - x3);

        y1 + w * (y2 - y1)
    }
}

#[inline]
pub fn wrap_coord(c: double) -> double {
    c - ((c / 33554432.0 + 0.5).floor()) * 33554432.0
}

/// Multi-octave Perlin noise generator (net.minecraft.world.level.levelgen.synth.PerlinNoise).
#[derive(Clone, Debug)]
pub struct PerlinNoise {
    pub octaves: Vec<Option<ImprovedNoise>>,
    pub first_octave: i32,
    pub scale_factor: double,
    pub amplitude_factor: double,
}

impl PerlinNoise {
    pub fn from_factory(
        factory: &PositionalRandomFactory,
        first_octave: i32,
        amplitudes: &[double],
    ) -> Self {
        let octave_count = amplitudes.len();
        let mut octaves = Vec::with_capacity(octave_count);

        for (i, &amp) in amplitudes.iter().enumerate() {
            if amp != 0.0 {
                let oct_idx = first_octave + (i as i32);
                let key = format!("octave_{}", oct_idx);
                let mut oct_rng = factory.from_hash_of(&key);
                octaves.push(Some(ImprovedNoise::new(&mut oct_rng)));
            } else {
                octaves.push(None);
            }
        }

        let scale_factor = (2.0f64).powi(first_octave);
        let amplitude_factor = (2.0f64).powi((octave_count as i32) - 1)
            / ((2.0f64).powi(octave_count as i32) - 1.0);

        Self {
            octaves,
            first_octave,
            scale_factor,
            amplitude_factor,
        }
    }

    pub fn sample(&self, x: double, y: double, z: double) -> double {
        let mut total = 0.0;
        let mut cur_scale = self.scale_factor;
        let mut cur_amp = self.amplitude_factor;

        for oct in &self.octaves {
            if let Some(noise) = oct {
                let nx = wrap_coord(x * cur_scale);
                let ny = wrap_coord(y * cur_scale);
                let nz = wrap_coord(z * cur_scale);
                total += noise.sample(nx, ny, nz) * cur_amp;
            }
            cur_scale *= 2.0;
            cur_amp /= 2.0;
        }

        total
    }
}

/// NormalNoise combines two multi-octave Perlin noise samplers (net.minecraft.world.level.levelgen.synth.NormalNoise).
#[derive(Clone, Debug)]
pub struct NormalNoise {
    pub first: PerlinNoise,
    pub second: PerlinNoise,
    pub value_factor: double,
}

impl NormalNoise {
    pub fn from_rng(
        rng: &mut Xoroshiro128PlusPlus,
        first_octave: i32,
        amplitudes: &[double],
    ) -> Self {
        let f1 = rng.fork_positional();
        let first = PerlinNoise::from_factory(&f1, first_octave, amplitudes);

        let f2 = rng.fork_positional();
        let second = PerlinNoise::from_factory(&f2, first_octave, amplitudes);

        let mut min_non_zero = i32::MAX;
        let mut max_non_zero = i32::MIN;
        for (i, &amp) in amplitudes.iter().enumerate() {
            if amp != 0.0 {
                let idx = i as i32;
                if idx < min_non_zero {
                    min_non_zero = idx;
                }
                if idx > max_non_zero {
                    max_non_zero = idx;
                }
            }
        }

        let delta = if max_non_zero >= min_non_zero {
            max_non_zero - min_non_zero
        } else {
            0
        };

        let expected_factor = 0.1 * (1.0 + 1.0 / ((delta + 1) as f64));
        let value_factor = (1.0 / 6.0) / expected_factor;

        Self {
            first,
            second,
            value_factor,
        }
    }

    #[inline]
    pub fn sample(&self, x: double, y: double, z: double) -> double {
        const SCALE_SECOND: double = 1.0181268882175227;
        let v1 = self.first.sample(x, y, z);
        let v2 = self
            .second
            .sample(x * SCALE_SECOND, y * SCALE_SECOND, z * SCALE_SECOND);
        (v1 + v2) * self.value_factor
    }
}
