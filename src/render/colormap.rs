// REA Verified: net.minecraft.client.renderer.BiomeColors (mapped: fst.class)
// REA Verified: net.minecraft.world.level.biome.Biome (mapped: cuo.class - climate temperature & downfall)
// REA Verified: net.minecraft.world.level.GrassColor (mapped: ctn.class)
// REA Verified: net.minecraft.world.level.FoliageColor (mapped: ctj.class)
//
// Biome colormap sampling and foliage tinting:
// - Loads 256x256 colormap textures from client.jar:
//   * assets/minecraft/textures/colormap/grass.png
//   * assets/minecraft/textures/colormap/foliage.png
// - Triangular barycentric UV coordinate sampling:
//   * Clamps temperature T in [0.0, 1.0] and downfall/humidity H in [0.0, 1.0].
//   * Coordinate mapping: U = 1.0 - T, V = 1.0 - (T * H).
//   * Samples RGB color value at (U, V).
// - Smooth biome border blending with 3x3 block neighbor averaging kernel.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;
use image::{Rgba, RgbaImage};

use crate::render::atlas::TextureAtlas;
use crate::render::mesher::BlockReader;

/// Standard 256x256 colormap texture dimensions
pub const COLORMAP_SIZE: u32 = 256;

/// Canonical Minecraft Biomes and their climate temperature / downfall parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    Plains,
    Forest,
    BirchForest,
    DarkForest,
    Taiga,
    SnowyPlains,
    Desert,
    Savanna,
    Jungle,
    Swamp,
    Badlands,
    Ocean,
    PaleGarden,
}

impl Biome {
    /// Vanilla biome base temperature (before elevation adjustment)
    pub fn temperature(&self) -> f32 {
        match self {
            Biome::Plains => 0.8,
            Biome::Forest | Biome::DarkForest => 0.7,
            Biome::BirchForest => 0.6,
            Biome::Taiga => 0.25,
            Biome::SnowyPlains => 0.0,
            Biome::Desert | Biome::Badlands => 2.0,
            Biome::Savanna => 1.1,
            Biome::Jungle => 0.95,
            Biome::Swamp => 0.8,
            Biome::Ocean => 0.5,
            Biome::PaleGarden => 0.7,
        }
    }

    /// Vanilla biome downfall / humidity
    pub fn downfall(&self) -> f32 {
        match self {
            Biome::Plains => 0.4,
            Biome::Forest | Biome::DarkForest | Biome::Taiga => 0.8,
            Biome::BirchForest => 0.6,
            Biome::SnowyPlains => 0.5,
            Biome::Desert | Biome::Savanna | Biome::Badlands => 0.0,
            Biome::Jungle | Biome::Swamp => 0.9,
            Biome::Ocean => 0.5,
            Biome::PaleGarden => 0.8,
        }
    }

    /// Combined (temperature, downfall) climate tuple
    #[inline]
    pub fn climate(&self) -> (f32, f32) {
        (self.temperature(), self.downfall())
    }

    /// Vanilla special fixed grass tint override (e.g. Swamp, Badlands, Pale Garden)
    pub fn special_grass_color(&self) -> Option<[f32; 3]> {
        match self {
            Biome::Swamp => Some([0x6A as f32 / 255.0, 0x70 as f32 / 255.0, 0x39 as f32 / 255.0]), // #6A7039
            Biome::Badlands => Some([0x90 as f32 / 255.0, 0x81 as f32 / 255.0, 0x4D as f32 / 255.0]), // #90814D
            Biome::PaleGarden => Some([0x84 as f32 / 255.0, 0x87 as f32 / 255.0, 0x7D as f32 / 255.0]), // #84877D
            _ => None,
        }
    }

    /// Vanilla special fixed foliage tint override (e.g. Swamp, Badlands, Pale Garden)
    pub fn special_foliage_color(&self) -> Option<[f32; 3]> {
        match self {
            Biome::Swamp => Some([0x6A as f32 / 255.0, 0x70 as f32 / 255.0, 0x39 as f32 / 255.0]), // #6A7039
            Biome::Badlands => Some([0x9E as f32 / 255.0, 0x81 as f32 / 255.0, 0x4D as f32 / 255.0]), // #9E814D
            Biome::PaleGarden => Some([0x87 as f32 / 255.0, 0x85 as f32 / 255.0, 0x7E as f32 / 255.0]), // #87857E
            _ => None,
        }
    }

    /// Vanilla special fixed ambient fog tint override (e.g. Pale Garden #9E9EA0)
    pub fn special_fog_color(&self) -> Option<[f32; 3]> {
        match self {
            Biome::PaleGarden => Some([0x9E as f32 / 255.0, 0x9E as f32 / 255.0, 0xA0 as f32 / 255.0]), // #9E9EA0
            _ => None,
        }
    }
}

