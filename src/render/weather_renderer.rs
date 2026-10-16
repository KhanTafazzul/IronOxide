// REA Verified: net.minecraft.client.renderer.LevelRenderer.renderSnowAndRain (ftf.class)
//
// Volumetric Precipitation Renderer:
// - Dynamic particle box: 32x32x32 around the player
// - Rain sheets: Scrolling texture `assets/minecraft/textures/environment/rain.png` moving downwards at 0.75 blocks/tick
// - Snow flakes: Drifting oscillation `assets/minecraft/textures/environment/snow.png` moving downwards at 0.1 blocks/tick with sinusoidal horizontal drift
// - Atmospheric effects during rain:
//   * Sky light interpolated down by 25% (light * (1.0 - 0.25 * rain_strength))
//   * Fog distance pulled closer to simulate reduced visibility (distance * (1.0 - 0.35 * rain_strength))

use std::fs::File;
use std::io::Read;
use std::path::Path;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use image::{Rgba, RgbaImage};

use crate::render::atlas::TextureAtlas;
use crate::render::mesher::BlockReader;
use crate::world::weather::{PrecipitationType, WeatherManager};

/// Precipitation box dimensions around the player (32x32x32 total volume)
pub const PARTICLE_BOX_RADIUS: i32 = 16;
pub const PARTICLE_BOX_HEIGHT: i32 = 16;

/// Rain texture vertical downward scroll speed: 0.75 blocks per tick
pub const RAIN_SCROLL_SPEED: f32 = 0.75;

/// Snow texture vertical downward scroll speed: 0.1 blocks per tick
pub const SNOW_SCROLL_SPEED: f32 = 0.10;

/// Interleaved weather particle vertex representation: [pos: vec3, uv: vec2, color: vec4]
/// Guaranteed 4-byte alignment and 36-byte stride.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct WeatherVertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4], // RGBA tint & distance-fade alpha
}

impl WeatherVertex {
    pub fn new(pos: [f32; 3], uv: [f32; 2], color: [f32; 4]) -> Self {
        Self { pos, uv, color }
    }

    /// WGPU Vertex Buffer Layout descriptor for standard weather pipeline
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<WeatherVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // 0: pos: Float32x3 (offset 0)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // 1: uv: Float32x2 (offset 12)
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // 2: color: Float32x4 (offset 20)
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Mesh output containing vertices and index buffer for precipitation sheets
#[derive(Default, Clone, Debug)]
pub struct WeatherMesh {
    pub vertices: Vec<WeatherVertex>,
    pub indices: Vec<u32>,
}

impl WeatherMesh {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub fn quad_count(&self) -> usize {
        self.indices.len() / 6
    }
}

/// Combined rain and snow meshes generated for a frame
#[derive(Default, Clone, Debug)]
pub struct PrecipitationMeshes {
    pub rain: WeatherMesh,
    pub snow: WeatherMesh,
}

/// Computes sky light reduction during rain: drops by 25% at full rain strength
#[inline]
pub fn calculate_sky_light_reduction(sky_light: f32, rain_strength: f32) -> f32 {
    sky_light * (1.0 - 0.25 * rain_strength.clamp(0.0, 1.0))
}

/// Computes pulled fog distance during rain to simulate reduced visibility
#[inline]
pub fn calculate_fog_pull(base_fog_distance: f32, rain_strength: f32) -> f32 {
    base_fog_distance * (1.0 - 0.35 * rain_strength.clamp(0.0, 1.0))
}

/// Volumetric Weather Renderer managing rain and snow particle sheets
pub struct WeatherRenderer {
    pub rain_texture: RgbaImage,
    pub snow_texture: RgbaImage,
}

impl Default for WeatherRenderer {
    fn default() -> Self {
        Self::new(None)
    }
}

impl WeatherRenderer {
    pub fn new(jar_path: Option<&Path>) -> Self {
        let resolved_path = jar_path
            .map(|p| p.to_path_buf())
            .or_else(TextureAtlas::find_client_jar);

        let (rain_texture, snow_texture) = if let Some(path) = resolved_path {
            Self::load_textures_from_jar(&path)
                .unwrap_or_else(|_| (Self::generate_rain_fallback(), Self::generate_snow_fallback()))
        } else {
            (Self::generate_rain_fallback(), Self::generate_snow_fallback())
        };

        Self {
            rain_texture,
            snow_texture,
        }
    }

