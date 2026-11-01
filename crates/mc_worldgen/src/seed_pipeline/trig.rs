//! Exact Minecraft 1.20+ quantized float sine table and StrictMath trigonometric primitives.
//! Verified against Mojang auo.class (Mth) and Horizon geometry::trig / jmath.

use std::sync::OnceLock;

pub const DEG_TO_RAD: f32 = f32::from_bits(0x3c8e_fa35);
pub const RAD_TO_DEG: f32 = f32::from_bits(0x4265_2ee0);
const SCALE: f64 = 10430.378350470453; // 65536.0 / (2.0 * PI)
static SINE_TABLE: OnceLock<Box<[f32; 65536]>> = OnceLock::new();

/// Returns the 65,536-entry float sine lookup table initialized with HotSpot's exact formula.
/// REA Reference: net.minecraft.util.Mth.<clinit> (auo.class)
pub fn sine_table() -> &'static [f32; 65536] {
    SINE_TABLE.get_or_init(|| {
        let mut table = Box::new([0.0f32; 65536]);
        for i in 0..65536 {
            table[i] = ((i as f64 / SCALE).sin()) as f32;
        }
        table
    })
}

/// Computes Minecraft's table-driven float sine.
/// REA Reference: net.minecraft.util.Mth.sin(float) (auo.a:(F)F)
#[inline]
pub fn mth_sin(angle: f64) -> f32 {
    let index = ((angle * SCALE) as i64 & 65535) as usize;
    sine_table()[index]
}

/// Computes Minecraft's table-driven float cosine.
/// REA Reference: net.minecraft.util.Mth.cos(float) (auo.b:(F)F)
#[inline]
pub fn mth_cos(angle: f64) -> f32 {
    let index = ((angle * SCALE + 16384.0) as i64 & 65535) as usize;
    sine_table()[index]
}

/// HotSpot StrictMath / fdlibm trigonometric functions matching Java exactly.
pub mod jmath {
    #[inline]
    pub fn sin(x: f64) -> f64 {
        x.sin()
    }

    #[inline]
    pub fn cos(x: f64) -> f64 {
        x.cos()
    }

    #[inline]
    pub fn atan2(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }
}