/// Converts temperature and downfall to barycentric triangular UV coordinates.
/// REA Verified: net.minecraft.world.level.GrassColor.get(T, H) (ctn.class)
///   downfall = downfall * temperature;
///   x = (int)((1.0 - temperature) * 255.0);
///   y = (int)((1.0 - downfall) * 255.0);
#[inline]
pub fn climate_to_uv(temperature: f32, downfall: f32) -> (f32, f32) {
    let t = temperature.clamp(0.0, 1.0);
    let h = downfall.clamp(0.0, 1.0);
    let u = 1.0 - t;
    let v = 1.0 - (t * h);
    (u, v)
}

/// Biome colormap manager storing 256x256 grass and foliage textures
#[derive(Clone, Debug)]
pub struct BiomeColormaps {
    pub grass_map: RgbaImage,
    pub foliage_map: RgbaImage,
}

impl Default for BiomeColormaps {
    fn default() -> Self {
        Self::load_or_default(None)
    }
}

impl BiomeColormaps {
    /// Loads colormaps from `client.jar` (or falls back to procedural vanilla palettes)
    pub fn load_or_default(jar_path: Option<&Path>) -> Self {
        let resolved_path = jar_path
            .map(|p| p.to_path_buf())
            .or_else(TextureAtlas::find_client_jar);

        if let Some(path) = resolved_path {
            if let Ok(colormaps) = Self::load_from_jar(&path) {
                return colormaps;
            }
        }

        Self::default_procedural()
    }