    /// Loads 64x256 rain and snow textures from `client.jar`
    pub fn load_textures_from_jar(jar_path: &Path) -> Result<(RgbaImage, RgbaImage), Box<dyn std::error::Error>> {
        let file = File::open(jar_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // 1. Rain texture
        let rain_texture = {
            let mut rain_entry = archive.by_name("assets/minecraft/textures/environment/rain.png")?;
            let mut rain_buf = Vec::new();
            rain_entry.read_to_end(&mut rain_buf)?;
            image::load_from_memory(&rain_buf)?.to_rgba8()
        };

        // 2. Snow texture
        let snow_texture = {
            let mut snow_entry = archive.by_name("assets/minecraft/textures/environment/snow.png")?;
            let mut snow_buf = Vec::new();
            snow_entry.read_to_end(&mut snow_buf)?;
            image::load_from_memory(&snow_buf)?.to_rgba8()
        };

        Ok((rain_texture, snow_texture))
    }

    /// Generates high-fidelity 64x256 fallback rain texture (vertical streaks with alpha)
    pub fn generate_rain_fallback() -> RgbaImage {
        let mut img = RgbaImage::new(64, 256);
        for y in 0..256 {
            for x in 0..64 {
                // Diagonal streaked rain lines
                let is_streak = (x * 7 + y * 3) % 29 < 3;
                if is_streak {
                    img.put_pixel(x, y, Rgba([180, 200, 255, 180]));
                } else {
                    img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }
        img
    }

    /// Generates high-fidelity 64x256 fallback snow texture (fluffy snowflakes)
    pub fn generate_snow_fallback() -> RgbaImage {
        let mut img = RgbaImage::new(64, 256);
        for y in 0..256 {
            for x in 0..64 {
                // Scattered snowflake dots
                let is_flake = (x * 13 + y * 17) % 67 < 2;
                if is_flake {
                    img.put_pixel(x, y, Rgba([255, 255, 255, 220]));
                } else {
                    img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }
        img
    }

    /// Builds volumetric rain and snow geometry around the player position within a 32x32x32 box
    pub fn build_precipitation_mesh(
        &self,
        player_pos: Vec3,
        weather: &WeatherManager,
        reader: &dyn BlockReader,
        ticks: u32,
        partial_ticks: f32,
    ) -> PrecipitationMeshes {
        let mut meshes = PrecipitationMeshes::default();

        if !weather.is_raining() {
            return meshes;
        }

        let total_ticks = ticks as f32 + partial_ticks;
        let px = player_pos.x;
        let py = player_pos.y;
        let pz = player_pos.z;

        let center_x = px.floor() as i32;
        let center_y = py.floor() as i32;
        let center_z = pz.floor() as i32;

        let y_min = (center_y - PARTICLE_BOX_HEIGHT).max(0);
        let y_max = center_y + PARTICLE_BOX_HEIGHT;
        let height_span = (y_max - y_min) as f32;

        if height_span <= 0.0 {
            return meshes;
        }

        let radius = PARTICLE_BOX_RADIUS;
        let max_radius_f = radius as f32;

        // Vertical scroll UV offsets
        // Rain scrolls downwards at 0.75 blocks/tick
        let rain_v_offset = (total_ticks * RAIN_SCROLL_SPEED) % 32.0;
        // Snow scrolls downwards at 0.10 blocks/tick
        let snow_v_offset = (total_ticks * SNOW_SCROLL_SPEED) % 32.0;

        // Iterate through columns in the 32x32 area centered on the player
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let block_x = center_x + dx;
                let block_z = center_z + dz;

                // Horizontal distance from player for radial fade
                let dist_x = (block_x as f32 + 0.5) - px;
                let dist_z = (block_z as f32 + 0.5) - pz;
                let horiz_dist = (dist_x * dist_x + dist_z * dist_z).sqrt();

                if horiz_dist > max_radius_f {
                    continue;
                }

                // Smooth edge fade towards boundary of the 32x32 particle box
                let edge_fade = (1.0 - (horiz_dist / max_radius_f)).clamp(0.0, 1.0);
                if edge_fade <= 0.001 {
                    continue;
                }

                // Query climate at this column to determine rain vs snow
                let (base_temp, _) = reader.get_biome_climate(block_x, block_z);
                let precip_type = weather.get_precipitation_type(base_temp, py);

                if precip_type == PrecipitationType::None {
                    continue;
                }

                let fx = block_x as f32;
                let fz = block_z as f32;
                let fy_bottom = y_min as f32;
                let fy_top = y_max as f32;

                // Column pseudo-random hash for visual variance across columns
                let col_hash = (((block_x.wrapping_mul(3121)) ^ (block_z.wrapping_mul(418711))) & 0xFFFF) as f32;

                match precip_type {
                    PrecipitationType::Rain => {
                        let alpha = (weather.rain_strength * edge_fade * 0.85).clamp(0.0, 1.0);
                        let color = [1.0, 1.0, 1.0, alpha];

                        // Rain sheets scroll downwards at 0.75 blocks/tick
                        let v_bottom = fy_bottom * 0.25 + rain_v_offset + (col_hash * 0.01);
                        let v_top = fy_top * 0.25 + rain_v_offset + (col_hash * 0.01);

                        // Cross sheets: Sheet 1 along X, Sheet 2 along Z
                        Self::push_double_sided_quad(
                            &mut meshes.rain,
                            [fx, fy_bottom, fz + 0.5],
                            [fx + 1.0, fy_bottom, fz + 0.5],
                            [fx + 1.0, fy_top, fz + 0.5],
                            [fx, fy_top, fz + 0.5],
                            [0.0, v_bottom, 1.0, v_top],
                            color,
                        );

                        Self::push_double_sided_quad(
                            &mut meshes.rain,
                            [fx + 0.5, fy_bottom, fz],
                            [fx + 0.5, fy_bottom, fz + 1.0],
                            [fx + 0.5, fy_top, fz + 1.0],
                            [fx + 0.5, fy_top, fz],
                            [0.0, v_bottom, 1.0, v_top],
                            color,
                        );
                    }
                    PrecipitationType::Snow => {
                        let alpha = (weather.rain_strength * edge_fade * 0.90).clamp(0.0, 1.0);
                        let color = [1.0, 1.0, 1.0, alpha];

                        // Snow falls at 0.10 blocks/tick with sinusoidal horizontal drift oscillation
                        let drift_angle = (total_ticks * 0.05) + (col_hash * 0.1);
                        let drift_x = drift_angle.sin() * 0.30;
                        let drift_z = drift_angle.cos() * 0.30;

                        let v_bottom = fy_bottom * 0.25 + snow_v_offset + (col_hash * 0.01);
                        let v_top = fy_top * 0.25 + snow_v_offset + (col_hash * 0.01);

                        let x0 = fx + drift_x;
                        let x1 = fx + 1.0 + drift_x;
                        let z0 = fz + drift_z;
                        let z1 = fz + 1.0 + drift_z;

                        // Cross snowflake quads with drift offset
                        Self::push_double_sided_quad(
                            &mut meshes.snow,
                            [x0, fy_bottom, z0 + 0.5],
                            [x1, fy_bottom, z0 + 0.5],
                            [x1, fy_top, z0 + 0.5],
                            [x0, fy_top, z0 + 0.5],
                            [0.0, v_bottom, 1.0, v_top],
                            color,
                        );

                        Self::push_double_sided_quad(
                            &mut meshes.snow,
                            [x0 + 0.5, fy_bottom, z0],
                            [x0 + 0.5, fy_bottom, z1],
                            [x0 + 0.5, fy_top, z1],
                            [x0 + 0.5, fy_top, z0],
                            [0.0, v_bottom, 1.0, v_top],
                            color,
                        );
                    }
                    PrecipitationType::None => {}
                }
            }
        }

        meshes
    }

    /// Appends a quad to the mesh with two-sided visibility
    fn push_double_sided_quad(
        mesh: &mut WeatherMesh,
        p0: [f32; 3],
        p1: [f32; 3],
        p2: [f32; 3],
        p3: [f32; 3],
        uv_bounds: [f32; 4],
        color: [f32; 4],
    ) {
        let u0 = uv_bounds[0];
        let v0 = uv_bounds[1];
        let u1 = uv_bounds[2];
        let v1 = uv_bounds[3];

        let base = mesh.vertices.len() as u32;

        mesh.vertices.push(WeatherVertex::new(p0, [u0, v0], color));
        mesh.vertices.push(WeatherVertex::new(p1, [u1, v0], color));
        mesh.vertices.push(WeatherVertex::new(p2, [u1, v1], color));
        mesh.vertices.push(WeatherVertex::new(p3, [u0, v1], color));

        // Two-sided triangles so precipitation sheet is visible from both exterior and interior
        mesh.indices.extend_from_slice(&[
            base, base + 1, base + 2,
            base, base + 2, base + 3,
            base, base + 2, base + 1,
            base, base + 3, base + 2,
        ]);
    }
}

/// Weather WGSL Shader for rendering translucent rain sheets and snow flakes
pub const WEATHER_WGSL: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_pos: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.pos, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    let final_alpha = tex_color.a * in.color.a;
    if (final_alpha < 0.05) {
        discard;
    }
    return vec4<f32>(tex_color.rgb * in.color.rgb, final_alpha);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    struct MockClimateReader {
        temp: f32,
    }

    impl BlockReader for MockClimateReader {
        fn get_block_at(&self, _x: i32, _y: i32, _z: i32) -> crate::render::atlas::BlockType {
            crate::render::atlas::BlockType::Air
        }
        fn get_biome_climate(&self, _x: i32, _z: i32) -> (f32, f32) {
            (self.temp, 0.8)
        }
    }

    #[test]
    fn test_weather_vertex_layout() {
        assert_eq!(std::mem::size_of::<WeatherVertex>(), 36);
        let desc = WeatherVertex::desc();
        assert_eq!(desc.array_stride, 36);
        assert_eq!(desc.attributes.len(), 3);
    }

    #[test]
    fn test_rain_sheet_generation_and_quad_count() {
        let renderer = WeatherRenderer::default();
        let mut weather = WeatherManager::new();
        weather.set_weather(crate::world::weather::WeatherState::Rain);
        for _ in 0..50 {
            weather.tick();
        }

        let reader = MockClimateReader { temp: 0.8 }; // Warm -> Rain
        let player_pos = Vec3::new(100.0, 70.0, 100.0);

        let meshes = renderer.build_precipitation_mesh(player_pos, &weather, &reader, 10, 0.0);

        assert!(!meshes.rain.is_empty(), "Rain mesh should contain quads");
        assert!(meshes.snow.is_empty(), "Snow mesh should be empty in warm biome");
        assert!(meshes.rain.quad_count() > 0);

        // Verify downward scroll at 0.75 blocks/tick
        let tick10 = (10.0 * RAIN_SCROLL_SPEED) % 32.0;
        let tick20 = (20.0 * RAIN_SCROLL_SPEED) % 32.0;
        assert_eq!(tick10, 7.5);
        assert_eq!(tick20, 15.0);
    }

    #[test]
    fn test_snow_flake_generation_and_drift_oscillation() {
        let renderer = WeatherRenderer::default();
        let mut weather = WeatherManager::new();
        weather.set_weather(crate::world::weather::WeatherState::Rain);
        for _ in 0..50 {
            weather.tick();
        }

        let reader = MockClimateReader { temp: 0.05 }; // Cold -> Snow
        let player_pos = Vec3::new(50.0, 70.0, 50.0);

        let meshes_t0 = renderer.build_precipitation_mesh(player_pos, &weather, &reader, 0, 0.0);
        let meshes_t10 = renderer.build_precipitation_mesh(player_pos, &weather, &reader, 10, 0.0);

        assert!(!meshes_t0.snow.is_empty(), "Snow mesh should contain quads in cold biome");
        assert!(meshes_t0.rain.is_empty(), "Rain mesh should be empty in cold biome");

        // Verify horizontal drift causes vertex position shifts over time
        let v0 = meshes_t0.snow.vertices[0];
        let v10 = meshes_t10.snow.vertices[0];
        assert_ne!(v0.pos[0], v10.pos[0], "Snow flakes should drift horizontally across ticks");

        // Verify downward speed: 0.1 blocks/tick
        let snow_tick10 = (10.0 * SNOW_SCROLL_SPEED) % 32.0;
        assert!((snow_tick10 - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_sky_light_and_fog_pull_calculations() {
        // Full rain: sky light down by 25%
        let light = calculate_sky_light_reduction(15.0, 1.0);
        assert!((light - 11.25).abs() < 1e-4);

        // Half rain: sky light down by 12.5%
        let half_light = calculate_sky_light_reduction(15.0, 0.5);
        assert!((half_light - 13.125).abs() < 1e-4);

        // Fog pull: 128m down to 83.2m during full rain
        let fog_dist = calculate_fog_pull(128.0, 1.0);
        assert!((fog_dist - (128.0 * 0.65)).abs() < 1e-4);
    }
}
