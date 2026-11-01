//! Exact cubic Hermite spline interpolation matching Minecraft 1.20+ CubicSpline.
//! REA Reference: net.minecraft.util.CubicSpline (atq.class) and Multipoint (atq$e.class)
//! Verified against Horizon horizon-core::density::spline.

use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct SplinePoint<C = usize> {
    pub location: f32,
    pub value: CubicSpline<C>,
    pub derivative: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultipointSpline<C = usize> {
    pub coordinate: C,
    pub points: Vec<SplinePoint<C>>,
}

/// An immutable cubic spline tree. Coordinate values are sampled via caller-supplied function.
#[derive(Clone, Debug, PartialEq)]
pub enum CubicSpline<C = usize> {
    Constant(f32),
    Multipoint(Arc<MultipointSpline<C>>),
}

impl<C> CubicSpline<C> {
    pub fn constant(val: f32) -> Self {
        CubicSpline::Constant(val)
    }

    pub fn multipoint(coordinate: C, points: Vec<SplinePoint<C>>) -> Self {
        CubicSpline::Multipoint(Arc::new(MultipointSpline {
            coordinate,
            points,
        }))
    }

    /// Evaluates the cubic Hermite spline with exact binary search and Hermite interpolation.
    /// REA Reference: atq$e.sample (lines 75-107)
    pub fn sample<F>(&self, coordinate: &mut F) -> f32
    where
        F: FnMut(&C) -> f32,
    {
        let node = match self {
            Self::Constant(value) => return *value,
            Self::Multipoint(node) => node,
        };

        let input = coordinate(&node.coordinate);

        // Vanilla binarySearch finds the first strictly greater location
        let mut first = 0;
        let mut remaining = node.points.len();
        while remaining != 0 {
            let half = remaining / 2;
            let middle = first + half;
            if input < node.points[middle].location {
                remaining = half;
            } else {
                first = middle + 1;
                remaining -= half + 1;
            }
        }

        if first == 0 || first == node.points.len() {
            let endpoint = if first == 0 { 0 } else { first - 1 };
            let point = &node.points[endpoint];
            let value = point.value.sample(coordinate);
            return if point.derivative == 0.0 {
                value
            } else {
                value + point.derivative * (input - point.location)
            };
        }

        let left = &node.points[first - 1];
        let right = &node.points[first];
        let width = right.location - left.location;
        let alpha = (input - left.location) / width;
        let left_value = left.value.sample(coordinate);
        let right_value = right.value.sample(coordinate);

        let left_correction = left.derivative * width - (right_value - left_value);
        let right_correction = -right.derivative * width + (right_value - left_value);
        let linear = left_value + alpha * (right_value - left_value);
        let correction = left_correction + alpha * (right_correction - left_correction);

        linear + (alpha * (1.0 - alpha)) * correction
    }
}

impl CubicSpline<String> {
    pub fn from_json_value(val: &serde_json::Value) -> Option<Self> {
        if let Some(num) = val.as_f64() {
            return Some(CubicSpline::Constant(num as f32));
        }
        let coord = val.get("coordinate")?.as_str()?.to_string();
        let points_arr = val.get("points")?.as_array()?;
        let mut points = Vec::with_capacity(points_arr.len());
        for p in points_arr {
            let location = p.get("location")?.as_f64()? as f32;
            let derivative = p.get("derivative")?.as_f64()? as f32;
            let value = Self::from_json_value(p.get("value")?)?;
            points.push(SplinePoint {
                location,
                value,
                derivative,
            });
        }
        Some(CubicSpline::Multipoint(Arc::new(MultipointSpline {
            coordinate: coord,
            points,
        })))
    }

    pub fn from_json_str(s: &str) -> Option<Self> {
        let val: serde_json::Value = serde_json::from_str(s).ok()?;
        Self::from_json_value(&val)
    }
}

pub static TERRAIN_OFFSET_SPLINE: std::sync::LazyLock<CubicSpline<String>> = std::sync::LazyLock::new(|| {
    let json_str = include_str!("offset_spline.json");
    CubicSpline::from_json_str(json_str).expect("Valid offset spline JSON")
});

/// Canonical folding function: NoiseRouterData.peaksAndValleys / ridges_folded
#[inline]
pub fn ridges_folded(w: f32) -> f32 {
    -3.0 * (-0.33333334 + (-0.6666667 + w.abs()).abs())
}

/// Evaluates terrain offset from canonical continentalness, erosion, and weirdness/ridge inputs.
pub fn evaluate_terrain_offset(continents: f32, erosion: f32, weirdness: f32) -> f32 {
    let folded = ridges_folded(weirdness);
    let spline_val = TERRAIN_OFFSET_SPLINE.sample(&mut |coord: &String| {
        match coord.as_str() {
            "minecraft:overworld/continents" => continents,
            "minecraft:overworld/erosion" => erosion,
            "minecraft:overworld/ridges_folded" => folded,
            _ => 0.0,
        }
    });
    -0.503750026 + spline_val
}

pub static TERRAIN_FACTOR_SPLINE: std::sync::LazyLock<CubicSpline<String>> = std::sync::LazyLock::new(|| {
    let json_str = include_str!("factor_spline.json");
    CubicSpline::from_json_str(json_str).expect("Valid factor spline JSON")
});

/// Evaluates terrain factor from canonical continentalness, erosion, ridges, and weirdness inputs.
pub fn evaluate_terrain_factor(continents: f32, erosion: f32, ridges: f32, weirdness: f32) -> f32 {
    let folded = ridges_folded(weirdness);
    TERRAIN_FACTOR_SPLINE.sample(&mut |coord: &String| {
        match coord.as_str() {
            "minecraft:overworld/continents" => continents,
            "minecraft:overworld/erosion" => erosion,
            "minecraft:overworld/ridges" => ridges,
            "minecraft:overworld/ridges_folded" => folded,
            _ => 0.0,
        }
    })
}

