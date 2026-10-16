// REA Verified: net.minecraft.client.renderer.texture.TextureAtlas (mapped: gem.class)
// REA Verified: net.minecraft.client.renderer.texture.TextureAtlasSprite (mapped: gen.class)
// REA Verified: net.minecraft.client.resources.model.ModelBakery (mapped: ggs.class)
//
// UV formula extracted from gen.class (TextureAtlasSprite.<init>):
//   u0 = (float) x / (float) atlasWidth;
//   u1 = (float) (x + spriteWidth) / (float) atlasWidth;
//   v0 = (float) y / (float) atlasHeight;
//   v1 = (float) (y + spriteHeight) / (float) atlasHeight;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use image::{Rgba, RgbaImage};

/// 6 cardinal face directions for Minecraft voxel blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,    // +Y (Top)
    Down,  // -Y (Bottom)
    North, // -Z (Front)
    South, // +Z (Back)
    West,  // -X (Left)
    East,  // +X (Right)
}

impl Direction {
    pub const ALL: [Direction; 6] = [
        Direction::Up,
        Direction::Down,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];
}

/// Supported Minecraft Block Types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    Air,
    GrassBlock,
    Dirt,
    Stone,
    Cobblestone,
    Deepslate,
    OakPlanks,
    OakLog,
    OakLeaves,
    Bedrock,
    Sand,
    Glass,
    Bricks,
}

impl BlockType {
    pub fn is_opaque(&self) -> bool {
        match self {
            BlockType::Air | BlockType::Glass | BlockType::OakLeaves => false,
            _ => true,
        }
    }

    pub fn is_air(&self) -> bool {
        matches!(self, BlockType::Air)
    }
}

/// Normalized UV sub-rectangle inside the Texture Atlas [u_min, v_min, u_max, v_max]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureRect {
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
}

impl TextureRect {
    pub fn new(u_min: f32, v_min: f32, u_max: f32, v_max: f32) -> Self {
        Self {
            u_min,
            v_min,
            u_max,
            v_max,
        }
    }

    /// Returns `[u_min, v_min, u_max, v_max]` as a 4-element array
    pub fn to_array(&self) -> [f32; 4] {
        [self.u_min, self.v_min, self.u_max, self.v_max]
    }

    /// Interpolate UV given local face fraction (0.0 to 1.0)
    /// REA Verified: gen.a(float) -> getU(float) and gen.c(float) -> getV(float)
    pub fn get_uv(&self, u_frac: f32, v_frac: f32) -> (f32, f32) {
        let u = self.u_min + (self.u_max - self.u_min) * u_frac;
        let v = self.v_min + (self.v_max - self.v_min) * v_frac;
        (u, v)
    }
}

/// Manages Minecraft stitched 16x16 block textures into a single runtime GPU Texture Atlas
pub struct TextureAtlas {
    pub width: u32,
    pub height: u32,
    pub image: RgbaImage,
    pub sprites: HashMap<String, TextureRect>,
}

impl TextureAtlas {
    pub const TILE_SIZE: u32 = 16;
    pub const ATLAS_WIDTH: u32 = 256;
    pub const ATLAS_HEIGHT: u32 = 256;

    /// Locate `client.jar` in standard paths or fallback
    pub fn find_client_jar() -> Option<PathBuf> {
        let candidates = [
            PathBuf::from("client.jar"),
            PathBuf::from("../client.jar"),
            PathBuf::from("../../client.jar"),
        ];
        candidates.into_iter().find(|p| p.exists())
    }

