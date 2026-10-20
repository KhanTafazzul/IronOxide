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

/// Supported Minecraft Block Types (aliased to canonical BlockId)
pub use crate::world::block_registry::BlockId;
pub type BlockType = BlockId;

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
                        "birch_planks",
                        "birch_log",
                        "birch_log_top",
                        "birch_leaves",
                        "spruce_planks",
                        "spruce_log",
                        "spruce_log_top",
                        "spruce_leaves",
                        "dark_oak_planks",
                        "dark_oak_log",
                        "dark_oak_log_top",
                        "dark_oak_leaves",
                        "andesite",
                        "granite",
                        "diorite",
                        "gravel",
                        "coal_ore",
                        "iron_ore",
                        "copper_ore",
                        "gold_ore",
                        "diamond_ore",
                        "redstone_ore",
                        "lapis_ore",
                        "emerald_ore",
                        "bedrock",
                        "sand",
                        "glass",
                        "bricks",
                        "obsidian",
                        "vine",
                        "netherrack",
                        "soul_sand",
                        "soul_soil",
                        "basalt_top",
                        "basalt_side",
                        "polished_basalt_top",
                        "polished_basalt_side",
                        "blackstone_top",
                        "blackstone",
                        "nether_bricks",
                        "glowstone",
                        "crimson_nylium",
                        "crimson_nylium_side",
                        "warped_nylium",
                        "warped_nylium_side",
                        "crimson_stem",
                        "crimson_stem_top",
                        "warped_stem",
                        "warped_stem_top",
                        "nether_wart_block",
                        "warped_wart_block",
                        "crying_obsidian",
                        "bone_block_side",
                        "bone_block_top",
                        "magma",
                        "end_stone",
                        "end_stone_bricks",
                        "purpur_block",
                        "purpur_pillar",
                        "purpur_pillar_top",
                        "end_rod",
                        "end_portal_frame_top",
                        "end_portal_frame_side",
                        "chorus_plant",
                        "chorus_flower",
                        "nether_portal",
                        "end_portal",
                        "dragon_head",
                        "destroy_stage_0",
                        "destroy_stage_1",
                        "destroy_stage_2",
                        "destroy_stage_3",
                        "destroy_stage_4",
                        "destroy_stage_5",
                        "destroy_stage_6",
                        "destroy_stage_7",
                        "destroy_stage_8",
                        "destroy_stage_9",
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
        let palette: &[(&str, [u8; 4])] = &[
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
            ("birch_planks", [196, 178, 133, 255]),
            ("birch_log", [215, 215, 210, 255]),
            ("birch_log_top", [190, 175, 140, 255]),
            ("birch_leaves", [80, 150, 45, 255]),
            ("spruce_planks", [114, 84, 52, 255]),
            ("spruce_log", [59, 41, 24, 255]),
            ("spruce_log_top", [95, 75, 48, 255]),
            ("spruce_leaves", [45, 95, 45, 255]),
            ("dark_oak_planks", [66, 43, 20, 255]),
            ("dark_oak_log", [54, 38, 21, 255]),
            ("dark_oak_log_top", [75, 55, 33, 255]),
            ("dark_oak_leaves", [35, 80, 25, 255]),
            ("andesite", [130, 130, 130, 255]),
            ("granite", [150, 100, 90, 255]),
            ("diorite", [185, 185, 185, 255]),
            ("gravel", [120, 115, 115, 255]),
            ("coal_ore", [40, 40, 40, 255]),
            ("iron_ore", [180, 150, 130, 255]),
            ("copper_ore", [190, 110, 80, 255]),
            ("gold_ore", [220, 190, 60, 255]),
            ("diamond_ore", [80, 220, 220, 255]),
            ("redstone_ore", [220, 40, 40, 255]),
            ("lapis_ore", [30, 70, 180, 255]),
            ("emerald_ore", [40, 200, 80, 255]),
            ("bedrock", [40, 40, 40, 255]),
            ("sand", [219, 207, 154, 255]),
            ("glass", [200, 220, 255, 120]),
            ("bricks", [150, 67, 50, 255]),
            ("obsidian", [20, 18, 30, 255]),
            ("torch", [255, 210, 40, 255]),
            ("lava", [240, 110, 20, 255]),
            ("water", [40, 80, 220, 180]),
            ("vine", [75, 145, 45, 255]),
            ("chest_top", [160, 115, 60, 255]),
            ("chest_side", [150, 105, 55, 255]),
            ("chest_front", [140, 95, 50, 255]),
            ("netherrack", [110, 35, 35, 255]),
            ("soul_sand", [80, 60, 50, 255]),
            ("soul_soil", [75, 55, 45, 255]),
            ("basalt_top", [80, 80, 85, 255]),
            ("basalt_side", [50, 50, 55, 255]),
            ("polished_basalt_top", [90, 90, 95, 255]),
            ("polished_basalt_side", [70, 70, 75, 255]),
            ("blackstone", [40, 35, 42, 255]),
            ("blackstone_top", [45, 40, 48, 255]),
            ("nether_bricks", [45, 20, 25, 255]),
            ("glowstone", [220, 180, 100, 255]),
            ("crimson_nylium", [140, 25, 30, 255]),
            ("crimson_nylium_side", [120, 30, 35, 255]),
            ("warped_nylium", [25, 115, 110, 255]),
            ("warped_nylium_side", [30, 90, 85, 255]),
            ("crimson_stem", [100, 25, 45, 255]),
            ("crimson_stem_top", [125, 40, 65, 255]),
            ("warped_stem", [45, 95, 95, 255]),
            ("warped_stem_top", [60, 120, 120, 255]),
            ("nether_wart_block", [120, 10, 15, 255]),
            ("warped_wart_block", [20, 130, 120, 255]),
            ("crying_obsidian", [40, 20, 60, 255]),
            ("bone_block_side", [225, 220, 200, 255]),
            ("bone_block_top", [215, 210, 190, 255]),
            ("magma", [180, 50, 10, 255]),
            ("end_stone", [220, 220, 160, 255]),
            ("end_stone_bricks", [215, 215, 155, 255]),
            ("purpur_block", [168, 122, 168, 255]),
            ("purpur_pillar", [175, 130, 175, 255]),
            ("purpur_pillar_top", [160, 115, 160, 255]),
            ("end_rod", [230, 230, 230, 255]),
            ("end_portal_frame_top", [90, 130, 110, 255]),
            ("end_portal_frame_side", [115, 135, 115, 255]),
            ("chorus_plant", [95, 55, 95, 255]),
            ("chorus_flower", [220, 180, 220, 255]),
            ("nether_portal", [148, 60, 220, 180]),
            ("end_portal", [15, 10, 25, 255]),
            ("dragon_head", [35, 35, 40, 255]),
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

        // Procedural fallbacks for destroy stage crack overlays 0..=9
        for stage in 0..10 {
            let name = format!("destroy_stage_{stage}");
            sprites.entry(name).or_insert_with(|| {
                let mut img = RgbaImage::new(Self::TILE_SIZE, Self::TILE_SIZE);
                let threshold = (stage + 1) as u32 * 2;
                for y in 0..Self::TILE_SIZE {
                    for x in 0..Self::TILE_SIZE {
                        // Procedural jagged crack lines radiating from center (8, 8)
                        let dx = (x as i32 - 8).abs() as u32;
                        let dy = (y as i32 - 8).abs() as u32;
                        let dist = dx + dy;
                        let is_crack = (dist <= threshold && ((x * 7 + y * 13) % 5 == 0))
                            || (dx == dy && dist <= threshold + 2);

                        if is_crack {
                            img.put_pixel(x, y, Rgba([15, 15, 15, 210]));
                        } else {
                            img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                        }
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

    /// REA Verified: net.minecraft.client.renderer.block.BlockModelShaper (fua.class)
    /// Returns normalized UV coordinates for destroy crack stage (0 to 9)
    pub fn get_destroy_stage_uv(&self, stage: usize) -> [f32; 4] {
        let name = format!("destroy_stage_{}", stage.min(9));
        let rect = self.get_sprite(&name);
        [rect.u_min, rect.v_min, rect.u_max, rect.v_max]
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

            BlockType::BirchPlanks => "birch_planks",
            BlockType::BirchLog => match face {
                Direction::Up | Direction::Down => "birch_log_top",
                _ => "birch_log",
            },
            BlockType::BirchLeaves => "birch_leaves",

            BlockType::SprucePlanks => "spruce_planks",
            BlockType::SpruceLog => match face {
                Direction::Up | Direction::Down => "spruce_log_top",
                _ => "spruce_log",
            },
            BlockType::SpruceLeaves => "spruce_leaves",

            BlockType::DarkOakPlanks => "dark_oak_planks",
            BlockType::DarkOakLog => match face {
                Direction::Up | Direction::Down => "dark_oak_log_top",
                _ => "dark_oak_log",
            },
            BlockType::DarkOakLeaves => "dark_oak_leaves",

            BlockType::PaleOakPlanks => "pale_oak_planks",
            BlockType::PaleOakLog | BlockType::PaleOakWood => match face {
                Direction::Up | Direction::Down => "pale_oak_log_top",
                _ => "pale_oak_log",
            },
            BlockType::PaleOakLeaves => "pale_oak_leaves",
            BlockType::PaleMossBlock => "pale_moss_block",
            BlockType::PaleMossCarpet => "pale_moss_carpet",
            BlockType::PaleHangingMoss => "pale_hanging_moss",
            BlockType::CreakingHeart => match face {
                Direction::Up | Direction::Down => "creaking_heart_top",
                _ => "creaking_heart",
            },

            BlockType::Andesite => "andesite",
            BlockType::Granite => "granite",
            BlockType::Diorite => "diorite",
            BlockType::Gravel => "gravel",

            BlockType::CoalOre => "coal_ore",
            BlockType::IronOre => "iron_ore",
            BlockType::CopperOre => "copper_ore",
            BlockType::GoldOre => "gold_ore",
            BlockType::DiamondOre => "diamond_ore",
            BlockType::RedstoneOre => "redstone_ore",
            BlockType::LapisOre => "lapis_ore",
            BlockType::EmeraldOre => "emerald_ore",

            BlockType::OakSlab | BlockType::OakStairs => "oak_planks",
            BlockType::StoneSlab => "stone",
            BlockType::CobblestoneSlab | BlockType::CobblestoneStairs => "cobblestone",

            BlockType::Bedrock => "bedrock",
            BlockType::Sand => "sand",
            BlockType::Glass => "glass",
            BlockType::Bricks => "bricks",
            BlockType::Torch => "torch",
            BlockType::Lava => "lava",
            BlockType::Water => "water",
            BlockType::Obsidian => "obsidian",
            BlockType::Vine => "vine",
            BlockType::Chest => match face {
                Direction::Up | Direction::Down => "chest_top",
                Direction::North => "chest_front",
                _ => "chest_side",
            },

            BlockType::Netherrack => "netherrack",
            BlockType::SoulSand => "soul_sand",
            BlockType::SoulSoil => "soul_soil",
            BlockType::Basalt => match face {
                Direction::Up | Direction::Down => "basalt_top",
                _ => "basalt_side",
            },
            BlockType::PolishedBasalt => match face {
                Direction::Up | Direction::Down => "polished_basalt_top",
                _ => "polished_basalt_side",
            },
            BlockType::Blackstone => match face {
                Direction::Up | Direction::Down => "blackstone_top",
                _ => "blackstone",
            },
            BlockType::NetherBricks => "nether_bricks",
            BlockType::Glowstone => "glowstone",
            BlockType::CrimsonNylium => match face {
                Direction::Up => "crimson_nylium",
                Direction::Down => "netherrack",
                _ => "crimson_nylium_side",
            },
            BlockType::WarpedNylium => match face {
                Direction::Up => "warped_nylium",
                Direction::Down => "netherrack",
                _ => "warped_nylium_side",
            },
            BlockType::CrimsonStem => match face {
                Direction::Up | Direction::Down => "crimson_stem_top",
                _ => "crimson_stem",
            },
            BlockType::WarpedStem => match face {
                Direction::Up | Direction::Down => "warped_stem_top",
                _ => "warped_stem",
            },
            BlockType::NetherWartBlock => "nether_wart_block",
            BlockType::WarpedWartBlock => "warped_wart_block",
            BlockType::CryingObsidian => "crying_obsidian",
            BlockType::BoneBlock => match face {
                Direction::Up | Direction::Down => "bone_block_top",
                _ => "bone_block_side",
            },
            BlockType::MagmaBlock => "magma",

            BlockType::EndStone => "end_stone",
            BlockType::EndStoneBricks => "end_stone_bricks",
            BlockType::PurpurBlock | BlockType::PurpurSlab | BlockType::PurpurStairs => "purpur_block",
            BlockType::PurpurPillar => match face {
                Direction::Up | Direction::Down => "purpur_pillar_top",
                _ => "purpur_pillar",
            },
            BlockType::EndRod => "end_rod",
            BlockType::EndPortalFrame => match face {
                Direction::Up => "end_portal_frame_top",
                Direction::Down => "end_stone",
                _ => "end_portal_frame_side",
            },
            BlockType::ChorusPlant => "chorus_plant",
            BlockType::ChorusFlower => "chorus_flower",
            BlockType::NetherPortal => "nether_portal",
            BlockType::EndPortal => "end_portal",
            BlockType::DragonHead => "dragon_head",
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