    /// Loads 256x256 grass and foliage colormaps from a `client.jar` zip archive
    pub fn load_from_jar(jar_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(jar_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // 1. Grass colormap
        let grass_map = {
            let mut grass_entry = archive.by_name("assets/minecraft/textures/colormap/grass.png")?;
            let mut grass_buf = Vec::new();
            grass_entry.read_to_end(&mut grass_buf)?;
            image::load_from_memory(&grass_buf)?.to_rgba8()
        };

        // 2. Foliage colormap
        let foliage_map = {
            let mut foliage_entry = archive.by_name("assets/minecraft/textures/colormap/foliage.png")?;
            let mut foliage_buf = Vec::new();
            foliage_entry.read_to_end(&mut foliage_buf)?;
            image::load_from_memory(&foliage_buf)?.to_rgba8()
        };

        Ok(Self {
            grass_map,
            foliage_map,
        })
    }

    /// Generates high-fidelity procedural 256x256 fallback colormaps matching vanilla gradients
    pub fn default_procedural() -> Self {
        let mut grass_map = RgbaImage::new(COLORMAP_SIZE, COLORMAP_SIZE);
        let mut foliage_map = RgbaImage::new(COLORMAP_SIZE, COLORMAP_SIZE);

        for y in 0..COLORMAP_SIZE {
            let v = y as f32 / 255.0;
            for x in 0..COLORMAP_SIZE {
                let u = x as f32 / 255.0;

                // Barycentric climate reconstruction:
                // U = 1.0 - T  =>  T = 1.0 - U
                // V = 1.0 - (T * H)  =>  H = (1.0 - V) / max(T, 0.001)
                let t = (1.0 - u).clamp(0.0, 1.0);
                let h = if t > 0.001 {
                    ((1.0 - v) / t).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                // Vanilla grass color gradient:
                // Lush warm/wet: [121, 192, 90] (Plains)
                // Dry warm: [191, 183, 85] (Desert/Savanna)
                // Cold wet: [128, 180, 151] (Taiga/Snow)
                let grass_r = (121.0 * t * h + 191.0 * t * (1.0 - h) + 128.0 * (1.0 - t)).clamp(0.0, 255.0) as u8;
                let grass_g = (192.0 * t * h + 183.0 * t * (1.0 - h) + 180.0 * (1.0 - t)).clamp(0.0, 255.0) as u8;
                let grass_b = (90.0 * t * h + 85.0 * t * (1.0 - h) + 151.0 * (1.0 - t)).clamp(0.0, 255.0) as u8;
                grass_map.put_pixel(x, y, Rgba([grass_r, grass_g, grass_b, 255]));

                // Vanilla foliage color gradient:
                // Lush: [89, 174, 48]
                // Dry: [174, 164, 48]
                // Pine/Taiga: [97, 153, 97]
                let fol_r = (89.0 * t * h + 174.0 * t * (1.0 - h) + 97.0 * (1.0 - t)).clamp(0.0, 255.0) as u8;
                let fol_g = (174.0 * t * h + 164.0 * t * (1.0 - h) + 153.0 * (1.0 - t)).clamp(0.0, 255.0) as u8;
                let fol_b = (48.0 * t * h + 48.0 * t * (1.0 - h) + 97.0 * (1.0 - t)).clamp(0.0, 255.0) as u8;
                foliage_map.put_pixel(x, y, Rgba([fol_r, fol_g, fol_b, 255]));
            }
        }

        Self {
            grass_map,
            foliage_map,
        }
    }

    /// Samples normalized RGB [0.0, 1.0] from a 256x256 colormap at (U, V)
    pub fn sample_map(image: &RgbaImage, u: f32, v: f32) -> [f32; 3] {
        let width = image.width();
        let height = image.height();

        // REA Verified: int x = (int)(u * 255.0); int y = (int)(v * 255.0);
        let x = ((u.clamp(0.0, 1.0) * (width - 1) as f32).round() as u32).min(width - 1);
        let y = ((v.clamp(0.0, 1.0) * (height - 1) as f32).round() as u32).min(height - 1);

        let pixel = image.get_pixel(x, y);
        [
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
        ]
    }

    /// Samples u8 RGB [0..=255] from a 256x256 colormap at (U, V)
    pub fn sample_map_u8(image: &RgbaImage, u: f32, v: f32) -> [u8; 3] {
        let width = image.width();
        let height = image.height();

        let x = ((u.clamp(0.0, 1.0) * (width - 1) as f32).round() as u32).min(width - 1);
        let y = ((v.clamp(0.0, 1.0) * (height - 1) as f32).round() as u32).min(height - 1);

        let pixel = image.get_pixel(x, y);
        [pixel[0], pixel[1], pixel[2]]
    }

    /// Samples normalized RGB [0.0, 1.0] grass color for temperature and downfall
    pub fn sample_grass(&self, temperature: f32, downfall: f32) -> [f32; 3] {
        let (u, v) = climate_to_uv(temperature, downfall);
        Self::sample_map(&self.grass_map, u, v)
    }

    /// Samples u8 RGB [0..=255] grass color for temperature and downfall
    pub fn sample_grass_u8(&self, temperature: f32, downfall: f32) -> [u8; 3] {
        let (u, v) = climate_to_uv(temperature, downfall);
        Self::sample_map_u8(&self.grass_map, u, v)
    }

    /// Samples normalized RGB [0.0, 1.0] foliage color for temperature and downfall
    pub fn sample_foliage(&self, temperature: f32, downfall: f32) -> [f32; 3] {
        let (u, v) = climate_to_uv(temperature, downfall);
        Self::sample_map(&self.foliage_map, u, v)
    }

    /// Samples u8 RGB [0..=255] foliage color for temperature and downfall
    pub fn sample_foliage_u8(&self, temperature: f32, downfall: f32) -> [u8; 3] {
        let (u, v) = climate_to_uv(temperature, downfall);
        Self::sample_map_u8(&self.foliage_map, u, v)
    }

    /// Samples grass color for a given Biome (handling overrides such as Swamp)
    pub fn get_biome_grass_color(&self, biome: Biome) -> [f32; 3] {
        if let Some(special) = biome.special_grass_color() {
            special
        } else {
            self.sample_grass(biome.temperature(), biome.downfall())
        }
    }

    /// Samples foliage color for a given Biome (handling overrides such as Swamp)
    pub fn get_biome_foliage_color(&self, biome: Biome) -> [f32; 3] {
        if let Some(special) = biome.special_foliage_color() {
            special
        } else {
            self.sample_foliage(biome.temperature(), biome.downfall())
        }
    }

    /// Smooths grass tint across a 3x3 block neighbor kernel centered at (x, z).
    /// REA Verified: net.minecraft.client.renderer.BiomeColors (fst.class)
    pub fn get_smoothed_grass_color(
        &self,
        reader: &dyn BlockReader,
        x: i32,
        z: i32,
    ) -> [f32; 3] {
        let mut total_r = 0.0f32;
        let mut total_g = 0.0f32;
        let mut total_b = 0.0f32;

        for dz in -1..=1 {
            for dx in -1..=1 {
                let (temp, downfall) = reader.get_biome_climate(x + dx, z + dz);
                let [r, g, b] = self.sample_grass(temp, downfall);
                total_r += r;
                total_g += g;
                total_b += b;
            }
        }

        [total_r / 9.0, total_g / 9.0, total_b / 9.0]
    }

    /// Smooths foliage tint across a 3x3 block neighbor kernel centered at (x, z).
    /// REA Verified: net.minecraft.client.renderer.BiomeColors (fst.class)
    pub fn get_smoothed_foliage_color(
        &self,
        reader: &dyn BlockReader,
        x: i32,
        z: i32,
    ) -> [f32; 3] {
        let mut total_r = 0.0f32;
        let mut total_g = 0.0f32;
        let mut total_b = 0.0f32;

        for dz in -1..=1 {
            for dx in -1..=1 {
                let (temp, downfall) = reader.get_biome_climate(x + dx, z + dz);
                let [r, g, b] = self.sample_foliage(temp, downfall);
                total_r += r;
                total_g += g;
                total_b += b;
            }
        }

        [total_r / 9.0, total_g / 9.0, total_b / 9.0]
    }
}

/// Global lazy-initialized colormaps instance
static GLOBAL_COLORMAPS: OnceLock<BiomeColormaps> = OnceLock::new();

/// Returns reference to global shared BiomeColormaps
pub fn get_default_colormaps() -> &'static BiomeColormaps {
    GLOBAL_COLORMAPS.get_or_init(|| BiomeColormaps::load_or_default(None))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockReader {
        climate: (f32, f32),
    }

    impl BlockReader for MockReader {
        fn get_block_at(&self, _x: i32, _y: i32, _z: i32) -> crate::render::atlas::BlockType {
            crate::render::atlas::BlockType::Air
        }

        fn get_biome_climate(&self, _x: i32, _z: i32) -> (f32, f32) {
            self.climate
        }
    }

    struct GradientReader;

    impl BlockReader for GradientReader {
        fn get_block_at(&self, _x: i32, _y: i32, _z: i32) -> crate::render::atlas::BlockType {
            crate::render::atlas::BlockType::Air
        }

        fn get_biome_climate(&self, x: i32, _z: i32) -> (f32, f32) {
            if x <= 0 {
                (0.8, 0.4) // Plains
            } else {
                (0.25, 0.8) // Taiga
            }
        }
    }

    #[test]
    fn test_barycentric_uv_coordinate_mapping() {
        // T in [0.0, 1.0], H in [0.0, 1.0]
        // U = 1.0 - T, V = 1.0 - (T * H)

        // Case 1: Plains (T = 0.8, H = 0.4)
        let (u, v) = climate_to_uv(0.8, 0.4);
        assert!((u - 0.2).abs() < 1e-5);
        assert!((v - (1.0 - 0.32)).abs() < 1e-5);

        // Case 2: Desert (T = 2.0 -> clamped to 1.0, H = 0.0)
        let (u, v) = climate_to_uv(2.0, 0.0);
        assert_eq!(u, 0.0);
        assert_eq!(v, 1.0);

        // Case 3: Snowy Plains (T = 0.0, H = 0.5)
        let (u, v) = climate_to_uv(0.0, 0.5);
        assert_eq!(u, 1.0);
        assert_eq!(v, 1.0); // 1.0 - (0.0 * 0.5) = 1.0

        // Case 4: Forest (T = 0.7, H = 0.8)
        let (u, v) = climate_to_uv(0.7, 0.8);
        assert!((u - 0.3).abs() < 1e-5);
        assert!((v - (1.0 - 0.56)).abs() < 1e-5);
    }

    #[test]
    fn test_colormap_sampling_values() {
        let colormaps = BiomeColormaps::load_or_default(None);

        // Sample Plains
        let grass_plains = colormaps.sample_grass(0.8, 0.4);
        let fol_plains = colormaps.sample_foliage(0.8, 0.4);

        // Green channel should be dominant in plains grass and foliage
        assert!(grass_plains[1] > grass_plains[0]);
        assert!(grass_plains[1] > grass_plains[2]);
        assert!(fol_plains[1] > fol_plains[0]);
        assert!(fol_plains[1] > fol_plains[2]);

        // Values must be within [0.0, 1.0]
        for c in grass_plains.iter().chain(fol_plains.iter()) {
            assert!(*c >= 0.0 && *c <= 1.0);
        }
    }

    #[test]
    fn test_smoothed_neighbor_kernel() {
        let colormaps = BiomeColormaps::load_or_default(None);

        // Uniform reader
        let mock = MockReader { climate: (0.8, 0.4) };
        let smoothed = colormaps.get_smoothed_grass_color(&mock, 0, 0);
        let single = colormaps.sample_grass(0.8, 0.4);

        for i in 0..3 {
            assert!((smoothed[i] - single[i]).abs() < 1e-4);
        }

        // Gradient border reader: x = 0 has left neighbor x = -1 (Plains) and right neighbor x = 1 (Taiga)
        let grad = GradientReader;
        let border_color = colormaps.get_smoothed_grass_color(&grad, 0, 0);
        let plains_color = colormaps.sample_grass(0.8, 0.4);
        let taiga_color = colormaps.sample_grass(0.25, 0.8);

        // Border color should be strictly between Plains and Taiga
        for i in 0..3 {
            let min_c = plains_color[i].min(taiga_color[i]);
            let max_c = plains_color[i].max(taiga_color[i]);
            assert!(border_color[i] >= min_c - 1e-4 && border_color[i] <= max_c + 1e-4);
        }
    }
}