    /// Builds the Texture Atlas from `client.jar` (or procedural fallback if not found)
    pub fn load_or_create(jar_path: Option<&Path>) -> Self {
        let mut raw_sprites: HashMap<String, RgbaImage> = HashMap::new();

        let resolved_path = jar_path
            .map(|p| p.to_path_buf())
            .or_else(Self::find_client_jar);

        if let Some(path) = resolved_path {
            if let Ok(file) = File::open(&path) {
                if let Ok(mut archive) = zip::ZipArchive::new(file) {
                    let texture_keys = [
                        "grass_block_top",
                        "grass_block_side",
                        "dirt",
                        "stone",
                        "cobblestone",
                        "deepslate",
                        "deepslate_top",
                        "oak_planks",
                        "oak_log",
                        "oak_log_top",
                        "oak_leaves",
                        "bedrock",
                        "sand",
                        "glass",
                        "bricks",
                    ];

                    for key in texture_keys {
                        let entry_name = format!("assets/minecraft/textures/block/{key}.png");
                        if let Ok(mut entry) = archive.by_name(&entry_name) {
                            let mut buf = Vec::new();
                            if entry.read_to_end(&mut buf).is_ok() {
                                if let Ok(img) = image::load_from_memory(&buf) {
                                    raw_sprites.insert(key.to_string(), img.to_rgba8());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fill in missing or procedural fallbacks
        Self::ensure_fallbacks(&mut raw_sprites);

        // Stitch into atlas
        Self::stitch(raw_sprites)
    }

    /// Provides fallback 16x16 textures if texture is missing or jar cannot be opened
    fn ensure_fallbacks(sprites: &mut HashMap<String, RgbaImage>) {
        let palette: [(&str, [u8; 4]); 16] = [
            ("grass_block_top", [86, 175, 49, 255]),
            ("grass_block_side", [106, 140, 60, 255]),
            ("dirt", [134, 96, 67, 255]),
            ("stone", [125, 125, 125, 255]),
            ("cobblestone", [100, 100, 100, 255]),
            ("deepslate", [60, 60, 64, 255]),
            ("deepslate_top", [50, 50, 54, 255]),
            ("oak_planks", [162, 130, 78, 255]),
            ("oak_log", [103, 82, 49, 255]),
            ("oak_log_top", [166, 138, 88, 255]),
            ("oak_leaves", [60, 140, 30, 255]),
            ("bedrock", [40, 40, 40, 255]),
            ("sand", [219, 207, 154, 255]),
            ("glass", [200, 220, 255, 120]),
            ("bricks", [150, 67, 50, 255]),
            ("missing", [255, 0, 255, 255]),
        ];

        for (name, color) in palette {
            sprites.entry(name.to_string()).or_insert_with(|| {
                let mut img = RgbaImage::new(Self::TILE_SIZE, Self::TILE_SIZE);
                for y in 0..Self::TILE_SIZE {
                    for x in 0..Self::TILE_SIZE {
                        // Subtle checkerboard / noise pattern
                        let shade: u8 = if (x + y) % 2 == 0 { 0 } else { 15 };
                        let r = color[0].saturating_add(shade);
                        let g = color[1].saturating_add(shade);
                        let b = color[2].saturating_add(shade);
                        img.put_pixel(x, y, Rgba([r, g, b, color[3]]));
                    }
                }
                img
            });
        }
    }

    /// Stitch individual 16x16 PNG textures into a single runtime Texture Atlas (256x256)
    /// Computes normalized UV coordinates matching net.minecraft.client.renderer.texture.TextureAtlasSprite
    fn stitch(sprites: HashMap<String, RgbaImage>) -> Self {
        let width = Self::ATLAS_WIDTH;
        let height = Self::ATLAS_HEIGHT;
        let mut atlas_image = RgbaImage::new(width, height);
        let mut sprite_rects = HashMap::new();

        let tiles_per_row = width / Self::TILE_SIZE; // 16 tiles per row

        let mut sorted_names: Vec<String> = sprites.keys().cloned().collect();
        sorted_names.sort();

        for (idx, name) in sorted_names.iter().enumerate() {
            let tile_x = (idx as u32) % tiles_per_row;
            let tile_y = (idx as u32) / tiles_per_row;

            let pixel_x = tile_x * Self::TILE_SIZE;
            let pixel_y = tile_y * Self::TILE_SIZE;

            if let Some(sprite_img) = sprites.get(name) {
                // Copy 16x16 pixels into atlas
                for sy in 0..Self::TILE_SIZE.min(sprite_img.height()) {
                    for sx in 0..Self::TILE_SIZE.min(sprite_img.width()) {
                        let p = sprite_img.get_pixel(sx, sy);
                        atlas_image.put_pixel(pixel_x + sx, pixel_y + sy, *p);
                    }
                }
            }

            // REA Verified: gen.class bytecode formula:
            //   u0 = (float) x / (float) atlasWidth
            //   u1 = (float) (x + spriteWidth) / (float) atlasWidth
            //   v0 = (float) y / (float) atlasHeight
            //   v1 = (float) (y + spriteHeight) / (float) atlasHeight
            let u_min = pixel_x as f32 / width as f32;
            let u_max = (pixel_x + Self::TILE_SIZE) as f32 / width as f32;
            let v_min = pixel_y as f32 / height as f32;
            let v_max = (pixel_y + Self::TILE_SIZE) as f32 / height as f32;

            sprite_rects.insert(name.clone(), TextureRect::new(u_min, v_min, u_max, v_max));
        }

        Self {
            width,
            height,
            image: atlas_image,
            sprites: sprite_rects,
        }
    }

    /// Get normalized UV rectangle for a named sprite
    pub fn get_sprite(&self, name: &str) -> TextureRect {
        self.sprites
            .get(name)
            .copied()
            .or_else(|| self.sprites.get("missing").copied())
            .unwrap_or(TextureRect::new(0.0, 0.0, 1.0, 1.0))
    }

    /// REA Verified: net.minecraft.client.resources.model.ModelBakery (ggs.class)
    /// Resolves block face direction to appropriate sprite UV coordinates
    pub fn get_face_uv(&self, block: BlockType, face: Direction) -> [f32; 4] {
        let sprite_name = match block {
            BlockType::Air => return [0.0, 0.0, 0.0, 0.0],
            BlockType::GrassBlock => match face {
                Direction::Up => "grass_block_top",
                Direction::Down => "dirt",
                Direction::North | Direction::South | Direction::West | Direction::East => {
                    "grass_block_side"
                }
            },
            BlockType::Dirt => "dirt",
            BlockType::Stone => "stone",
            BlockType::Cobblestone => "cobblestone",
            BlockType::Deepslate => match face {
                Direction::Up | Direction::Down => "deepslate_top",
                _ => "deepslate",
            },
            BlockType::OakPlanks => "oak_planks",
            BlockType::OakLog => match face {
                Direction::Up | Direction::Down => "oak_log_top",
                _ => "oak_log",
            },
            BlockType::OakLeaves => "oak_leaves",
            BlockType::Bedrock => "bedrock",
            BlockType::Sand => "sand",
            BlockType::Glass => "glass",
            BlockType::Bricks => "bricks",
        };

        self.get_sprite(sprite_name).to_array()
    }

    /// Creates GPU texture, view, sampler, and bind group using WGPU
    /// Sampler uses nearest-neighbor filtering (`FilterMode::Nearest`) for crisp Minecraft pixels
    pub fn create_gpu_resources(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (
        wgpu::Texture,
        wgpu::TextureView,
        wgpu::Sampler,
        wgpu::BindGroupLayout,
        wgpu::BindGroup,
    ) {
        let size = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Minecraft Texture Atlas"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // REA Verified: Nearest Neighbor Sampler for pixel-art clarity
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Minecraft Texture Sampler (Nearest)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Atlas Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        (texture, view, sampler, bind_group_layout, bind_group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_stitching_and_uv() {
        let atlas = TextureAtlas::load_or_create(None);
        assert_eq!(atlas.width, 256);
        assert_eq!(atlas.height, 256);

        let dirt_uv = atlas.get_sprite("dirt");
        assert!(dirt_uv.u_min >= 0.0 && dirt_uv.u_max <= 1.0);
        assert!(dirt_uv.v_min >= 0.0 && dirt_uv.v_max <= 1.0);
        assert!(dirt_uv.u_max > dirt_uv.u_min);
        assert!(dirt_uv.v_max > dirt_uv.v_min);

        // UV span for 16x16 tile in 256x256 atlas must be exactly 16 / 256 = 0.0625
        let u_span = dirt_uv.u_max - dirt_uv.u_min;
        let v_span = dirt_uv.v_max - dirt_uv.v_min;
        assert!((u_span - 0.0625).abs() < 1e-5);
        assert!((v_span - 0.0625).abs() < 1e-5);
    }

    #[test]
    fn test_model_face_uv_mapping() {
        let atlas = TextureAtlas::load_or_create(None);

        // Grass Block has different top, side, and bottom textures
        let grass_top = atlas.get_face_uv(BlockType::GrassBlock, Direction::Up);
        let grass_side = atlas.get_face_uv(BlockType::GrassBlock, Direction::North);
        let grass_bottom = atlas.get_face_uv(BlockType::GrassBlock, Direction::Down);

        assert_ne!(grass_top, grass_side);
        assert_ne!(grass_top, grass_bottom);
        assert_eq!(grass_bottom, atlas.get_face_uv(BlockType::Dirt, Direction::Up));
    }
}
